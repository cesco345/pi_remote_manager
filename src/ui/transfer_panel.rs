// ui/transfer_panel.rs - File transfer panel
pub mod transfer_panel {
    use fltk::{
        button::Button,
        enums::{Color, FrameType},
        group::Group,
        input::Input,
        prelude::*,
    };
    
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    
    use crate::config::Config;

    // Updated imports to use the new module structure
    use crate::transfer::ssh::SSHTransferFactory;
    use crate::transfer::method::{
        TransferMethod,
        TransferMethodFactory,
    };
    
    use crate::ui::dialogs::dialogs;
    
    pub struct TransferPanel {
        group: Group,
        source_input: Input,
        dest_input: Input,
        transfer_button: Button,
        direction_button: Button,
        source_is_local: bool,
        config: Arc<Mutex<Config>>,
        // Changed from Fn to FnMut
        callback: Option<Box<dyn FnMut(bool, PathBuf, PathBuf) + Send + Sync>>,
    }
    
    impl Clone for TransferPanel {
        fn clone(&self) -> Self {
            Self {
                group: self.group.clone(),
                source_input: self.source_input.clone(),
                dest_input: self.dest_input.clone(),
                transfer_button: self.transfer_button.clone(),
                direction_button: self.direction_button.clone(),
                source_is_local: self.source_is_local,
                config: self.config.clone(),
                callback: None, // Cannot clone the callback
            }
        }
    }
    
    impl TransferPanel {
        pub fn new(
            x: i32, 
            y: i32, 
            w: i32, 
            h: i32,
            config: Arc<Mutex<Config>>
        ) -> Self {
            let mut group = Group::new(x, y, w, h, None);
            group.set_frame(FrameType::EngravedBox);
            
            // Add panel components
            let padding = 10;
            let label_width = 100;
            let button_width = 120;
            let input_width = w - label_width - button_width - 3 * padding;
            let row_height = 25;
            
            // Title
            let mut title = fltk::frame::Frame::new(
                x + w / 2 - 60,
                y + padding,
                120,
                20,
                "File Transfer"
            );
            title.set_label_size(14);
            title.set_align(fltk::enums::Align::Center);
            
            // Source path
            let row1_y = y + padding + 25;
            let mut source_label = fltk::frame::Frame::new(
                x + padding,
                row1_y,
                label_width,
                row_height,
                "Source:"
            );
            source_label.set_align(fltk::enums::Align::Inside | fltk::enums::Align::Left);
            
            let source_input = Input::new(
                x + padding + label_width,
                row1_y,
                input_width,
                row_height,
                None
            );
            
            let direction_button = Button::new(
                x + padding + label_width + input_width + padding,
                row1_y,
                button_width,
                row_height,
                "Local → Remote"
            );
            
            // Destination path
            let row2_y = row1_y + row_height + padding;
            let mut dest_label = fltk::frame::Frame::new(
                x + padding,
                row2_y,
                label_width,
                row_height,
                "Destination:"
            );
            dest_label.set_align(fltk::enums::Align::Inside | fltk::enums::Align::Left);
            
            let dest_input = Input::new(
                x + padding + label_width,
                row2_y,
                input_width,
                row_height,
                None
            );
            
            let mut transfer_button = Button::new(
                x + padding + label_width + input_width + padding,
                row2_y,
                button_width,
                row_height,
                "Transfer"
            );
            transfer_button.set_color(Color::from_rgb(0, 120, 255));
            transfer_button.set_label_color(Color::White);
            
            group.end();
            
            let mut panel = TransferPanel {
                group,
                source_input,
                dest_input,
                transfer_button,
                direction_button,
                source_is_local: true,
                config,
                callback: None,
            };
            
            panel.setup_callbacks();
            
            panel
        }
        
