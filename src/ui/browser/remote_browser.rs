use fltk::{prelude::*, app};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::env;
use std::fs;

use crate::transfer::TransferMethod;
use crate::ui::file_browser::file_browser::FileBrowserPanel;
use crate::core::file::get_file_type_info;


/// An extension of FileBrowserPanel with enhanced remote preview capabilities
pub struct RemoteBrowserPanel {
    /// The base file browser panel
    pub browser: FileBrowserPanel,
    /// Temporary directory for downloaded previews
    temp_dir: Arc<Mutex<PathBuf>>,
    /// Callback for file previews
    preview_callback: Option<Box<dyn FnMut(PathBuf, bool) + Send + Sync>>,
}

impl Clone for RemoteBrowserPanel {
    fn clone(&self) -> Self {
        RemoteBrowserPanel {
            browser: self.browser.clone(),
            temp_dir: self.temp_dir.clone(),
            preview_callback: None, // Callbacks cannot be cloned
        }
    }
}

impl RemoteBrowserPanel {
    /// Create a new remote browser panel
    pub fn new(x: i32, y: i32, w: i32, h: i32, title: &str) -> Self {
        // Create base browser panel
        let browser = FileBrowserPanel::new(x, y, w, h, title);
        
        // Create a temp dir for remote file previews
        let mut temp_dir = env::temp_dir();
        temp_dir.push("pi_image_processor_preview");
        
        // Create the temp directory if it doesn't exist
        if !temp_dir.exists() {
            let _ = fs::create_dir_all(&temp_dir);
        }
        
        RemoteBrowserPanel {
            browser,
            temp_dir: Arc::new(Mutex::new(temp_dir)),
            preview_callback: None,
        }
    }
    
    /// Set a callback for file selection
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnMut(PathBuf, bool) + 'static + Send + Sync,
    {
        self.preview_callback = Some(Box::new(callback));
        
        // Create a local wrapper around the preview callback
        let preview_callback = Arc::new(Mutex::new(self.preview_callback.take()));
        let temp_dir = self.temp_dir.clone();
        
        // Standard file select callback - will handle downloading files
        self.browser.set_callback(move |path, is_dir| {
            if !is_dir {
                println!("Remote file selected: {}", path.display());
                
                // Check if we need to download for preview
                let path_exists = path.exists();
                let file_info = get_file_type_info(&path);
                
                if file_info.previewable && !path_exists {
                    println!("File needs download for preview: {}", path.display());
                    
                    // Get temporary location
                    let mut temp_file = {
                        let temp_dir = temp_dir.lock().unwrap();
                        temp_dir.clone()
                    };
                    
                    // Create a file name in temp dir
                    if let Some(file_name) = path.file_name() {
                        temp_file.push(file_name);
                        
                        println!("Temporary file location: {}", temp_file.display());
                        
                        // Call the preview callback with the original path
                        // The main window will handle downloading if needed
                        if let Ok(mut callback_guard) = preview_callback.lock() {
                            if let Some(ref mut callback) = *callback_guard {
                                // Pass original path and is_dir flag
                                callback(path, is_dir);
                            }
                        }
                    }
                } else {
                    // File exists locally or isn't previewable, just call the callback
                    if let Ok(mut callback_guard) = preview_callback.lock() {
                        if let Some(ref mut callback) = *callback_guard {
                            callback(path, is_dir);
                        }
                    }
                }
            } else {
                // For directories, just pass through to the callback
                if let Ok(mut callback_guard) = preview_callback.lock() {
                    if let Some(ref mut callback) = *callback_guard {
                        callback(path, is_dir);
                    }
                }
            }
        });
    }
    
 /// Download a remote file for preview
 pub fn download_for_preview(&self, remote_path: &Path) -> Result<PathBuf, String> {
    // Check if we have a transfer method
    if !self.browser.has_transfer_method() {
        return Err("No transfer method available".to_string());
    }
    
    // Get temporary location
    let mut temp_file = {
        let temp_dir = self.temp_dir.lock().unwrap();
        temp_dir.clone()
    };
    
    // Create a file name in temp dir
    if let Some(file_name) = remote_path.file_name() {
        temp_file.push(file_name);
        
        println!("Downloading to: {}", temp_file.display());
        
        // Since we don't have direct access to the transfer method yet,
        // we'll provide a workaround solution
        
        // This function should be replaced with actual implementation
        // once FileBrowserPanel gets a get_transfer_method() function
        println!("Attempting to download: {} -> {}", 
            remote_path.display(), 
            temp_file.display()
        );
        
        // For now, we'll just check if the file already exists locally
        if remote_path.exists() {
            // Copy the file to the temp location
            match fs::copy(remote_path, &temp_file) {
                Ok(_) => {
                    println!("File copied successfully");
                    return Ok(temp_file);
                },
                Err(e) => {
                    return Err(format!("File copy failed: {}", e));
                }
            }
        }
        
        // Return an error for now
        Err("Remote file download not yet implemented".to_string())
    } else {
        Err("Invalid file path".to_string())
    }
}
    
    /// Clean up temporary files
    pub fn cleanup_temp_files(&self) {
        let temp_dir = self.temp_dir.lock().unwrap();
        
        // Delete all files in the temp directory
        if temp_dir.exists() {
            if let Ok(entries) = fs::read_dir(&*temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
    }
    
    /// Set directory path (delegate to base browser)
    pub fn set_directory(&mut self, dir: &PathBuf) {
        self.browser.set_directory(dir);
    }
    
    /// Set remote directory (delegate to base browser)
    pub fn set_remote_directory(&mut self, dir: &PathBuf, transfer_method: Box<dyn TransferMethod>) {
        self.browser.set_remote_directory(dir, transfer_method);
    }
    
    /// Is in remote mode (delegate to base browser)
    pub fn is_remote(&self) -> bool {
        self.browser.is_remote()
    }
    
    /// Has transfer method (delegate to base browser)
    pub fn has_transfer_method(&self) -> bool {
        self.browser.has_transfer_method()
    }
    
    /// Print debug status (delegate to base browser)
    pub fn print_debug_status(&self) {
        self.browser.print_debug_status();
    }
    
    /// Force remote mode (delegate to base browser)
    pub fn force_remote_mode(&mut self) {
        self.browser.force_remote_mode();
    }
    
    /// Refresh (delegate to base browser)
    pub fn refresh(&mut self) {
        self.browser.refresh();
    }
}