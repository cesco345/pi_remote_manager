// src/ui/file_browser.rs - File browser panel
pub mod file_browser {
    use fltk::{
        app,
        browser::HoldBrowser,
        button::Button,
        enums::{Color, FrameType, Shortcut},
        group::Group,
        input::Input,
        prelude::*,
    };
    
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::sync::{Arc, Mutex};
    
    pub struct FileBrowserPanel {
        group: Group,
        path_input: Input,
        file_browser: HoldBrowser,
        refresh_button: Button,
        current_dir: Arc<Mutex<PathBuf>>,
        callback: Option<Box<dyn Fn(PathBuf, bool) + Send + Sync>>,
    }
    
    impl Clone for FileBrowserPanel {
        fn clone(&self) -> Self {
            Self {
                group: self.group.clone(),
                path_input: self.path_input.clone(),
                file_browser: self.file_browser.clone(),
                refresh_button: self.refresh_button.clone(),
                current_dir: self.current_dir.clone(),
                callback: None, // Cannot clone the callback
            }
        }
    }
    
    impl FileBrowserPanel {
        pub fn new(x: i32, y: i32, w: i32, h: i32, title: &str) -> Self {
            let mut group = Group::new(x, y, w, h, None);
            group.set_frame(FrameType::BorderBox);
            
            // Add title
            let padding = 5;
            let row_height = 25;
            let title_h = 20;
            
            let mut title_box = fltk::frame::Frame::new(
                x + padding, 
                y + padding, 
                w - 2 * padding, 
                title_h, 
                Some(title)
            );
            title_box.set_label_size(14);
            title_box.set_align(fltk::enums::Align::Center);
            
            // Path input and refresh button
            let input_y = y + padding + title_h + padding;
            let mut path_input = Input::new(
                x + padding, 
                input_y, 
                w - 70 - 2 * padding, 
                row_height, 
                None
            );
            
            let mut refresh_button = Button::new(
                x + w - 70 - padding, 
                input_y, 
                70, 
                row_height, 
                "Refresh"
            );
            
            // File browser
            let browser_y = input_y + row_height + padding;
            let browser_h = h - (browser_y - y) - padding;
            
            let mut file_browser = HoldBrowser::new(
                x + padding, 
                browser_y, 
                w - 2 * padding, 
                browser_h, 
                None
            );
            
            group.end();
            
            // Set initial directory to current directory
            let current_dir = Arc::new(Mutex::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))));
            
            let mut panel = FileBrowserPanel {
                group,
                path_input,
                file_browser,
                refresh_button,
                current_dir,
                callback: None,
            };
            
            panel.setup_callbacks();
            
            panel
        }
        
        pub fn setup_callbacks(&mut self) {
            // Refresh button callback
            let current_dir = self.current_dir.clone();
            let mut file_browser = self.file_browser.clone();
            let mut path_input = self.path_input.clone();
            
            let mut refresh_button = self.refresh_button.clone();
            refresh_button.set_callback(move |_| {
                let dir = {
                    let dir = current_dir.lock().unwrap();
                    dir.clone()
                };
                
                Self::refresh_file_list(&dir, &mut file_browser);
                path_input.set_value(&dir.to_string_lossy());
            });
            
            // File browser double-click callback
            let current_dir = self.current_dir.clone();
            let callback_ref = Arc::new(Mutex::new(None::<Box<dyn Fn(PathBuf, bool) + Send + Sync>>));
            let callback_clone = callback_ref.clone();
            
            let mut file_browser = self.file_browser.clone();
            let mut path_input = self.path_input.clone();
            
            file_browser.set_callback(move |b| {
                let line = b.value();
                if line <= 0 {
                    return;
                }
                
                let text = b.text(line).unwrap_or_default();
                let is_dir = text.starts_with("[") && text.ends_with("]");
                let name = if is_dir {
                    text.trim_start_matches("[").trim_end_matches("]")
                } else {
                    &text
                };
                
                let mut dir = {
                    let dir = current_dir.lock().unwrap();
                    dir.clone()
                };
                
                if is_dir {
                    if name == ".." {
                        if let Some(parent) = dir.parent() {
                            dir = parent.to_path_buf();
                        }
                    } else {
                        dir.push(name);
                    }
                    
                    // Update current directory
                    {
                        let mut current = current_dir.lock().unwrap();
                        *current = dir.clone();
                    }
                    
                    // Refresh file list
                    Self::refresh_file_list(&dir, b);
                    path_input.set_value(&dir.to_string_lossy());
                } else {
                    // File selected, call the callback if set
                    let mut file_path = dir.clone();
                    file_path.push(name);
                    
                    if let Some(ref callback) = *callback_clone.lock().unwrap() {
                        callback(file_path, false);
                    }
                }
            });
            
            // Store callback reference for later use
            self.callback = {
                let mut callback_guard = callback_ref.lock().unwrap();
                std::mem::take(&mut *callback_guard)
            };
        }
        
        pub fn set_directory(&mut self, dir: &Path) {
            if dir.exists() && dir.is_dir() {
                let mut current = self.current_dir.lock().unwrap();
                *current = dir.to_path_buf();
                
                self.path_input.set_value(&dir.to_string_lossy());
                Self::refresh_file_list(dir, &mut self.file_browser);
            }
        }
        
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: Fn(PathBuf, bool) + 'static + Send + Sync,
        {
            self.callback = Some(Box::new(callback));
        }
        
        pub fn refresh(&mut self) {
            let dir = {
                let dir = self.current_dir.lock().unwrap();
                dir.clone()
            };
            
            Self::refresh_file_list(&dir, &mut self.file_browser);
        }
        
        fn refresh_file_list(dir: &Path, browser: &mut HoldBrowser) {
            browser.clear();
            
            if let Some(parent) = dir.parent() {
                // Add parent directory
                browser.add("[..]");
            }
            
            // List directories and files
            if let Ok(entries) = fs::read_dir(dir) {
                let mut dirs = Vec::new();
                let mut files = Vec::new();
                
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    if path.is_dir() {
                        dirs.push(format!("[{}]", name));
                    } else {
                        files.push(name);
                    }
                }
                
                // Sort alphabetically
                dirs.sort();
                files.sort();
                
                // Add to browser (directories first, then files)
                for dir in dirs {
                    browser.add(&dir);
                }
                
                for file in files {
                    browser.add(&file);
                }
            }
        }
    }
}