        fn setup_callbacks(&mut self) {
            // Create a shared state for source_is_local
            let source_is_local_state = Arc::new(Mutex::new(self.source_is_local));
            
            // Direction button callback
            let mut direction_button = self.direction_button.clone();
            let source_is_local_clone = source_is_local_state.clone();
            
            direction_button.set_callback(move |b| {
                let mut source_is_local = source_is_local_clone.lock().unwrap();
                *source_is_local = !*source_is_local;
                
                if *source_is_local {
                    b.set_label("Local → Remote");
                } else {
                    b.set_label("Remote → Local");
                }
            });
            
            // Transfer button callback
            let source_input = self.source_input.clone();
            let dest_input = self.dest_input.clone();
            let config = self.config.clone();
            let source_is_local_clone = source_is_local_state.clone();
            
            // Changed from Fn to FnMut
            let callback_ref = Arc::new(Mutex::new(None::<Box<dyn FnMut(bool, PathBuf, PathBuf) + Send + Sync>>));
            let callback_clone = callback_ref.clone();
            
            let mut transfer_button = self.transfer_button.clone();
            transfer_button.set_callback(move |_| {
                let source_path = source_input.value();
                let dest_path = dest_input.value();
                
                if source_path.is_empty() || dest_path.is_empty() {
                    dialogs::message_dialog("Error", "Source and destination paths cannot be empty.");
                    return;
                }
                
                let source = PathBuf::from(&source_path);
                let dest = PathBuf::from(&dest_path);
                
                // Get the current transfer direction from the shared state
                let source_is_local = *source_is_local_clone.lock().unwrap();
                println!("Transfer with source_is_local = {}", source_is_local);
                
                // Get the currently selected host
                let host = {
                    let config_guard = config.lock().unwrap();
                    if config_guard.hosts.is_empty() {
                        dialogs::message_dialog("Error", "No host configured. Please add a host first.");
                        return;
                    }
                    
                    // Use the last selected host
                    let index = config_guard.last_used_host_index.min(config_guard.hosts.len() - 1);
                    config_guard.hosts[index].clone()
                };
                
                // Create a transfer method
                let factory = SSHTransferFactory::new(
                    host.hostname.clone(),
                    host.username.clone(),
                    host.port,
                    host.use_key_auth,
                    host.key_path.clone(),
                );
                
                let mut method = factory.create_method();
                
                // Ask for password if needed
                if !host.use_key_auth {
                    if let Some(password) = dialogs::password_dialog(
                        "SSH Password", 
                        &format!("Enter password for {}@{}", host.username, host.hostname)
                    ) {
                        if let Some(method_mut) = method.as_any().downcast_mut::<crate::transfer::ssh::SSHTransfer>() {
                            method_mut.set_password(password.clone());
                        }
                    } else {
                        // User canceled password dialog
                        return;
                    }
                }
                
                // Perform the transfer 
                println!("Transferring file:");
                println!("  Source: {}", source.display());
                println!("  Destination: {}", dest.display());
                println!("  Direction: {}", if source_is_local { "Local → Remote" } else { "Remote → Local" });
                
                let result = if source_is_local {
                    println!("Uploading local file to remote...");
                    method.upload_file(&source, &dest)
                } else {
                    println!("Downloading remote file to local...");
                    method.download_file(&source, &dest)
                };
                
                match result {
                    Ok(_) => {
                        dialogs::message_dialog("Success", "File transfer completed successfully.");
                        
                        // Call the callback if set
                        if let Ok(mut callback_guard) = callback_clone.lock() {
                            if let Some(ref mut callback) = *callback_guard {
                                callback(source_is_local, source, dest);
                            }
                        }
                    },
                    Err(e) => {
                        dialogs::message_dialog("Error", &format!("File transfer failed: {}", e));
                    }
                }
            });
            
            // Store callback reference for later use
            self.callback = {
                let mut callback_guard = callback_ref.lock().unwrap();
                std::mem::take(&mut *callback_guard)
            };
            
            // Store the reference to the shared state
            self.source_is_local = *source_is_local_state.lock().unwrap();
        }
        
        pub fn set_source_path(&mut self, path: PathBuf, is_local: bool) {
            // Set the source path
            self.source_input.set_value(&path.to_string_lossy());
            
            // Update direction if needed
            if self.source_is_local != is_local {
                self.source_is_local = is_local;
                
                if is_local {
                    self.direction_button.set_label("Local → Remote");
                } else {
                    self.direction_button.set_label("Remote → Local");
                }
            }
            
            // Generate a reasonable destination path
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
                
            let dest_path = if is_local {
                // Local to remote, use remote home directory
                format!("/home/{}/{}", 
                    self.config.lock().unwrap().hosts[0].username,
                    filename
                )
            } else {
                // Remote to local, use local downloads directory
                let local_dir = dirs::download_dir()
                    .unwrap_or_else(|| PathBuf::from("."));
                format!("{}/{}", local_dir.to_string_lossy(), filename)
            };
            
            self.dest_input.set_value(&dest_path);
        }
        
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: FnMut(bool, PathBuf, PathBuf) + 'static + Send + Sync,
        {
            self.callback = Some(Box::new(callback));
        }
    }
}
