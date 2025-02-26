// src/ui/dialogs.rs
pub mod dialogs {
    use std::sync::{Arc, Mutex};
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::cell::RefCell;
    use fltk::{
        app,
        button::Button,
        dialog::{FileDialog, FileDialogType},
        enums::{Align, Color},
        frame::Frame,
        input::Input,
        menu::Choice,
        prelude::*,
        window::Window,
    };
    use crate::config::{Config, Host};

    pub fn open_file_dialog(title: &str, filter: &str) -> Option<PathBuf> {
        let mut dialog = FileDialog::new(FileDialogType::BrowseFile);
        dialog.set_title(title);
        
        if !filter.is_empty() {
            dialog.set_filter(filter);
        }
        
        dialog.show();
        
        let filename = dialog.filename();
        if filename.to_string_lossy().is_empty() {
            None
        } else {
            Some(filename)
        }
    }

    pub fn save_file_dialog(title: &str, filter: &str) -> Option<PathBuf> {
        let mut dialog = FileDialog::new(FileDialogType::BrowseSaveFile);
        dialog.set_title(title);
        
        if !filter.is_empty() {
            dialog.set_filter(filter);
        }
        
        dialog.show();
        
        let filename = dialog.filename();
        if filename.to_string_lossy().is_empty() {
            None
        } else {
            Some(filename)
        }
    }

    pub fn message_dialog(title: &str, message: &str) {
        choice_dialog(title, message, &["OK"]);
    }
    // Add this to src/ui/dialogs.rs
// This creates a password dialog for SSH connections

pub fn password_dialog(title: &str, prompt: &str) -> Option<String> {
    use fltk::{
        app,
        button::Button,
        enums::{Align, Color},
        frame::Frame,
        input::SecretInput,
        window::Window,
        prelude::*,
    };
    
    let mut dialog = Window::new(100, 100, 300, 150, title);
    dialog.set_border(true);
    
    let padding = 10;
    let input_height = 25;
    let button_width = 80;
    
    // Prompt message
    let mut message_frame = Frame::new(
        padding, 
        padding, 
        300 - padding * 2, 
        30,
        prompt
    );
    message_frame.set_align(Align::Left | Align::Inside | Align::Top);
    
    // Password input field
    let mut password_input = SecretInput::new(
        padding,
        padding + 35,
        300 - padding * 2,
        input_height,
        ""
    );
    
    // Buttons
    let mut cancel_button = Button::new(
        padding,
        150 - padding - input_height,
        button_width,
        input_height,
        "Cancel"
    );
    
    let mut ok_button = Button::new(
        300 - padding - button_width,
        150 - padding - input_height,
        button_width,
        input_height,
        "OK"
    );
    ok_button.set_color(Color::from_rgb(0, 120, 255));
    ok_button.set_label_color(Color::White);
    
    // Password result
    let password_result = std::rc::Rc::new(std::cell::RefCell::new(None::<String>));
    let password_result_clone = password_result.clone();
    
    // Cancel button callback
    cancel_button.set_callback(move |_| {
        if let Some(mut win) = app::first_window() {
            win.hide();
        }
    });
    
    // OK button callback
    let password_input_clone = password_input.clone();
    ok_button.set_callback(move |_| {
        let password = password_input_clone.value();
        if !password.is_empty() {
            *password_result_clone.borrow_mut() = Some(password);
        }
        
        if let Some(mut win) = app::first_window() {
            win.hide();
        }
    });
    
    // Set focus to password input and handle Enter key
    password_input.take_focus().ok();
    password_input.set_trigger(fltk::enums::CallbackTrigger::EnterKey);
    let password_clone = password_result.clone();
    password_input.set_callback(move |i| {
        let password = i.value();
        if !password.is_empty() {
            *password_clone.borrow_mut() = Some(password);
            
            if let Some(mut win) = app::first_window() {
                win.hide();
            }
        }
    });
    
    dialog.end();
    dialog.show();
    
    while dialog.shown() {
        app::wait();
    }
    
    // Get the final result
    let result = password_result.borrow().clone();
    result
}

