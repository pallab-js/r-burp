use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Hardcoded passphrase for encrypting CA private keys at rest.
/// In a production app, this should be derived from a user-provided password
/// or OS-level keychain. For now, we use encryption-at-rest as a defense-in-depth
/// measure against casual filesystem access.
const KEY_ENCRYPTION_PASSPHRASE: &str = "r-burp-local-key-2026";

/// Manages the CA certificate used for HTTPS interception
pub struct CertManager {
    pub ca_cert: RwLock<Option<CaCertificate>>,
    pub cert_dir: RwLock<Option<PathBuf>>,
}

#[derive(Debug, Clone)]
pub struct CaCertificate {
    pub cert_pem: String,
    pub key_pem: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfo {
    pub generated: bool,
    pub fingerprint: String,
    pub cert_path: String,
    pub installed: bool,
}

impl Default for CertManager {
    fn default() -> Self {
        Self {
            ca_cert: RwLock::new(None),
            cert_dir: RwLock::new(None),
        }
    }
}

impl CertManager {
    /// Initialize cert manager with a directory for storage
    pub fn init(&self, dir: PathBuf) -> Result<(), String> {
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| format!("Failed to create cert dir: {}", e))?;
        }
        *self.cert_dir.write() = Some(dir);

        // Load existing certs if present
        self.load_existing()?;

