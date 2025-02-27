use fltk::{
    enums::{Color, FrameType, Align},
    group::Group,
    frame::Frame,
    button::Button,
    prelude::*,
};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::process::Command;

/// Component for previewing document files (PDF, DOC, etc.)
pub struct DocumentPreviewComponent {
    /// Container group
    group: Group,
    /// Info display frame
    info_frame: Frame,
    /// Open externally button
    open_button: Button,
    /// Currently loaded file path
    current_file: Arc<Mutex<Option<PathBuf>>>,
}

impl Clone for DocumentPreviewComponent {
    fn clone(&self) -> Self {
        Self {
            group: self.group.clone(),
            info_frame: self.info_frame.clone(),
            open_button: self.open_button.clone(),
            current_file: self.current_file.clone(),
        }
    }
}

impl DocumentPreviewComponent {
    /// Create a new document preview component
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut group = Group::new(x, y, w, h, None);
        group.set_frame(FrameType::FlatBox);
        
        // Add info display area
        let padding = 5;
        let frame_x = x + padding;
        let frame_y = y + padding;
        let frame_w = w - 2 * padding;
        let frame_h = h - 50 - 2 * padding; // Leave space for button
        
        let mut info_frame = Frame::new(
            frame_x,
            frame_y,
            frame_w,
            frame_h,
            None
        );
        info_frame.set_frame(FrameType::BorderFrame);
        info_frame.set_color(Color::from_rgb(245, 245, 245));
        info_frame.set_label_size(14);
        info_frame.set_align(Align::Center | Align::Inside);
        
        // Add button to open the file externally
        let button_x = x + w/2 - 75;
        let button_y = y + h - 40;
        let button_w = 150;
        let button_h = 30;
        
        let mut open_button = Button::new(
            button_x,
            button_y,
            button_w,
            button_h,
            "Open with External App"
        );
        open_button.set_color(Color::from_rgb(230, 230, 230));
        
        group.end();
        
        let preview = DocumentPreviewComponent {
            group,
            info_frame,
            open_button,
            current_file: Arc::new(Mutex::new(None)),
        };
        
        // Setup button callback
        let current_file = preview.current_file.clone();
        preview.open_button.set_callback(move |_| {
            if let Some(path) = {
                let guard = current_file.lock().unwrap();
                guard.clone()
            } {
                // Open the file with the default system application
                #[cfg(target_os = "windows")]
                let _ = Command::new("cmd")
                    .args(&["/c", "start", "", &path.to_string_lossy()])
                    .spawn();
                
                #[cfg(target_os = "macos")]
                let _ = Command::new("open")
                    .arg(&path)
                    .spawn();
                
                #[cfg(target_os = "linux")]
                let _ = Command::new("xdg-open")
                    .arg(&path)
                    .spawn();
            }
        });
        
        preview
    }
    
    /// Load and display document info
    pub fn load_document(&mut self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        
        // Clear any previous content
        self.clear();
        
        // Get file metadata
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                self.info_frame.set_label(&format!("Error accessing file: {}", e));
                return false;
            }
        };
        
        // Display file info
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("[Unknown]");
            
        let file_size = metadata.len();
        let size_str = if file_size < 1024 {
            format!("{} bytes", file_size)
        } else if file_size < 1024 * 1024 {
            format!("{:.1} KB", file_size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", file_size as f64 / (1024.0 * 1024.0))
        };
        
        let file_type = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_uppercase())
            .unwrap_or_else(|| "Unknown".to_string());
            
        let info_text = format!(
            "Document: {}\nType: {} File\nSize: {}\n\nUse the button below to open this document with your default application.",
            file_name,
            file_type,
            size_str
        );
        
        self.info_frame.set_label(&info_text);
        
        // Store the current file path
        let mut current = self.current_file.lock().unwrap();
        *current = Some(path.to_path_buf());
        
        // Show the button
        self.open_button.show();
        
        // Force redraw
        self.group.redraw();
        
        true
    }
    
    /// Get the current file path
    pub fn get_current_file(&self) -> Option<PathBuf> {
        let current = self.current_file.lock().unwrap();
        current.clone()
    }
    
    /// Clear the document preview
    pub fn clear(&mut self) {
        // Clear the info frame
        self.info_frame.set_label("");
        
        // Hide the button
        self.open_button.hide();
        
        // Clear the path reference
        let mut current = self.current_file.lock().unwrap();
        *current = None;
        
        // Force a redraw
        self.group.redraw();
    }
    
    /// Hide the component
    pub fn hide(&mut self) {
        self.group.hide();
    }
    
    /// Show the component
    pub fn show(&mut self) {
        self.group.show();
    }
}