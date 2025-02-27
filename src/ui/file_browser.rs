// src/ui/file_browser.rs - File browser panel
pub mod file_browser {
    use fltk::{
        browser::FileBrowser,
        button::Button,
        enums::{FrameType},
        group::Group,
        input::Input,
        prelude::*,
        app,
        dialog, // Added for message dialogs
    };
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    
    use crate::transfer::method::TransferMethod;
    use crate::transfer::method::TransferMethodFactory;
    use crate::transfer::method::TransferError;
    
    // A struct to represent a file entry in a directory
    #[derive(Clone, Debug)]
    pub struct FileEntry {
        pub name: String,
        pub path: PathBuf,
        pub is_dir: bool,
        pub size: u64,
    }
    
    // Create a struct to hold state that needs to be shared between callbacks
    struct SharedState {
        is_remote: bool,
        current_dir: PathBuf,
        entries: Vec<FileEntry>,
        transfer_method: Option<Box<dyn TransferMethod>>,
    }
    
    pub struct FileBrowserPanel {
        group: Group,
        browser: FileBrowser,
        path_input: Input,
        refresh_button: Button,
        // Move state to a shared Arc<Mutex>
        shared_state: Arc<Mutex<SharedState>>,
        callback: Option<Box<dyn FnMut(PathBuf, bool) + Send + Sync>>,
        // Connection credentials
        pub current_hostname: Option<String>,
        pub current_username: Option<String>,
        pub current_password: Option<String>,
    }
    
    impl Clone for FileBrowserPanel {
        fn clone(&self) -> Self {
            // Create clone that shares the same state
            let clone = Self {
                group: self.group.clone(),
                browser: self.browser.clone(),
                path_input: self.path_input.clone(),
                refresh_button: self.refresh_button.clone(),
                shared_state: self.shared_state.clone(), // Share the same state
                callback: None, // Cannot clone the callback
                current_hostname: self.current_hostname.clone(),
                current_username: self.current_username.clone(),
                current_password: self.current_password.clone(),
            };
            
            println!("FileBrowserPanel cloned with shared state");
            clone
        }
    }
    
    impl FileBrowserPanel {
        pub fn new(x: i32, y: i32, w: i32, h: i32, title: &str) -> Self {
            let mut group = Group::new(x, y, w, h, None);
            group.set_frame(FrameType::EngravedBox);
            
            // Create panel title
            let mut title_frame = fltk::frame::Frame::new(
                x + 10, 
                y + 10, 
                w - 20, 
                25, 
                title
            );
            title_frame.set_label_size(14);
            title_frame.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);
            
            // Create path input
            let mut path_input = Input::new(
                x + 10, 
                y + 40, 
                w - 110, 
                25, 
                None
            );
            path_input.set_readonly(true);
            
            // Refresh button
            let refresh_button = Button::new(
                x + w - 90, 
                y + 40, 
                80, 
                25, 
                "Refresh"
            );
            
            // File browser
            let mut browser = FileBrowser::new(
                x + 10, 
                y + 75, 
                w - 20, 
                h - 85, 
                None
            );
            browser.set_type(fltk::browser::BrowserType::Hold);
            browser.set_frame(FrameType::EngravedBox);
            browser.set_text_size(12);
            
            group.end();
            
            // Create shared state
            let shared_state = Arc::new(Mutex::new(SharedState {
                is_remote: false,
                current_dir: PathBuf::new(),
                entries: Vec::new(),
                transfer_method: None,
            }));
            
            let mut panel = FileBrowserPanel {
                group,
                browser,
                path_input,
                refresh_button,
                shared_state,
                callback: None,
                current_hostname: None,
                current_username: None,
                current_password: None,
            };
            
            panel.setup_callbacks();
            
            panel
        }
        
