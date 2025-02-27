use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{self, Write};
use std::any::Any;

use crate::transfer::method::{TransferMethod, TransferError, TransferMethodFactory};

pub struct SSHTransfer {
    hostname: String,
    username: String,
    port: u16,
    use_key_auth: bool,
    key_path: Option<PathBuf>,
    password: Option<String>,
}

impl SSHTransfer {
    pub fn new(
        hostname: String,
        username: String,
        port: u16,
        use_key_auth: bool,
        key_path: Option<PathBuf>,
    ) -> Self {
        Self {
            hostname,
            username,
            port,
            use_key_auth,
            key_path,
            password: None,
        }
    }
    
    pub fn with_password(
        hostname: String,
        username: String,
        port: u16,
        password: String,
    ) -> Self {
        Self {
            hostname,
            username,
            port,
            use_key_auth: false,
            key_path: None,
            password: Some(password),
        }
    }
    
    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }
    
    // Debug function to help troubleshoot commands
    fn debug_command(&self, cmd: &mut Command, command_name: &str) -> Result<std::process::Output, TransferError> {
        // Print the command that's about to be executed (sanitize password for security)
        let mut cmd_str = format!("{:?}", cmd);
        if let Some(ref password) = self.password {
            cmd_str = cmd_str.replace(password, "********");
        }
        println!("Executing {}: {}", command_name, cmd_str);
        
        let output = cmd.output().map_err(|e| {
            TransferError::TransferFailed(format!("Failed to execute {}: {}", command_name, e))
        })?;
        
        // Print output status and contents
        println!("Command status: {}", output.status);
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        
        if !output.status.success() {
            return Err(TransferError::TransferFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(output)
    }
    
    // Get password from user interactively if needed
    fn ensure_password(&mut self) -> Result<(), TransferError> {
        if !self.use_key_auth && self.password.is_none() {
            // In a GUI app, this should be replaced with a proper password dialog
            print!("Enter password for {}@{}: ", self.username, self.hostname);
            io::stdout().flush().map_err(|e| {
                TransferError::TransferFailed(format!("Failed to flush stdout: {}", e))
            })?;
            
            // Simple CLI password input (replace with GUI dialog in real app)
            let mut password = String::new();
            io::stdin().read_line(&mut password).map_err(|e| {
                TransferError::TransferFailed(format!("Failed to read password: {}", e))
            })?;
            self.password = Some(password.trim().to_string());
        }
        Ok(())
    }
}

impl TransferMethod for SSHTransfer {
    fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &Path
    ) -> Result<(), TransferError> {
        // Create a mutable copy for potential password prompt
        let mut self_copy = self.clone();
        self_copy.ensure_password()?;
        
        // Choose command based on authentication method
        let mut cmd;
        
        if !self.use_key_auth {
            // For password auth, use sshpass
            if let Some(ref password) = self_copy.password {
                cmd = Command::new("sshpass");
                cmd.arg("-p").arg(password);
                cmd.arg("scp");
            } else {
                return Err(TransferError::TransferFailed(
                    "Password required for password authentication".to_string()
                ));
            }
        } else {
            // For key auth, use scp directly
            cmd = Command::new("scp");
        }
        
        // Add options
        cmd.arg("-P").arg(self.port.to_string());
        
        // Add key if using key authentication
        if self.use_key_auth {
            if let Some(key_path) = &self.key_path {
                cmd.arg("-i").arg(key_path);
            }
        }
        
        // Add source and destination
        cmd.arg(local_path);
        
        let remote = format!(
            "{}@{}:{}",
            self.username,
            self.hostname,
            remote_path.to_string_lossy()
        );
        cmd.arg(remote);
        
        // Use debug command
        self_copy.debug_command(&mut cmd, "scp upload")?;
        
        Ok(())
    }
    
    fn download_file(
        &self,
        remote_path: &Path,
        local_path: &Path
    ) -> Result<(), TransferError> {
        // Create a mutable copy for potential password prompt
        let mut self_copy = self.clone();
        self_copy.ensure_password()?;
        
        // Choose command based on authentication method
        let mut cmd;
        
        if !self.use_key_auth {
            // For password auth, use sshpass
            if let Some(ref password) = self_copy.password {
                cmd = Command::new("sshpass");
                cmd.arg("-p").arg(password);
                cmd.arg("scp");
            } else {
                return Err(TransferError::TransferFailed(
                    "Password required for password authentication".to_string()
                ));
            }
        } else {
            // For key auth, use scp directly
            cmd = Command::new("scp");
        }
        
        // Add options
        cmd.arg("-P").arg(self.port.to_string());
        
        // Add key if using key authentication
        if self.use_key_auth {
            if let Some(key_path) = &self.key_path {
                cmd.arg("-i").arg(key_path);
            }
        }
        
        // Add source and destination
        let remote = format!(
            "{}@{}:{}",
            self.username,
            self.hostname,
            remote_path.to_string_lossy()
        );
        cmd.arg(remote);
        cmd.arg(local_path);
        
        // Use debug command
        self_copy.debug_command(&mut cmd, "scp download")?;
        
        Ok(())
    }
    
    fn list_files(
        &self,
        remote_dir: &Path
    ) -> Result<Vec<(String, bool)>, TransferError> {
        // Create a mutable copy for potential password prompt
        let mut self_copy = self.clone();
        self_copy.ensure_password()?;
        
        // Choose command based on authentication method
        let mut cmd;
        
        if !self.use_key_auth {
            // For password auth, use sshpass
            if let Some(ref password) = self_copy.password {
                cmd = Command::new("sshpass");
                cmd.arg("-p").arg(password);
                cmd.arg("ssh");
            } else {
                return Err(TransferError::TransferFailed(
                    "Password required for password authentication".to_string()
                ));
            }
        } else {
            // For key auth, use ssh directly
            cmd = Command::new("ssh");
        }
        
        // Add options
        cmd.arg("-p").arg(self.port.to_string());
        
        // Add key if using key authentication
        if self.use_key_auth {
            if let Some(key_path) = &self.key_path {
                cmd.arg("-i").arg(key_path);
            }
        }
        
        // Add remote username and host
        let remote_user_host = format!("{}@{}", self.username, self.hostname);
        cmd.arg(remote_user_host);
        
        // Command to list files with format: name,is_dir
        let ls_cmd = format!("ls -la {}", remote_dir.to_string_lossy());
        cmd.arg(ls_cmd);
        
        println!("Executing SSH list files command: {:?}", cmd);
        
        // Execute command
        let output = cmd.output().map_err(|e| {
            TransferError::TransferFailed(format!("Failed to execute ssh/ls: {}", e))
        })?;
        
        // Debug output
        println!("Command status: {}", output.status);
        if !output.stdout.is_empty() {
            println!("STDOUT first 100 bytes: {:?}", 
                String::from_utf8_lossy(&output.stdout[..std::cmp::min(100, output.stdout.len())]));
        } else {
            println!("STDOUT is empty");
        }
        
        if !output.stderr.is_empty() {
            println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        if !output.status.success() {
            return Err(TransferError::TransferFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Parse output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();
        
        println!("Parsing output lines: {}", output_str.lines().count());
        
        // More robust parsing for ls -la output
        for line in output_str.lines().skip(1) { // Skip the first line (total)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let file_type = parts[0].chars().next().unwrap_or('-');
                let is_dir = file_type == 'd';
                let name = parts[8].to_string();
                
                // Skip . and .. directories
                if name != "." && name != ".." {
                    println!("Found file: {} (is_dir: {})", name, is_dir);
                    files.push((name, is_dir));
                }
            } else {
                println!("Couldn't parse line: {}", line);
            }
        }
        
        println!("Returning {} files", files.len());
        Ok(files)
    }
    
    fn get_name(&self) -> &str {
        "SSH Transfer"
    }
    
    fn get_description(&self) -> String {
        format!("SSH/SCP transfer to {}@{}", self.username, self.hostname)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn set_password(&mut self, password: &str) {
        self.password = Some(password.to_string());
    }
}

// Make SSHTransfer cloneable for password handling
impl Clone for SSHTransfer {
    fn clone(&self) -> Self {
        Self {
            hostname: self.hostname.clone(),
            username: self.username.clone(),
            port: self.port,
            use_key_auth: self.use_key_auth,
            key_path: self.key_path.clone(),
            password: self.password.clone(),
        }
    }
}

pub struct SSHTransferFactory {
    hostname: String,
    username: String,
    port: u16,
    use_key_auth: bool,
    key_path: Option<PathBuf>,
    password: Option<String>,
}

impl SSHTransferFactory {
    pub fn new(
        hostname: String,
        username: String,
        port: u16,
        use_key_auth: bool,
        key_path: Option<String>,
    ) -> Self {
        Self {
            hostname,
            username,
            port,
            use_key_auth,
            key_path: key_path.map(PathBuf::from),
            password: None,
        }
    }
    
    pub fn with_password(
        hostname: String,
        username: String,
        port: u16,
        password: String,
    ) -> Self {
        Self {
            hostname,
            username,
            port,
            use_key_auth: false,
            key_path: None,
            password: Some(password),
        }
    }
    
    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }
}

impl TransferMethodFactory for SSHTransferFactory {
    fn create_method(&self) -> Box<dyn TransferMethod> {
        let mut transfer = SSHTransfer::new(
            self.hostname.clone(),
            self.username.clone(),
            self.port,
            self.use_key_auth,
            self.key_path.clone(),
        );
        
        // Pass password if available
        if let Some(ref password) = self.password {
            transfer.set_password(password.clone());
        }
        
        Box::new(transfer)
    }
    
    fn get_name(&self) -> String {
        format!("SSH/SCP to {}@{}", self.username, self.hostname)
    }
}

// Compatibility module to match the original import path
pub mod ssh {
    pub use super::*;
}