    pub fn connection_dialog(config: Arc<Mutex<Config>>) -> Option<Host> {
        // Get available hosts
        let hosts = {
            let config = config.lock().unwrap();
            config.hosts.clone()
        };
        
        // Create a custom dialog window
        let mut dialog = Window::new(100, 100, 400, 400, "Connection Settings");
        dialog.set_border(true);
        
        let padding = 10;
        let input_height = 25;
        let label_width = 120;
        let input_width = 400 - label_width - padding * 3;
        
        // Host selection or create new
        let mut host_choice = Choice::new(
            padding + label_width, 
            padding, 
            input_width, 
            input_height,
            "Select Host:"
        );
        host_choice.set_align(Align::Left);
        
        // Add existing hosts
        for (i, host) in hosts.iter().enumerate() {
            host_choice.add_choice(&format!("{} ({}@{}:{}) [{}]", 
                host.name, 
                host.username, 
                host.hostname, 
                host.port, 
                if host.use_key_auth { "Key" } else { "Password" }
            ));
            if i == 0 {
                host_choice.set_value(0);
            }
        }
        
        // Add "Create New Host" option
        host_choice.add_choice("Create New Host...");
        
        // Name input
        let mut name_label = Frame::new(
            padding, 
            padding * 2 + input_height, 
            label_width, 
            input_height,
            "Name:"
        );
        name_label.set_align(Align::Left | Align::Inside);
        
        let mut name_input = Input::new(
            padding + label_width, 
            padding * 2 + input_height, 
            input_width, 
            input_height,
            ""
        );
        
        // Hostname input
        let mut hostname_label = Frame::new(
            padding, 
            padding * 3 + input_height * 2, 
            label_width, 
            input_height,
            "Hostname/IP:"
        );
        hostname_label.set_align(Align::Left | Align::Inside);
        
        let mut hostname_input = Input::new(
            padding + label_width, 
            padding * 3 + input_height * 2, 
            input_width, 
            input_height,
            ""
        );
        
        // Username input
        let mut username_label = Frame::new(
            padding, 
            padding * 4 + input_height * 3, 
            label_width, 
            input_height,
            "Username:"
        );
        username_label.set_align(Align::Left | Align::Inside);
        
        let mut username_input = Input::new(
            padding + label_width, 
            padding * 4 + input_height * 3, 
            input_width, 
            input_height,
            ""
        );
        
        // Port input
        let mut port_label = Frame::new(
            padding, 
            padding * 5 + input_height * 4, 
            label_width, 
            input_height,
            "Port:"
        );
        port_label.set_align(Align::Left | Align::Inside);
        
        let mut port_input = Input::new(
            padding + label_width, 
            padding * 5 + input_height * 4, 
            input_width, 
            input_height,
            "22"
        );
        
        // Authentication method
        let mut auth_label = Frame::new(
            padding, 
            padding * 6 + input_height * 5, 
            label_width, 
            input_height,
            "Authentication:"
        );
        auth_label.set_align(Align::Left | Align::Inside);
        
        let mut auth_choice = Choice::new(
            padding + label_width, 
            padding * 6 + input_height * 5, 
            input_width, 
            input_height,
            ""
        );
        auth_choice.add_choice("Password");
        auth_choice.add_choice("SSH Key");
        auth_choice.set_value(0);
        
        // Key file selection (initially hidden)
        let mut key_label = Frame::new(
            padding, 
            padding * 7 + input_height * 6, 
            label_width, 
            input_height,
            "Key File:"
        );
        key_label.set_align(Align::Left | Align::Inside);
        key_label.hide();
        
        let mut key_input = Input::new(
            padding + label_width, 
            padding * 7 + input_height * 6, 
            input_width - 80, 
            input_height,
            ""
        );
        key_input.hide();
        
        let mut browse_button = Button::new(
            padding + label_width + input_width - 70, 
            padding * 7 + input_height * 6, 
            70, 
            input_height,
            "Browse..."
        );
        browse_button.hide();
        
        // Connection test button
        let mut test_button = Button::new(
            padding, 
            400 - padding * 2 - input_height * 2, 
            120, 
            input_height,
            "Test Connection"
        );
        test_button.set_color(Color::from_rgb(0, 180, 0));
        test_button.set_label_color(Color::White);
        
        // Buttons
        let mut cancel_button = Button::new(
            padding, 
            400 - padding - input_height, 
            100, 
            input_height,
            "Cancel"
        );
        
        let mut save_button = Button::new(
            400 - padding - 100, 
            400 - padding - input_height, 
            100, 
            input_height,
            "Save"
        );
        save_button.set_color(Color::from_rgb(0, 120, 255));
        save_button.set_label_color(Color::White);
        
        // Delete button (for existing hosts)
        let mut delete_button = Button::new(
            padding + 110, 
            400 - padding - input_height, 
            100, 
            input_height,
            "Delete"
        );
        delete_button.set_color(Color::from_rgb(220, 0, 0));
        delete_button.set_label_color(Color::White);
        
        // Status message
        let mut status_frame = Frame::new(
            padding, 
            400 - padding * 3 - input_height * 3, 
            400 - padding * 2, 
            input_height,
            ""
        );
        status_frame.set_align(Align::Left | Align::Inside);
        
        // Initial state
        if !hosts.is_empty() {
            let host = &hosts[0];
            name_input.set_value(&host.name);
            hostname_input.set_value(&host.hostname);
            username_input.set_value(&host.username);
            port_input.set_value(&host.port.to_string());
            
            if host.use_key_auth {
                auth_choice.set_value(1); // SSH Key
                if let Some(path) = &host.key_path {
                    key_input.set_value(path);
                }
                key_label.show();
                key_input.show();
                browse_button.show();
            }
        }
        
        // Create a host result that will be returned at the end
        let host_result = Rc::new(RefCell::new(None::<Host>));
        
        // Host choice callback
        let hosts_clone = hosts.clone();
        let mut name_input_clone = name_input.clone();
        let mut hostname_input_clone = hostname_input.clone();
        let mut username_input_clone = username_input.clone();
        let mut port_input_clone = port_input.clone();
        let mut auth_choice_clone = auth_choice.clone();
        let mut key_input_clone = key_input.clone();
        let mut key_label_clone = key_label.clone();
        let mut key_input_inner = key_input.clone();
        let mut browse_button_clone = browse_button.clone();
        let mut delete_button_clone = delete_button.clone();
        
        host_choice.set_callback(move |c| {
            let selection = c.value();
            
            if selection < hosts_clone.len() as i32 {
                // Existing host
                let host = &hosts_clone[selection as usize];
                name_input_clone.set_value(&host.name);
                hostname_input_clone.set_value(&host.hostname);
                username_input_clone.set_value(&host.username);
                port_input_clone.set_value(&host.port.to_string());
                delete_button_clone.activate();
                
                if host.use_key_auth {
                    auth_choice_clone.set_value(1); // SSH Key
                    if let Some(path) = &host.key_path {
                        key_input_clone.set_value(path);
                    } else {
                        key_input_clone.set_value("");
                    }
                    key_label_clone.show();
                    key_input_clone.show();
                    browse_button_clone.show();
                } else {
                    auth_choice_clone.set_value(0); // Password
                    key_label_clone.hide();
                    key_input_clone.hide();
                    browse_button_clone.hide();
                }
            } else {
                // New host
                name_input_clone.set_value("New Host");
                hostname_input_clone.set_value("");
                username_input_clone.set_value("pi");
                port_input_clone.set_value("22");
                auth_choice_clone.set_value(0); // Password
                key_input_clone.set_value("");
                key_label_clone.hide();
                key_input_clone.hide();
                browse_button_clone.hide();
                delete_button_clone.deactivate();
            }
        });
        
        // Auth choice callback
        let mut key_label_clone = key_label.clone();
        let mut key_input_clone = key_input.clone();
        let mut browse_button_clone = browse_button.clone();
        
        auth_choice.set_callback(move |c| {
            let selection = c.value();
            
            if selection == 1 {
                // SSH Key
                key_label_clone.show();
                key_input_clone.show();
                browse_button_clone.show();
            } else {
                // Password
                key_label_clone.hide();
                key_input_clone.hide();
                browse_button_clone.hide();
            }
        });
        
        // Browse button callback
        browse_button.set_callback(move |_| {
            let mut dialog = FileDialog::new(FileDialogType::BrowseFile);
            dialog.set_title("Select SSH Key File");
            dialog.show();
            
            let filename = dialog.filename();
            if !filename.to_string_lossy().is_empty() {
                key_input_inner.set_value(&filename.to_string_lossy());
            }
        });
        
        // Test connection button callback
        let hostname_input_clone = hostname_input.clone();
        let username_input_clone = username_input.clone();
        let port_input_clone = port_input.clone();
        let auth_choice_clone = auth_choice.clone();
        let key_input_clone = key_input.clone();
        let mut status_frame_clone = status_frame.clone();
        
        test_button.set_callback(move |_| {
            let hostname = hostname_input_clone.value();
            let username = username_input_clone.value();
            let port_str = port_input_clone.value();
            let use_key_auth = auth_choice_clone.value() == 1;
            let key_path = if use_key_auth && !key_input_clone.value().is_empty() {
                Some(key_input_clone.value())
            } else {
                None
            };
            
            // Validate inputs
            if hostname.is_empty() || username.is_empty() || port_str.is_empty() {
                status_frame_clone.set_label("Error: All fields must be filled");
                status_frame_clone.set_label_color(Color::Red);
                return;
            }
            
            let port = match port_str.parse::<u16>() {
                Ok(p) => p,
                Err(_) => {
                    status_frame_clone.set_label("Error: Port must be a valid number");
                    status_frame_clone.set_label_color(Color::Red);
                    return;
                }
            };
            
            if use_key_auth && key_path.is_none() {
                status_frame_clone.set_label("Error: SSH key file must be selected for key authentication");
                status_frame_clone.set_label_color(Color::Red);
                return;
            }
            
            // Test connection using a command that prompts for password
            status_frame_clone.set_label("Testing connection...");
            status_frame_clone.set_label_color(Color::Blue);
            app::flush();
            
            // This uses sshpass to handle password for SSH
            use std::process::Command;
            
            let mut cmd;
            let mut has_password = false;
            
            if !use_key_auth {
                // For password auth, prompt for password using our custom dialog
                let password = password_dialog(
                    "SSH Password",
                    &format!("Enter password for {}@{}:", username, hostname)
                );
                
                if let Some(pass) = password {
                    // Use the password with sshpass
                    cmd = Command::new("sshpass");
                    cmd.arg("-p").arg(&pass);
                    cmd.arg("ssh");
                    has_password = true;
                } else {
                    // User canceled, abort connection test
                    status_frame_clone.set_label("Connection test canceled");
                    status_frame_clone.set_label_color(Color::Red);
                    return;
                }
            } else {
                // For key auth, use ssh directly
                cmd = Command::new("ssh");
                if let Some(path) = &key_path {
                    cmd.arg("-i").arg(path);
                }
            }
            
            // Add common options
            cmd.arg("-o").arg("NumberOfPasswordPrompts=1");  // Only prompt once
            cmd.arg("-o").arg("ConnectTimeout=10");         // Timeout after 10 seconds
            cmd.arg("-p").arg(port.to_string());            // Port
            
            // Add host
            cmd.arg(format!("{}@{}", username, hostname));
            
            // Add a simple test command that will execute on the remote host
            cmd.arg("echo 'Connection successful'");
            
            // Show the command for debugging (but mask password)
            let cmd_str = if has_password {
                // Create a safe version of the command string with password masked
                format!("sshpass -p ******** ssh -o NumberOfPasswordPrompts=1 -o ConnectTimeout=10 -p {} {}@{} \"echo 'Connection successful'\"", 
                    port, username, hostname)
            } else {
                format!("{:?}", cmd)
            };
            
            println!("Testing connection with command: {}", cmd_str);
            
            // Execute the command
            let result = cmd.output();
            
            match result {
                Ok(output) => {
                    let success = output.status.success();
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    
                    println!("Command output: {}", stdout);
                    println!("Command error: {}", stderr);
                    
                    if success {
                        status_frame_clone.set_label("Connection successful!");
                        status_frame_clone.set_label_color(Color::Green);
                    } else {
                        let error_msg = if stderr.contains("Permission denied") {
                            "Authentication failed. Check username/password or key."
                        } else if stderr.contains("Could not resolve hostname") {
                            "Hostname could not be resolved. Check network."
                        } else if stderr.contains("Connection refused") {
                            "Connection refused. Check if SSH server is running."
                        } else if stderr.contains("Connection timed out") {
                            "Connection timed out. Check hostname and network."
                        } else {
                            "Connection failed. See console for details."
                        };
                        
                        status_frame_clone.set_label(error_msg);
                        status_frame_clone.set_label_color(Color::Red);
                    }
                },
                Err(e) => {
                    println!("Failed to execute command: {}", e);
                    status_frame_clone.set_label("Failed to execute SSH command");
                    status_frame_clone.set_label_color(Color::Red);
                }
            }
        });
        
        // Delete button callback
        let host_choice_clone = host_choice.clone();
        let hosts_clone = hosts.clone();
        let config_clone = config.clone();
        
        delete_button.set_callback(move |_| {
            let selection = host_choice_clone.value();
            
            if selection < hosts_clone.len() as i32 {
                let result = choice_dialog(
                    "Confirm Delete",
                    &format!("Are you sure you want to delete the host '{}'?", hosts_clone[selection as usize].name),
                    &["Yes", "No"]
                );
                
                if result == 0 { // User clicked "Yes"
                    let mut config = config_clone.lock().unwrap();
                    
                    // Remove the host
                    if selection < config.hosts.len() as i32 {
                        config.hosts.remove(selection as usize);
                        
                        // Update last_used_host_index if needed
                        if config.last_used_host_index >= selection as usize && config.last_used_host_index > 0 {
                            config.last_used_host_index -= 1;
                        }
                        
                        // Save the updated config
                        if let Err(e) = config.save() {
                            message_dialog("Error", &format!("Failed to save config: {}", e));
                        }
                    }
                    
                    // Close dialog
                    if let Some(mut win) = app::first_window() {
                        win.hide();
                    }
                }
            }
        });
        
        // Save button callback
        let host_result_clone = host_result.clone();
        let host_choice_clone = host_choice.clone();
        let hosts_clone = hosts.clone();
        let config_clone = config.clone();
        let name_input_copy = name_input.clone();
        let hostname_input_copy = hostname_input.clone();
        let username_input_copy = username_input.clone();
        let port_input_copy = port_input.clone();
        let auth_choice_copy = auth_choice.clone();
        let key_input_copy = key_input.clone();
        
        save_button.set_callback(move |_| {
            let selection = host_choice_clone.value();
            let name = name_input_copy.value();
            let hostname = hostname_input_copy.value();
            let username = username_input_copy.value();
            let port_str = port_input_copy.value();
            let use_key_auth = auth_choice_copy.value() == 1;
            let key_path = if use_key_auth && !key_input_copy.value().is_empty() {
                Some(key_input_copy.value())
            } else {
                None
            };
            
            // Validate inputs
            if name.is_empty() || hostname.is_empty() || username.is_empty() || port_str.is_empty() {
                message_dialog("Error", "All fields must be filled");
                return;
            }
            
            let port = match port_str.parse::<u16>() {
                Ok(p) => p,
                Err(_) => {
                    message_dialog("Error", "Port must be a valid number");
                    return;
                }
            };
            
            if use_key_auth && key_path.is_none() {
                message_dialog("Error", "SSH key file must be selected for key authentication");
                return;
            }
            
            // Create host
            let new_host = Host {
                name,
                hostname,
                username,
                port,
                use_key_auth,
                key_path,
            };
            
            // Update config
            let mut config = config_clone.lock().unwrap();
            if selection < hosts_clone.len() as i32 {
                // Update existing host
                config.hosts[selection as usize] = new_host.clone();
            } else {
                // Add new host
                config.hosts.push(new_host.clone());
                config.last_used_host_index = config.hosts.len() - 1;
            }
            
            // Save the updated config
            if let Err(e) = config.save() {
                message_dialog("Error", &format!("Failed to save config: {}", e));
            }
            
            // Store the host result
            *host_result_clone.borrow_mut() = Some(new_host);
            
            // Close dialog
            if let Some(mut win) = app::first_window() {
                win.hide();
            }
        });
        
        dialog.end();
        dialog.show();
        
        while dialog.shown() {
            app::wait();
        }
        
        // Capture the result before it goes out of scope
        let final_result = host_result.borrow().clone();
        final_result
    }

