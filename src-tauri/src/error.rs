/// Errors from the certificate management module
#[derive(Debug, thiserror::Error)]
pub enum CertError {
    #[error("cert dir not initialized")]
    DirNotInitialized,
    #[error("failed to create cert dir: {0}")]
    DirCreate(String),
    #[error("failed to read cert file: {0}")]
    FileRead(String),
    #[error("failed to write cert file: {0}")]
    FileWrite(String),
    #[error("key decrypt failed: {0}")]
    Decrypt(String),
    #[error("key encrypt failed: {0}")]
    Encrypt(String),
    #[error("passphrase must not be empty")]
    EmptyPassphrase,
    #[error("rcgen error: {0}")]
    Rcgen(String),
}

/// Errors from the proxy server module
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("request has too many headers: {count} > {limit}")]
    TooManyHeaders { count: usize, limit: usize },
    #[error("request body too large: {size} > {limit} bytes")]
    RequestBodyTooLarge { size: usize, limit: usize },
    #[error("response body too large: {size} > {limit} bytes")]
    ResponseBodyTooLarge { size: usize, limit: usize },
    #[error("CONNECT request too large")]
    ConnectRequestTooLarge,
    #[error("malformed CONNECT request: {0}")]
    MalformedConnect(String),
    #[error("upstream error: {0}")]
    Upstream(String),
}
