use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::fs;
use std::num::NonZeroUsize;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use crate::error::CertError;

/// Manages the CA certificate used for HTTPS interception
pub struct CertManager {
    pub ca_cert: RwLock<Option<CaCertificate>>,
    pub cert_dir: RwLock<Option<PathBuf>>,
    /// Runtime-generated passphrase stored in tauri-plugin-store
    passphrase: RwLock<String>,
    /// Live rcgen CA key — avoids re-parsing PEM on every domain cert generation
    ca_key_obj: RwLock<Option<KeyPair>>,
    /// In-memory cache of generated domain certs (domain → (cert_pem, key_pem))
    domain_cert_cache: parking_lot::Mutex<lru::LruCache<String, (String, String)>>,
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
            passphrase: RwLock::new(String::new()),
            ca_key_obj: RwLock::new(None),
            domain_cert_cache: parking_lot::Mutex::new(
                lru::LruCache::new(NonZeroUsize::new(256).unwrap())
            ),
        }
    }
}

impl CertManager {
    pub fn with_passphrase(passphrase: String) -> Self {
        Self {
            ca_cert: RwLock::new(None),
            cert_dir: RwLock::new(None),
            passphrase: RwLock::new(passphrase),
            ca_key_obj: RwLock::new(None),
            domain_cert_cache: parking_lot::Mutex::new(
                lru::LruCache::new(NonZeroUsize::new(256).unwrap())
            ),
        }
    }

