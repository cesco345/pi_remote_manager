use std::path::Path;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TransferError {
    ConnectionFailed(String),
    AuthenticationFailed(String),
    PermissionDenied(String),
    FileNotFound(String),
    TransferFailed(String),
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            Self::TransferFailed(msg) => write!(f, "Transfer failed: {}", msg),
        }
    }
}

impl Error for TransferError {}

// TransferMethod trait - "Product" in our Factory Method pattern
pub trait TransferMethod: Send + Sync {
    fn upload_file(
        &self, 
        local_path: &Path, 
        remote_path: &Path
    ) -> Result<(), TransferError>;
    
    fn download_file(
        &self,
        remote_path: &Path,
        local_path: &Path
    ) -> Result<(), TransferError>;
    
    fn list_files(
        &self,
        remote_dir: &Path
    ) -> Result<Vec<(String, bool)>, TransferError>;
    
    fn get_name(&self) -> &str;
    fn get_description(&self) -> String;
}

// TransferMethodFactory trait - "Creator" in our Factory Method pattern
pub trait TransferMethodFactory {
    fn create_method(&self) -> Box<dyn TransferMethod>;
    fn get_name(&self) -> String;
}