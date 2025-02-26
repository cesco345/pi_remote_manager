// src/ui/dialogs.rs - Dialog functions
pub mod dialogs {
    use fltk::{
        dialog,
        input::IntInput,
    };
    
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    
    use crate::config::{Config, Host};
    
    pub fn message_dialog(title: &str, message: &str) {
        dialog::message_default(&format!("{}: {}", title, message));
    }
    
    pub fn open_file_dialog(title: &str, filter: &str) -> Option<PathBuf> {
        let mut dialog = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
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
        let mut dialog = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
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
    
    pub fn choice_dialog(_title: &str, message: &str, choices: &[&str]) -> i32 {
        // Make sure we don't pass more arguments than expected
        if choices.len() >= 2 {
            dialog::choice_default(message, "Cancel", choices[0], choices[1])
        } else if choices.len() == 1 {
            dialog::choice_default(message, "Cancel", choices[0], "")
        } else {
            dialog::choice_default(message, "Cancel", "OK", "")
        }
    }
    
    pub fn resize_dialog() -> Option<(u32, u32)> {
        let mut width = 0;
        let mut height = 0;
        
        let width_ok = dialog::input_default("Enter width in pixels:", "").map(|w| {
            if let Ok(w) = w.parse::<u32>() {
                width = w;
                true
            } else {
                false
            }
        }).unwrap_or(false);
        
        if !width_ok {
            return None;
        }
        
        let height_ok = dialog::input_default("Enter height in pixels:", "").map(|h| {
            if let Ok(h) = h.parse::<u32>() {
                height = h;
                true
            } else {
                false
            }
        }).unwrap_or(false);
        
        if !height_ok {
            return None;
        }
        
        Some((width, height))
    }
    
    pub fn brightness_dialog() -> Option<i32> {
        dialog::input_default("Enter brightness adjustment (-100 to 100):", "").and_then(|value| {
            value.parse::<i32>().ok().map(|v| v.max(-100).min(100))
        })
    }
    
    pub fn connection_dialog(config: Arc<Mutex<Config>>) -> Option<Host> {
        // Get available hosts
        let hosts = {
            let config = config.lock().unwrap();
            config.hosts.clone()
        };
        
        // Create a host selection dialog if hosts exist
        let host_index = if !hosts.is_empty() {
            let mut host_names = Vec::new();
            for host in &hosts {
                host_names.push(format!("{} ({}@{})", host.name, host.username, host.hostname));
            }
            host_names.push("Add new host...".to_string());
            
            let selected = dialog::choice_default(
                "Choose a host or add a new one:",
                "Cancel",
                &host_names[0],
                if host_names.len() > 1 { &host_names[1] } else { "" }
            );
            
            if selected < 0 {
                return None;
            }
            
            if selected < hosts.len() as i32 {
                // Return the selected host
                return Some(hosts[selected as usize].clone());
            }
            
            // Add new host (continue with the dialog)
            -1
        } else {
            // No hosts, add a new one
            -1
        };
        
        // Name
        let name = match dialog::input_default("Enter a name for this host:", "") {
            Some(name) if !name.trim().is_empty() => name,
            _ => return None,
        };
        
        // Hostname
        let hostname = match dialog::input_default("Enter hostname or IP address:", "") {
            Some(hostname) if !hostname.trim().is_empty() => hostname,
            _ => return None,
        };
        
        // Username
        let username = match dialog::input_default("Enter username:", "") {
            Some(username) if !username.trim().is_empty() => username,
            _ => return None,
        };
        
        // Port
        let port = match dialog::input_default("Enter SSH port:", "22") {
            Some(port) => port.parse::<u16>().unwrap_or(22),
            _ => 22,
        };
        
        // Authentication method
        let use_key_auth = dialog::choice_default(
            "Choose authentication method:",
            "Cancel",
            "Password",
            "SSH Key"
        ) == 1;
        
        // Key path if using key auth
        let key_path = if use_key_auth {
            let mut dialog = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
            dialog.set_title("Select SSH Key File");
            dialog.show();
            
            let filename = dialog.filename();
            if filename.to_string_lossy().is_empty() {
                None
            } else {
                Some(filename.to_string_lossy().to_string())
            }
        } else {
            None
        };
        
        // Create and return the new host
        Some(Host {
            name,
            hostname,
            username,
            port,
            use_key_auth,
            key_path,
        })
    }
}