// utils/image.rs - Image utility functions
pub mod image {
    use std::path::{Path, PathBuf};
    use std::fs;
    
    use crate::core::image_processor::image_processor::ImageFormat;
    
    pub fn is_image_file(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp"
            )
        } else {
            false
        }
    }
    
    pub fn get_image_format(path: &Path) -> Option<ImageFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ImageFormat::from_extension(ext))
    }
    
    pub fn find_images_in_dir(dir: &Path) -> Vec<PathBuf> {
        let mut images = Vec::new();
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() && is_image_file(&path) {
                    images.push(path);
                }
            }
        }
        
        images
    }
    
    pub fn generate_output_filename(
        input_path: &Path,
        output_format: ImageFormat,
        suffix: Option<&str>
    ) -> PathBuf {
        let stem = input_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
            
        let mut filename = if let Some(suffix) = suffix {
            format!("{}_{}", stem, suffix)
        } else {
            stem.to_string()
        };
        
        // Add the new extension
        filename.push('.');
        filename.push_str(output_format.extension());
        
        // Get the parent directory or use the current directory
        let parent = input_path.parent().unwrap_or_else(|| Path::new("."));
        
        parent.join(filename)
    }
}