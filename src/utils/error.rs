// src/utils/error.rs - Error handling utilities
pub mod error {
    use std::fmt;
    use std::error::Error;
    
    #[derive(Debug)]
    pub enum AppError {
        ConfigError(String),
        FileError(String),
        NetworkError(String),
        ProcessingError(String),
        UIError(String),
    }
    
    impl fmt::Display for AppError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
                Self::FileError(msg) => write!(f, "File error: {}", msg),
                Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
                Self::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
                Self::UIError(msg) => write!(f, "UI error: {}", msg),
            }
        }
    }
    
    impl Error for AppError {}
    
    pub type AppResult<T> = Result<T, AppError>;
    
    pub fn log_error(error: &dyn Error) {
        eprintln!("Error: {}", error);
        
        let mut source = error.source();
        while let Some(err) = source {
            eprintln!("Caused by: {}", err);
            source = err.source();
        }
    }
}
