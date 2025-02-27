use std::path::Path;

/// Represents different file types that can be handled by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Image files (jpg, png, etc.)
    Image,
    /// Text files (txt, md, rs, etc.)
    Text,
    /// Document files (pdf, doc, etc.)
    Document,
    /// Code files with syntax highlighting
    Code,
    /// Archive files (zip, tar, etc.)
    Archive,
    /// Media files (audio, video)
    Media,
    /// Unknown or unsupported file type
    Other,
}

/// Result of checking a file for preview support
pub struct FileTypeInfo {
    /// Whether this file can be previewed
    pub previewable: bool,
    /// The detected file type
    pub file_type: FileType,
    /// MIME type if known
    pub mime_type: Option<String>,
}

/// Check if a file is an image file
pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "svg"
        )
    } else {
        false
    }
}

/// Check if a file is a text file
pub fn is_text_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "txt" | "md" | "csv" | "json" | "xml" | "html" | "css" | "log"
        )
    } else {
        false
    }
}

/// Check if a file is a code file
pub fn is_code_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "hpp" | "java" | "sh" | "toml" | "yaml" | "yml"
        )
    } else {
        false
    }
}

/// Check if a file is a document file
pub fn is_document_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "pdf" | "doc" | "docx" | "rtf" | "odt" | "xlsx" | "pptx"
        )
    } else {
        false
    }
}

/// Get comprehensive file type information
pub fn get_file_type_info(path: &Path) -> FileTypeInfo {
    if is_image_file(path) {
        return FileTypeInfo {
            previewable: true,
            file_type: FileType::Image,
            mime_type: get_mime_type_for_path(path),
        };
    }

    if is_text_file(path) {
        return FileTypeInfo {
            previewable: true,
            file_type: FileType::Text,
            mime_type: get_mime_type_for_path(path),
        };
    }

    if is_code_file(path) {
        return FileTypeInfo {
            previewable: true,
            file_type: FileType::Code,
            mime_type: get_mime_type_for_path(path),
        };
    }

    if is_document_file(path) {
        return FileTypeInfo {
            previewable: true,
            file_type: FileType::Document,
            mime_type: get_mime_type_for_path(path),
        };
    }

    // Default for unknown file types
    FileTypeInfo {
        previewable: false,
        file_type: FileType::Other,
        mime_type: get_mime_type_for_path(path),
    }
}

/// Simple function to get a MIME type based on file extension
fn get_mime_type_for_path(path: &Path) -> Option<String> {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            // Images
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "bmp" => Some("image/bmp".to_string()),
            "tif" | "tiff" => Some("image/tiff".to_string()),
            "webp" => Some("image/webp".to_string()),
            "svg" => Some("image/svg+xml".to_string()),
            
            // Text
            "txt" => Some("text/plain".to_string()),
            "md" => Some("text/markdown".to_string()),
            "html" | "htm" => Some("text/html".to_string()),
            "css" => Some("text/css".to_string()),
            "csv" => Some("text/csv".to_string()),
            "xml" => Some("text/xml".to_string()),
            
            // Code
            "json" => Some("application/json".to_string()),
            "rs" => Some("text/x-rust".to_string()),
            "py" => Some("text/x-python".to_string()),
            "js" => Some("text/javascript".to_string()),
            "ts" => Some("text/typescript".to_string()),
            "sh" => Some("text/x-sh".to_string()),
            
            // Documents
            "pdf" => Some("application/pdf".to_string()),
            "doc" => Some("application/msword".to_string()),
            "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
            "xls" => Some("application/vnd.ms-excel".to_string()),
            "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
            
            // Other common types
            "zip" => Some("application/zip".to_string()),
            "tar" => Some("application/x-tar".to_string()),
            "gz" => Some("application/gzip".to_string()),
            
            _ => None,
        }
    } else {
        None
    }
}