        Ok(())
    }

    fn load_existing(&self) -> Result<(), String> {
        let dir = self.cert_dir.read();
        let dir = dir.as_ref().ok_or("Cert dir not initialized")?;

        let cert_path = dir.join("ca-cert.pem");
        // Key is now stored encrypted with .enc extension
        let key_path = dir.join("ca-key.pem.enc");

        if cert_path.exists() && key_path.exists() {
            let cert_pem = fs::read_to_string(&cert_path)
                .map_err(|e| format!("Failed to read cert: {}", e))?;
            let encrypted_key = fs::read(&key_path)
                .map_err(|e| format!("Failed to read encrypted key: {}", e))?;
            let key_pem = Self::decrypt_key(&encrypted_key)?;

            // Generate fingerprint from cert using SHA-256
            let fingerprint = Self::fingerprint_from_pem(&cert_pem);

            *self.ca_cert.write() = Some(CaCertificate {
                cert_pem,
                key_pem,
                fingerprint,
            });
        }

        Ok(())
    }

    /// Generate a new CA certificate
    pub fn generate_ca(&self) -> Result<String, String> {
        let mut params = CertificateParams::new(vec!["r-burp-ca.local".to_string()])
            .map_err(|e| format!("Failed to create cert params: {}", e))?;

        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, "r-burp CA");
        dn.push(rcgen::DnType::OrganizationName, "r-burp");
        dn.push(rcgen::DnType::CountryName, "US");
        params.distinguished_name = dn;

        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];

        let key_pair = KeyPair::generate()
            .map_err(|e| format!("Failed to generate key pair: {}", e))?;

        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| format!("Failed to sign cert: {}", e))?;

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();
        let fingerprint = Self::fingerprint_from_pem(&cert_pem);

        let ca_cert = CaCertificate {
            cert_pem: cert_pem.clone(),
            key_pem: key_pem.clone(),
            fingerprint: fingerprint.clone(),
        };

        *self.ca_cert.write() = Some(ca_cert);

        // Save to disk
        if let Some(dir) = self.cert_dir.read().as_ref() {
            let cert_path = dir.join("ca-cert.pem");
            let key_path = dir.join("ca-key.pem.enc");

            fs::write(&cert_path, cert_pem)
                .map_err(|e| format!("Failed to write cert: {}", e))?;

            // Encrypt and save private key
            let encrypted_key = Self::encrypt_key(&key_pem)?;
            fs::write(&key_path, encrypted_key)
                .map_err(|e| format!("Failed to write encrypted key: {}", e))?;

            // Restrict key file permissions to owner read/write only
            #[cfg(unix)]
            {
                let mut perms = fs::metadata(&key_path)
                    .map_err(|e| format!("Failed to read key metadata: {}", e))?
                    .permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&key_path, perms)
                    .map_err(|e| format!("Failed to set key permissions: {}", e))?;
            }
        }

        Ok(fingerprint)
    }

    /// Get the CA certificate info
    pub fn get_cert_info(&self) -> CertInfo {
        let cert = self.ca_cert.read();
        match cert.as_ref() {
            Some(c) => CertInfo {
                generated: true,
                fingerprint: c.fingerprint.clone(),
                cert_path: "r-burp-ca".to_string(),
                installed: false,
            },
            None => CertInfo {
                generated: false,
                fingerprint: String::new(),
                cert_path: String::new(),
                installed: false,
            },
        }
    }

    /// Get the CA certificate PEM
    pub fn get_cert_pem(&self) -> Option<String> {
        self.ca_cert.read().as_ref().map(|c| c.cert_pem.clone())
    }

    /// Get the CA private key PEM
    pub fn get_key_pem(&self) -> Option<String> {
        self.ca_cert.read().as_ref().map(|c| c.key_pem.clone())
    }

    /// Generate a certificate for a specific domain, signed by our CA
    pub fn generate_domain_cert(&self, domain: &str) -> Option<(String, String)> {
        let ca = self.ca_cert.read();
        let ca = ca.as_ref()?;

        // Parse the CA key
        let ca_key = KeyPair::from_pem(&ca.key_pem).ok()?;

        // Reconstruct CA cert params from stored PEM
        let mut ca_params = CertificateParams::new(vec!["r-burp-ca.local".to_string()]).ok()?;
        let mut dn = rcgen::DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, "r-burp CA");
        dn.push(rcgen::DnType::OrganizationName, "r-burp");
        dn.push(rcgen::DnType::CountryName, "US");
        ca_params.distinguished_name = dn;
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];

        // Recreate the CA cert (same params as original)
        let ca_cert = ca_params.self_signed(&ca_key).ok()?;

        // Create domain cert params
        let mut params = CertificateParams::new(vec![domain.to_string()]).ok()?;
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, domain);
        dn.push(rcgen::DnType::OrganizationName, "r-burp");
        params.distinguished_name = dn;

        let key_pair = KeyPair::generate().ok()?;

        // Sign the domain cert with our CA key
        let cert = params.signed_by(&key_pair, &ca_cert, &ca_key).ok()?;

        Some((cert.pem(), key_pair.serialize_pem()))
    }

    /// Generate a SHA-256 fingerprint from the cert PEM
    fn fingerprint_from_pem(pem: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pem.as_bytes());
        let result = hasher.finalize();
        // Use first 16 hex chars of the SHA-256 hash for a compact fingerprint
        format!("{:x}", result)
    }

    /// Encrypt the private key PEM data using XOR with a derived key stream.
    /// This is a lightweight obfuscation to prevent casual reading of the key file.
    /// For production use, consider proper AES encryption.
    fn encrypt_key(key_pem: &str) -> Result<Vec<u8>, String> {
        let key_bytes = key_pem.as_bytes();
        let passphrase = KEY_ENCRYPTION_PASSPHRASE.as_bytes();
        let mut encrypted = Vec::with_capacity(8 + key_bytes.len());

        // Add a simple magic header to identify encrypted format
        encrypted.extend_from_slice(b"RBURPEK1");

        // XOR with passphrase-derived key stream
        for (i, &byte) in key_bytes.iter().enumerate() {
            let key_byte = passphrase[i % passphrase.len()];
            encrypted.push(byte ^ key_byte);
        }

        Ok(encrypted)
    }

    /// Decrypt the private key from the encrypted file format.
    fn decrypt_key(encrypted: &[u8]) -> Result<String, String> {
        // Validate minimum length (8 byte header + at least 1 byte of data)
        if encrypted.len() < 9 {
            return Err("Encrypted key file too small".to_string());
        }

        // Check magic header
        if &encrypted[0..8] != b"RBURPEK1" {
            return Err("Invalid encrypted key format".to_string());
        }

        let passphrase = KEY_ENCRYPTION_PASSPHRASE.as_bytes();
        let mut decrypted = Vec::with_capacity(encrypted.len() - 8);

        // XOR with same key stream to decrypt
        for (i, &byte) in encrypted[8..].iter().enumerate() {
            let key_byte = passphrase[i % passphrase.len()];
            decrypted.push(byte ^ key_byte);
        }

        String::from_utf8(decrypted)
            .map_err(|_| "Failed to decrypt key: invalid passphrase or corrupted data".to_string())
    }
}