    // Helper function for choice dialogs
    pub fn choice_dialog(title: &str, message: &str, options: &[&str]) -> i32 {
        let mut dialog = Window::new(100, 100, 300, 150, title);
        dialog.set_border(true);
        
        let padding = 10;
        let button_height = 25;
        let button_width = 80;
        
        let mut message_frame = Frame::new(
            padding, 
            padding, 
            300 - padding * 2, 
            70,
            message
        );
        message_frame.set_align(Align::Left | Align::Inside | Align::Top);
        
        // We need a way to track the choice across callbacks
        let choice = Rc::new(RefCell::new(-1));
        
        let mut buttons = Vec::new();
        let option_count = options.len();
        
        for (i, &option) in options.iter().enumerate() {
            let x = 300 - padding - button_width * (option_count - i) as i32;
            let mut button = Button::new(
                x, 
                150 - padding - button_height, 
                button_width, 
                button_height,
                option
            );
            
            let choice_clone = choice.clone();
            let i_val = i;
            
            button.set_callback(move |_| {
                // Set the choice when clicked
                *choice_clone.borrow_mut() = i_val as i32;
                
                // Hide the dialog
                if let Some(mut win) = app::first_window() {
                    win.hide();
                }
            });
            
            buttons.push(button);
        }
        
        dialog.end();
        dialog.show();
        
        // Wait for the dialog to close
        while dialog.shown() {
            app::wait();
        }
        
        // Return the choice
        let x = *choice.borrow(); x
    }

    // Add these helper functions for the operations panel
    pub fn resize_dialog() -> Option<(u32, u32)> {
        // Implement a dialog to get width and height
        // This is a simplified implementation
        let width = 800;
        let height = 600;
        Some((width, height))
    }

    pub fn brightness_dialog() -> Option<i32> {
        // Implement a dialog to get brightness level
        // Changed to return i32 instead of f32 to match BrightnessOperation
        Some(20) // For example, +20% brightness
    }
}