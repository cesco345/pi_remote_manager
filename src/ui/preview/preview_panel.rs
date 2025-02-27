use fltk::{
    enums::{Color, FrameType},
    group::Group,
    prelude::*,
};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::core::file::{FileType, get_file_type_info};
use crate::ui::preview::image_preview::ImagePreviewComponent;
use crate::ui::preview::text_preview::TextPreviewComponent;

/// A unified preview panel that can display various file types
pub struct PreviewPanel {
    /// Main container group
    pub group: Group,
    /// Image preview component
    image_preview: ImagePreviewComponent,
    /// Text preview component
    text_preview: TextPreviewComponent,
    /// Currently active preview type
    current_type: Option<FileType>,
    /// Currently previewed file path
    current_file: Arc<Mutex<Option<PathBuf>>>,
}

impl Clone for PreviewPanel {
    fn clone(&self) -> Self {
        Self {
            group: self.group.clone(),
            image_preview: self.image_preview.clone(),
            text_preview: self.text_preview.clone(),
            current_type: self.current_type,
            current_file: self.current_file.clone(),
        }
    }
}

impl PreviewPanel {
    /// Create a new preview panel
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        // Create main container
        let mut group = Group::new(x, y, w, h, None);
        group.set_frame(FrameType::FlatBox);
        
        // Create image preview component (initially hidden)
        let image_preview = ImagePreviewComponent::new(x, y, w, h);
        
        // Create text preview component (initially hidden)
        let text_preview = TextPreviewComponent::new(x, y, w, h);
        
        group.end();
        
        // Hide all preview components initially
        image_preview.hide();
        text_preview.hide();
        
        PreviewPanel {
            group,
            image_preview,
            text_preview,
            current_type: None,
            current_file: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Preview a file
    pub fn preview_file(&mut self, path: &Path) -> bool {
        // Clear any existing preview
        self.clear();
        
        // Check if file exists
        if !path.exists() {
            println!("Preview file doesn't exist: {}", path.display());
            return false;
        }
        
        // Get file type info
        let file_type_info = get_file_type_info(path);
        if !file_type_info.previewable {
            println!("File type not supported for preview: {}", path.display());
            return false;
        }
        
        println!("Previewing file: {} (type: {:?})", path.display(), file_type_info.file_type);
        
        // Store current file and type
        self.current_type = Some(file_type_info.file_type);
        {
            let mut current = self.current_file.lock().unwrap();
            *current = Some(path.to_path_buf());
        }
        
        // Show appropriate preview component based on file type
        let result = match file_type_info.file_type {
            FileType::Image => {
                self.image_preview.show();
                self.image_preview.load_image(path)
            },
            FileType::Text | FileType::Code => {
                self.text_preview.show();
                self.text_preview.load_text(path)
            },
            FileType::Document => {
                // For now, try to display documents as text
                self.text_preview.show();
                self.text_preview.load_text(path)
            },
            _ => {
                println!("Unsupported preview type: {:?}", file_type_info.file_type);
                false
            }
        };
        
        // Redraw the group
        self.group.redraw();
        
        result
    }
    
    /// Clear the preview
    pub fn clear(&mut self) {
        // Clear and hide all preview components
        self.image_preview.clear();
        self.image_preview.hide();
        
        self.text_preview.clear();
        self.text_preview.hide();
        
        // Reset state
        self.current_type = None;
        {
            let mut current = self.current_file.lock().unwrap();
            *current = None;
        }
        
        // Redraw
        self.group.redraw();
    }
    
    /// Get the current file being previewed
    pub fn get_current_file(&self) -> Option<PathBuf> {
        let current = self.current_file.lock().unwrap();
        current.clone()
    }
    
    /// Get the current preview type
    pub fn get_current_type(&self) -> Option<FileType> {
        self.current_type
    }
    
    /// Back-compatibility alias for ImageViewPanel
    pub fn load_image(&mut self, path: &Path) -> bool {
        self.preview_file(path)
    }
}