        fn setup_callbacks(&mut self) {
            let mut browser_clone = self.browser.clone();
            let path_input_clone = self.path_input.clone();
            let callback_data = Arc::new(Mutex::new(None::<Box<dyn FnMut(PathBuf, bool) + Send + Sync>>));
            
            // Shared state for callback closures
            let shared_state_refresh = self.shared_state.clone();
            
            let mut refresh_button = self.refresh_button.clone();
            refresh_button.set_callback(move |_| {
                // Lock the state and make a copy of what we need
                let current_dir;
                let is_remote;
                let has_transfer_method;
                let transfer_method_name;
                
                {
                    let state = shared_state_refresh.lock().unwrap();
                    is_remote = state.is_remote;
                    current_dir = state.current_dir.clone();
                    has_transfer_method = state.transfer_method.is_some();
                    transfer_method_name = state.transfer_method.as_ref().map(|m| m.get_name().to_string());
                }
                
                println!("Refresh callback with is_remote = {}", is_remote);
                
                // Clear browser
                browser_clone.clear();
                
                // Add parent directory option if not at root
                if current_dir != PathBuf::from("/") && !current_dir.as_os_str().is_empty() {
                    browser_clone.add("..");
                }
                
                if is_remote {
                    // Remote directory refresh
                    println!("Refreshing remote directory: {}", current_dir.display());
                    
                    if has_transfer_method {
                        let method_name = transfer_method_name.unwrap_or_else(|| "Unknown".to_string());
                        println!("Using transfer method: {}", method_name);
                        
                        // Lock the state to get the transfer method and list files
                        let entries = {
                            let state = shared_state_refresh.lock().unwrap();
                            if let Some(ref method) = state.transfer_method {
                                match method.list_files(&current_dir) {
                                    Ok(entries) => Some(entries),
                                    Err(e) => {
                                        println!("Error listing remote directory: {}", e);
                                        browser_clone.add(&format!("Error: {}", e));
                                        None
                                    }
                                }
                            } else {
                                println!("No transfer method available");
                                browser_clone.add("(No connection to remote server)");
                                None
                            }
                        };
                        
                        // Process entries outside the lock
                        if let Some(entries) = entries {
                            let mut entries_vec = Vec::new();
                                
                            for (name, is_dir) in entries {
                                // Add entry to browser - prefix directories with a dot
                                let display_name = if is_dir {
                                    format!(".{}", name)
                                } else {
                                    name.clone()
                                };
                                
                                browser_clone.add(&display_name);
                                
                                // Store the entry in the entries vector
                                entries_vec.push(FileEntry {
                                    name: name.clone(),
                                    path: current_dir.join(&name),
                                    is_dir,
                                    size: 0, // Size information isn't available from list_files
                                });
                            }
                            
                            // Get the length before moving entries_vec
                            let entries_len = entries_vec.len();
                            
                            // Update entries in shared state
                            let mut state = shared_state_refresh.lock().unwrap();
                            state.entries = entries_vec;
                            
                            println!("Listed {} items in remote directory", entries_len);
                        }
                    } else {
                        println!("No transfer method available for remote directory");
                        browser_clone.add("(No connection to remote server)");
                    }
                } else {
                    // Local directory refresh
                    if let Ok(entries) = std::fs::read_dir(&current_dir) {
                        let mut entries_vec = Vec::new();
                        
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                let is_dir = path.is_dir();
                                let name = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("[invalid]");
                                    
                                // Add to browser
                                browser_clone.add(&format!("{}{}", 
                                    if is_dir { "." } else { "" },
                                    name
                                ));
                                
                                // Add to entries vector
                                entries_vec.push(FileEntry {
                                    name: name.to_string(),
                                    path: path.clone(),
                                    is_dir,
                                    size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                                });
                            }
                        }
                        
                        // Get the length before moving entries_vec
                        let entries_len = entries_vec.len();
                        
                        // Update entries in shared state
                        let mut state = shared_state_refresh.lock().unwrap();
                        state.entries = entries_vec;
                        
                        println!("Listed {} items in local directory: {}", 
                            entries_len, current_dir.display());
                    } else {
                        println!("Error reading local directory: {}", current_dir.display());
                    }
                }
                
                // Force the UI to update after making changes
                app::flush();
                app::awake();
                app::redraw();
            });
            
            // Browser selection callback
            let mut browser = self.browser.clone();
            let shared_state_browser = self.shared_state.clone();
            let callback_data_clone = callback_data.clone();
            let mut path_input_clone = path_input_clone.clone();
            let mut refresh_button = refresh_button.clone();
            
            browser.set_callback(move |b| {
                let line = b.value();
                if line == 0 {
                    return;
                }
                
                let text = b.text(line).unwrap_or_default();
                
                // Lock state and make copies of what we need
                let is_remote;
                let current_dir;
                
                {
                    let state = shared_state_browser.lock().unwrap();
                    is_remote = state.is_remote;
                    current_dir = state.current_dir.clone();
                }
                
                println!("Browser callback with is_remote = {}", is_remote);
                
                if text == ".." {
                    // Go to parent directory
                    if let Some(parent) = current_dir.parent() {
                        // Update shared state
                        {
                            let mut state = shared_state_browser.lock().unwrap();
                            state.current_dir = parent.to_path_buf();
                        }
                        
                        // Update path input
                        path_input_clone.set_value(&parent.to_string_lossy());
                        
                        println!("Navigating to parent directory: {}", parent.display());
                        refresh_button.do_callback(); // Use the refresh to load the directory
                    }
                } else {
                    // Check if it's a directory (prefixed with ".")
                    let is_dir = text.starts_with(".");
                    let name = if is_dir { &text[1..] } else { &text };
                    
                    if is_dir {
                        // Navigate to the directory
                        let new_dir = current_dir.join(name);
                        
                        // Update shared state
                        {
                            let mut state = shared_state_browser.lock().unwrap();
                            state.current_dir = new_dir.clone();
                        }
                        
                        // Update path input and refresh
                        path_input_clone.set_value(&new_dir.to_string_lossy());
                        println!("Navigating to directory: {}", new_dir.display());
                        refresh_button.do_callback(); // Use the refresh to load the directory
                    } else {
                        // File selected - call the callback if set
                        let file_path = current_dir.join(name);
                        
                        if let Ok(mut callback_guard) = callback_data_clone.lock() {
                            if let Some(ref mut callback) = *callback_guard {
                                callback(file_path, false);
                            }
                        }
                    }
                }
            });
            
            // Store callback reference
            self.callback = {
                let mut callback_guard = callback_data.lock().unwrap();
                std::mem::take(&mut *callback_guard)
            };
        }
        
        // Show debug info in a non-modal way
        pub fn show_debug_info(&self) {
            // Get all the info before creating the dialog
            let status_text;
            
            {
                let state = self.shared_state.lock().unwrap();
                status_text = format!(
                    "Remote mode: {}\nHas transfer: {}\nCurrent dir: {}\nTransfer method: {}",
                    state.is_remote,
                    state.transfer_method.is_some(),
                    state.current_dir.display(),
                    state.transfer_method.as_ref()
                        .map(|m| m.get_name())
                        .unwrap_or("NONE")
                );
            }
            
            // Log the info
            println!("\n***** FILE BROWSER DEBUG INFO *****");
            println!("{}", status_text);
            println!("*****************************\n");
            
            // Show a message box (non-modal)
            dialog::message_title("Browser Status");
            dialog::message(300, 200, &status_text);
            
            // Make sure UI is updated
            app::flush();
            app::redraw();
        }
        
        // Method for navigating remote directories
        pub fn set_current_remote_directory(&mut self, dir: &PathBuf) {
            println!("Changing remote directory to: {}", dir.display());
            
            // Check if remote mode is set and transfer method exists
            let has_transfer_method;
            
            {
                let mut state = self.shared_state.lock().unwrap();
                
                if !state.is_remote {
                    println!("WARNING: set_current_remote_directory called while not in remote mode!");
                    // Force remote mode
                    state.is_remote = true;
                }
                
                has_transfer_method = state.transfer_method.is_some();
                
                // Set new directory
                state.current_dir = dir.clone();
            }
            
            if !has_transfer_method {
                println!("ERROR: No transfer method available for remote directory change!");
                self.browser.clear();
                self.browser.add("ERROR: No remote connection available");
                return;
            }
            
            self.path_input.set_value(&dir.to_string_lossy());
            
            // Refresh to load the directory contents
            self.refresh();
        }
        
        // Debug method
        pub fn print_debug_status(&self) {
            let state = self.shared_state.lock().unwrap();
            
            println!("\n***** FILE BROWSER DEBUG INFO *****");
            println!("is_remote: {}", state.is_remote);
            println!("has_transfer_method: {}", state.transfer_method.is_some());
            println!("current_dir: {}", state.current_dir.display());
            
            if let Some(ref method) = state.transfer_method {
                println!("transfer_method: {}", method.get_name());
            } else {
                println!("transfer_method: NONE");
            }
            println!("*****************************\n");
        }
        
        // Accessor for remote status
        pub fn is_remote(&self) -> bool {
            self.shared_state.lock().unwrap().is_remote
        }
        
        // Check for transfer method
        pub fn has_transfer_method(&self) -> bool {
            self.shared_state.lock().unwrap().transfer_method.is_some()
        }
        
        // Method to store password
        pub fn store_password(&mut self, password: &str) {
            let mut state = self.shared_state.lock().unwrap();
            
            if let Some(ref mut method) = state.transfer_method {
                method.set_password(password);
                println!("Stored password for SSH connection");
            }
        }
        
        // Set directory for local browsing
        pub fn set_directory(&mut self, dir: &PathBuf) {
            {
                let mut state = self.shared_state.lock().unwrap();
                state.current_dir = dir.clone();
                state.is_remote = false;
                state.transfer_method = None;
            }
            
            self.path_input.set_value(&dir.to_string_lossy());
            self.refresh();
        }
        
        // Set directory for remote browsing
        pub fn set_remote_directory(&mut self, dir: &PathBuf, transfer_method: Box<dyn TransferMethod>) {
            println!("\n***** SETTING REMOTE DIRECTORY *****");
            println!("Path: {}", dir.display());
            println!("Transfer method: {}", transfer_method.get_name());
            
            // Update shared state
            {
                let mut state = self.shared_state.lock().unwrap();
                state.current_dir = dir.clone();
                state.is_remote = true;
                state.transfer_method = Some(transfer_method);
            }
            
            self.path_input.set_value(&dir.to_string_lossy());
            
            println!("***** REFRESHING REMOTE DIRECTORY *****\n");
            self.refresh();
        }
        
        // Clear the browser
        pub fn clear(&mut self) {
            self.browser.clear();
            
            {
                let mut state = self.shared_state.lock().unwrap();
                state.current_dir = PathBuf::new();
                state.entries.clear();
            }
            
            self.path_input.set_value("");
        }
        
        // Refresh the browser
        pub fn refresh(&mut self) {
            // Get the shared state for logging
            {
                let state = self.shared_state.lock().unwrap();
                println!("In refresh() - is_remote = {}", state.is_remote);
            }
            
            // Use refresh button to trigger the actual refresh
            self.refresh_button.do_callback();
        }
        
        // Force remote mode
        pub fn force_remote_mode(&mut self) {
            println!("\n***** FORCING REMOTE MODE *****");
            
            let needs_transfer;
            
            {
                let mut state = self.shared_state.lock().unwrap();
                needs_transfer = state.transfer_method.is_none() && 
                                self.current_hostname.is_some() && 
                                self.current_username.is_some();
                
                // Set remote flag
                state.is_remote = true;
                println!("Set shared state remote = true");
            }
            
            // Check if we need to recreate the transfer method
            if needs_transfer {
                println!("Attempting to recreate SSH connection with stored credentials");
                
                let hostname = self.current_hostname.clone().unwrap_or("raspberrypi.local".to_string());
                let username = self.current_username.clone().unwrap_or("pi".to_string());
                let port = 22; // Default port
                
                // Create a new SSH connection
                use crate::transfer::ssh::SSHTransferFactory;
                
                let factory = SSHTransferFactory::new(
                    hostname.clone(),
                    username.clone(),
                    port,
                    false, // Use password auth
                    None,  // No key path
                );
                
                // Create new transfer method
                let mut transfer_method = factory.create_method();
                
                // Apply password if we have one
                if let Some(ref password) = self.current_password {
                    transfer_method.set_password(password);
                    println!("Applied stored password to new connection");
                }
                
                // Update shared state with the new transfer method
                {
                    let mut state = self.shared_state.lock().unwrap();
                    state.transfer_method = Some(transfer_method);
                    println!("Created new transfer method");
                }
            }
            
            // Refresh the browser
            self.refresh();
            
            // Force FLTK to update the UI promptly
            app::flush();
        }
        
        // Set callback
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: FnMut(PathBuf, bool) + 'static + Send + Sync,
        {
            self.callback = Some(Box::new(callback));
        }
        
        // NEW METHOD: Download a file from remote to a local path
        pub fn download_remote_file(&self, remote_path: &Path, local_path: &Path) -> Result<(), String> {
            let state = self.shared_state.lock().unwrap();
            
            if !state.is_remote {
                return Err("Not in remote mode".to_string());
            }
            
            if let Some(ref method) = state.transfer_method {
                match method.download_file(remote_path, local_path) {
                    Ok(_) => {
                        println!("Downloaded: {} -> {}", remote_path.display(), local_path.display());
                        Ok(())
                    },
                    Err(e) => Err(format!("Download failed: {}", e))
                }
            } else {
                Err("No transfer method available".to_string())
            }
        }
        
        // Helper to check if a file is an image based on extension
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
        
        // Get the current directory
        pub fn get_current_directory(&self) -> PathBuf {
            let state = self.shared_state.lock().unwrap();
            state.current_dir.clone()
        }
    }
}