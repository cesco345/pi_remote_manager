use fltk::{
    enums::{Color, FrameType, Font, Align},
    group::Group,
    text::{TextDisplay, TextBuffer},
    frame::Frame,
    prelude::*,
};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs;

use crate::core::file::get_text_preview;

/// Maximum file size for text preview (5MB)
const MAX_TEXT_SIZE: u64 = 5 * 1024 * 1024;

/// Component for previewing text files
pub struct TextPreviewComponent {
    /// Container group
    group: Group,
    /// Text display widget
    text_display: TextDisplay,
    /// Text buffer
    text_buffer: TextBuffer,
    /// Error message frame
    error_frame: Frame,
    /// Currently loaded file path
    current_file: Arc<Mutex<Option<PathBuf>>>,
}

impl Clone for TextPreviewComponent {
    fn clone(&self) -> Self {
        // Create a new text buffer when cloning
        let text_buffer = TextBuffer::default();
        
        // We need to update the text display with the new buffer
        let mut text_display = self.text_display.clone();
        text_display.set_buffer(text_buffer.clone());
        
        Self {
            group: self.group.clone(),
            text_display,
            text_buffer,
            error_frame: self.error_frame.clone(),
            current_file: self.current_file.clone(),
        }
    }
}

impl TextPreviewComponent {
    /// Create a new text preview component
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut group = Group::new(x, y, w, h, None);
        group.set_frame(FrameType::FlatBox);
        
        // Add text display area
        let padding = 5;
        let display_x = x + padding;
        let display_y = y + padding;
        let display_w = w - 2 * padding;
        let display_h = h - 2 * padding;
        
        // Create text buffer and display
        let text_buffer = TextBuffer::default();
        
        let mut text_display = TextDisplay::new(
            display_x,
            display_y,
            display_w,
            display_h,
            None
        );
        text_display.set_buffer(text_buffer.clone());
        text_display.set_frame(FrameType::BorderFrame);
        text_display.set_color(Color::from_rgb(250, 250, 250));
        text_display.set_text_font(Font::Courier);
        text_display.set_text_size(12);
        text_display.wrap_mode(true, 0); // Enable word wrap
        
        // Add error message frame (initially hidden)
        let mut error_frame = Frame::new(
            display_x,
            display_y,
            display_w,
            display_h,
            None
        );
        error_frame.set_frame(FrameType::BorderFrame);
        error_frame.set_color(Color::from_rgb(250, 240, 240));
        error_frame.set_label_size(12);
        error_frame.set_align(Align::Center | Align::Inside);
        error_frame.hide();
        
        group.end();
        
        TextPreviewComponent {
            group,
            text_display,
            text_buffer,
            error_frame,
            current_file: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Load and display a text file
    pub fn load_text(&mut self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        
        // Clear any previous content
        self.clear();
        
        // Check file size
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() > MAX_TEXT_SIZE {
                    self.show_error(&format!(
                        "File too large to preview ({} bytes)\nMaximum size: {} bytes",
                        metadata.len(),
                        MAX_TEXT_SIZE
                    ));
                    return false;
                }
            },
            Err(e) => {
                self.show_error(&format!("Error accessing file: {}", e));
                return false;
            }
        }
        
        // Try to read the file
        match get_text_preview(path) {
            Ok(content) => {
                // Set the content to the text buffer
                self.text_buffer.set_text(&content);
                
                // Show the text display, hide the error frame
                self.text_display.show();
                self.error_frame.hide();
                
                // Store the current file path
                let mut current = self.current_file.lock().unwrap();
                *current = Some(path.to_path_buf());
                
                // Scroll to the top
                self.text_display.scroll(0, 0);
                
                true
            },
            Err(e) => {
                self.show_error(&format!("Error reading file: {}", e));
                false
            }
        }
    }
    
    /// Display an error message
    fn show_error(&mut self, message: &str) {
        // Hide text display, show error frame
        self.text_display.hide();
        self.error_frame.set_label(message);
        self.error_frame.show();
        
        // Force redraw
        self.group.redraw();
    }
    
    /// Get the current file path
    pub fn get_current_file(&self) -> Option<PathBuf> {
        let current = self.current_file.lock().unwrap();
        current.clone()
    }
    
    /// Clear the text display
    pub fn clear(&mut self) {
        // Clear the text buffer
        self.text_buffer.set_text("");
        
        // Hide error frame, show text display
        self.error_frame.hide();
        self.text_display.show();
        
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