    /// Initialize cert manager with a directory for storage
    pub fn init(&self, dir: PathBuf) -> Result<(), CertError> {
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| CertError::DirCreate(e.to_string()))?;
        }
        *self.cert_dir.write() = Some(dir);
        self.load_existing()?;
        Ok(())
    }

    fn load_existing(&self) -> Result<(), CertError> {
        let dir = self.cert_dir.read();
        let dir = dir.as_ref().ok_or(CertError::DirNotInitialized)?;

        let cert_path = dir.join("ca-cert.pem");
        let key_path = dir.join("ca-key.pem.enc");

        if cert_path.exists() && key_path.exists() {
            let cert_pem = fs::read_to_string(&cert_path)
                .map_err(|e| CertError::FileRead(e.to_string()))?;
            let encrypted_key = fs::read(&key_path)
                .map_err(|e| CertError::FileRead(e.to_string()))?;
            let passphrase = self.passphrase.read().clone();
            let key_pem = Self::decrypt_key(&encrypted_key, &passphrase)?;

            let fingerprint = Self::fingerprint_from_pem(&cert_pem);
            let key_obj = KeyPair::from_pem(&key_pem).ok();

            *self.ca_cert.write() = Some(CaCertificate { cert_pem, key_pem, fingerprint });
            *self.ca_key_obj.write() = key_obj;
        }

        Ok(())
    }

    /// Generate a new CA certificate
    pub fn generate_ca(&self) -> Result<String, CertError> {
        let mut params = CertificateParams::new(vec!["r-burp-ca.local".to_string()])
            .map_err(|e| CertError::Rcgen(e.to_string()))?;

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

        let key_pair = KeyPair::generate().map_err(|e| CertError::Rcgen(e.to_string()))?;
        let cert = params.self_signed(&key_pair).map_err(|e| CertError::Rcgen(e.to_string()))?;

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();
        let fingerprint = Self::fingerprint_from_pem(&cert_pem);
        let key_obj = KeyPair::from_pem(&key_pem).ok();

        *self.ca_cert.write() = Some(CaCertificate {
            cert_pem: cert_pem.clone(),
            key_pem: key_pem.clone(),
            fingerprint: fingerprint.clone(),
        });
        *self.ca_key_obj.write() = key_obj;
        self.domain_cert_cache.lock().clear();

        if let Some(dir) = self.cert_dir.read().as_ref() {
            let cert_path = dir.join("ca-cert.pem");
            let key_path = dir.join("ca-key.pem.enc");

            fs::write(&cert_path, &cert_pem).map_err(|e| CertError::FileWrite(e.to_string()))?;

            let encrypted_key = Self::encrypt_key(&key_pem, &self.passphrase.read())?;
            fs::write(&key_path, encrypted_key).map_err(|e| CertError::FileWrite(e.to_string()))?;

            #[cfg(unix)]
            {
                let mut perms = fs::metadata(&key_path)
                    .map_err(|e| CertError::FileRead(e.to_string()))?
                    .permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&key_path, perms)
                    .map_err(|e| CertError::FileWrite(e.to_string()))?;
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

    /// Generate a certificate for a specific domain, signed by our CA.
    /// Results are cached in memory for 256 domains.
    pub fn generate_domain_cert(&self, domain: &str) -> Option<(String, String)> {
        // Check cache first
        if let Some(cached) = self.domain_cert_cache.lock().get(domain) {
            return Some(cached.clone());
        }

        let ca = self.ca_cert.read();
        let ca = ca.as_ref()?;

        // Reconstruct CA cert params to sign the domain cert
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
        // Re-parse the CA key to get a fresh owned KeyPair for signing
        let ca_key_for_sign = KeyPair::from_pem(&ca.key_pem).ok()?;
        let ca_cert_obj = ca_params.self_signed(&ca_key_for_sign).ok()?;

        // Create domain cert
        let mut params = CertificateParams::new(vec![domain.to_string()]).ok()?;
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, domain);
        dn.push(rcgen::DnType::OrganizationName, "r-burp");
        params.distinguished_name = dn;

        let domain_key = KeyPair::generate().ok()?;
        let cert = params.signed_by(&domain_key, &ca_cert_obj, &ca_key_for_sign).ok()?;

        let result = (cert.pem(), domain_key.serialize_pem());
        self.domain_cert_cache.lock().put(domain.to_string(), result.clone());
        Some(result)
    }

    /// Generate a SHA-256 fingerprint from the cert PEM
    fn fingerprint_from_pem(pem: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pem.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Encrypt the private key PEM data using XOR with a derived key stream.
    fn encrypt_key(key_pem: &str, passphrase: &str) -> Result<Vec<u8>, CertError> {
        if passphrase.is_empty() {
            return Err(CertError::EmptyPassphrase);
        }
        let key_bytes = key_pem.as_bytes();
        let pass_bytes = passphrase.as_bytes();
        let mut encrypted = Vec::with_capacity(key_bytes.len());
        for (i, &byte) in key_bytes.iter().enumerate() {
            encrypted.push(byte ^ pass_bytes[i % pass_bytes.len()]);
        }
        Ok(encrypted)
    }

    /// Decrypt the private key from the encrypted file format.
    fn decrypt_key(encrypted: &[u8], passphrase: &str) -> Result<String, CertError> {
        if encrypted.is_empty() {
            return Err(CertError::Decrypt("encrypted key file is empty".to_string()));
        }
        if passphrase.is_empty() {
            return Err(CertError::EmptyPassphrase);
        }
        let pass_bytes = passphrase.as_bytes();
        let mut decrypted = Vec::with_capacity(encrypted.len());
        for (i, &byte) in encrypted.iter().enumerate() {
            decrypted.push(byte ^ pass_bytes[i % pass_bytes.len()]);
        }
        String::from_utf8(decrypted)
            .map_err(|_| CertError::Decrypt("invalid passphrase or corrupted data".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passphrase_round_trip() {
        let pem = "-----BEGIN PRIVATE KEY-----\nfakekey\n-----END PRIVATE KEY-----\n";
        let passphrase = "test-passphrase-abc123";
        let encrypted = CertManager::encrypt_key(pem, passphrase).unwrap();
        let decrypted = CertManager::decrypt_key(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, pem);
    }

    #[test]
    fn test_wrong_passphrase_returns_garbage_not_original() {
        let pem = "hello world";
        let encrypted = CertManager::encrypt_key(pem, "correct").unwrap();
        let decrypted = CertManager::decrypt_key(&encrypted, "wrong").unwrap_or_default();
        assert_ne!(decrypted, pem);
    }

    #[test]
    fn test_empty_passphrase_returns_error() {
        let result = CertManager::encrypt_key("data", "");
        assert!(result.is_err());
        let result2 = CertManager::decrypt_key(b"data", "");
        assert!(result2.is_err());
    }

    #[test]
    fn test_generate_ca_produces_valid_pem() {
        let mgr = CertManager::with_passphrase("test-pass".to_string());
        let fingerprint = mgr.generate_ca().unwrap();
        assert!(!fingerprint.is_empty());
        let pem = mgr.get_cert_pem().unwrap();
        assert!(pem.contains("BEGIN CERTIFICATE"));
    }

    #[test]
    fn test_generate_domain_cert_signed_by_ca() {
        let mgr = CertManager::with_passphrase("test-pass".to_string());
        mgr.generate_ca().unwrap();
        let (cert_pem, key_pem) = mgr.generate_domain_cert("example.com").unwrap();
        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(key_pem.contains("BEGIN PRIVATE KEY") || key_pem.contains("BEGIN EC PRIVATE KEY"));
    }

    #[test]
    fn test_domain_cert_cache_hit() {
        let mgr = CertManager::with_passphrase("test-pass".to_string());
        mgr.generate_ca().unwrap();
        let first = mgr.generate_domain_cert("cached.example.com").unwrap();
        let second = mgr.generate_domain_cert("cached.example.com").unwrap();
        // Same cert returned from cache
        assert_eq!(first.0, second.0);
    }

    #[test]
    fn test_cache_cleared_on_ca_regeneration() {
        let mgr = CertManager::with_passphrase("test-pass".to_string());
        mgr.generate_ca().unwrap();
        let first = mgr.generate_domain_cert("example.com").unwrap();
        mgr.generate_ca().unwrap(); // regenerate CA
        let second = mgr.generate_domain_cert("example.com").unwrap();
        // New CA means new domain cert
        assert_ne!(first.0, second.0);
    }
}
