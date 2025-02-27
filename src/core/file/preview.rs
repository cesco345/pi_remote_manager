use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};

use super::file_type::{FileType, get_file_type_info};

/// Maximum size for text files to be previewed (5MB)
const MAX_TEXT_PREVIEW_SIZE: u64 = 5 * 1024 * 1024;

/// Information about a previewed file
pub struct PreviewInfo {
    /// The path to the file
    pub path: PathBuf,
    /// The file type info
    pub file_type_info: super::file_type::FileTypeInfo,
    /// File size in bytes
    pub size: u64,
    /// Error message if preview generation failed
    pub error: Option<String>,
}

/// Read the first n bytes of a file
pub fn read_file_start(path: &Path, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = vec![0; max_bytes];
    
    let n = file.read(&mut buffer)?;
    buffer.truncate(n);
    
    Ok(buffer)
}

/// Get text content from a file, with size limit
pub fn get_text_preview(path: &Path) -> Result<String, String> {
    // Check file size first
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => return Err(format!("Failed to get file metadata: {}", e)),
    };
    
    if metadata.len() > MAX_TEXT_PREVIEW_SIZE {
        return Err(format!(
            "File too large for preview ({} bytes). Maximum size is {} bytes.", 
            metadata.len(), 
            MAX_TEXT_PREVIEW_SIZE
        ));
    }
    
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file content: {}", e)),
    }
}

/// Get preview info for a file
pub fn get_preview_info(path: &Path) -> PreviewInfo {
    let file_type_info = get_file_type_info(path);
    
    // Get file size
    let size = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };
    
    PreviewInfo {
        path: path.to_path_buf(),
        file_type_info,
        size,
        error: None,
    }
}

/// Create a temporary file for preview
pub fn create_temp_file(suffix: &str) -> io::Result<PathBuf> {
    let mut temp_path = std::env::temp_dir();
    
    // Create a unique filename using current time as milliseconds
    // This avoids the dependency on chrono
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
        
    let file_name = format!("preview_{}{}", timestamp, suffix);
    
    temp_path.push(file_name);
    Ok(temp_path)
}

/// Find all previewable files in a directory
pub fn find_previewable_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let info = get_file_type_info(&path);
                if info.previewable {
                    files.push(path);
                }
            }
        }
    }
    
    files
}