// src/transfer/ssh.rs - SSH-based transfer methods
pub mod ssh {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::io;
    
    use crate::transfer::transfer_method::transfer_method::{TransferMethod, TransferError, TransferMethodFactory};
    
    pub struct SSHTransfer {
        hostname: String,
        username: String,
        port: u16,
        use_key_auth: bool,
        key_path: Option<PathBuf>,
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
            }
        }
    }
    
    impl TransferMethod for SSHTransfer {
        fn upload_file(
            &self,
            local_path: &Path,
            remote_path: &Path
        ) -> Result<(), TransferError> {
            // Use scp command for file transfer
            let mut cmd = Command::new("scp");
            
            // Add options
            cmd.arg("-P").arg(self.port.to_string());
            
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
            
            // Execute command
            let output = cmd.output().map_err(|e| {
                TransferError::TransferFailed(format!("Failed to execute scp: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(TransferError::TransferFailed(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
            
            Ok(())
        }
        
        fn download_file(
            &self,
            remote_path: &Path,
            local_path: &Path
        ) -> Result<(), TransferError> {
            // Use scp command for file transfer
            let mut cmd = Command::new("scp");
            
            // Add options
            cmd.arg("-P").arg(self.port.to_string());
            
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
            
            // Execute command
            let output = cmd.output().map_err(|e| {
                TransferError::TransferFailed(format!("Failed to execute scp: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(TransferError::TransferFailed(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
            
            Ok(())
        }
        
        fn list_files(
            &self,
            remote_dir: &Path
        ) -> Result<Vec<(String, bool)>, TransferError> {
            // Use ssh/ls to list files
            let mut cmd = Command::new("ssh");
            
            // Add options
            cmd.arg("-p").arg(self.port.to_string());
            
            if self.use_key_auth {
                if let Some(key_path) = &self.key_path {
                    cmd.arg("-i").arg(key_path);
                }
            }
            
            // Add remote username and host
            let remote_user_host = format!("{}@{}", self.username, self.hostname);
            cmd.arg(remote_user_host);
            
            // Command to list files with format: name,is_dir
            let ls_cmd = format!(
                "ls -la {} | awk '{{print $9 \",\" substr($1,1,1);}}'",
                remote_dir.to_string_lossy()
            );
            cmd.arg(ls_cmd);
            
            // Execute command
            let output = cmd.output().map_err(|e| {
                TransferError::TransferFailed(format!("Failed to execute ssh/ls: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(TransferError::TransferFailed(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
            
            // Parse output
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut files = Vec::new();
            
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() == 2 {
                    let name = parts[0].to_string();
                    let is_dir = parts[1] == "d";
                    
                    // Skip . and .. directories
                    if name != "." && name != ".." {
                        files.push((name, is_dir));
                    }
                }
            }
            
            Ok(files)
        }
        
        fn get_name(&self) -> &str {
            "SSH Transfer"
        }
        
        fn get_description(&self) -> String {
            format!("SSH/SCP transfer to {}@{}", self.username, self.hostname)
        }
    }
    
    pub struct SSHTransferFactory {
        hostname: String,
        username: String,
        port: u16,
        use_key_auth: bool,
        key_path: Option<PathBuf>,
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
            }
        }
    }
    
    impl TransferMethodFactory for SSHTransferFactory {
        fn create_method(&self) -> Box<dyn TransferMethod> {
            Box::new(SSHTransfer::new(
                self.hostname.clone(),
                self.username.clone(),
                self.port,
                self.use_key_auth,
                self.key_path.clone(),
            ))
        }
        
        fn get_name(&self) -> String {
            format!("SSH/SCP to {}@{}", self.username, self.hostname)
        }
    }
    
    // Rsync-based transfer (another concrete implementation)
    pub struct RsyncTransfer {
        hostname: String,
        username: String,
        port: u16,
        use_key_auth: bool,
        key_path: Option<PathBuf>,
        options: Vec<String>,
    }
    
    impl RsyncTransfer {
        pub fn new(
            hostname: String,
            username: String,
            port: u16,
            use_key_auth: bool,
            key_path: Option<PathBuf>,
            options: Vec<String>,
        ) -> Self {
            Self {
                hostname,
                username,
                port,
                use_key_auth,
                key_path,
                options,
            }
        }
    }
    
    impl TransferMethod for RsyncTransfer {
        fn upload_file(
            &self,
            local_path: &Path,
            remote_path: &Path
        ) -> Result<(), TransferError> {
            // Use rsync command for file transfer
            let mut cmd = Command::new("rsync");
            
            // Add standard options
            cmd.arg("-avz");
            
            // Add custom options
            for option in &self.options {
                cmd.arg(option);
            }
            
            // Add SSH options
            let mut ssh_opts = format!("ssh -p {}", self.port);
            
            if self.use_key_auth {
                if let Some(key_path) = &self.key_path {
                    ssh_opts.push_str(&format!(" -i {}", key_path.to_string_lossy()));
                }
            }
            
            cmd.arg("-e").arg(ssh_opts);
            
            // Add source and destination
            cmd.arg(local_path);
            
            let remote = format!(
                "{}@{}:{}",
                self.username,
                self.hostname,
                remote_path.to_string_lossy()
            );
            cmd.arg(remote);
            
            // Execute command
            let output = cmd.output().map_err(|e| {
                TransferError::TransferFailed(format!("Failed to execute rsync: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(TransferError::TransferFailed(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
            
            Ok(())
        }
        
        fn download_file(
            &self,
            remote_path: &Path,
            local_path: &Path
        ) -> Result<(), TransferError> {
            // Use rsync command for file transfer
            let mut cmd = Command::new("rsync");
            
            // Add standard options
            cmd.arg("-avz");
            
            // Add custom options
            for option in &self.options {
                cmd.arg(option);
            }
            
            // Add SSH options
            let mut ssh_opts = format!("ssh -p {}", self.port);
            
            if self.use_key_auth {
                if let Some(key_path) = &self.key_path {
                    ssh_opts.push_str(&format!(" -i {}", key_path.to_string_lossy()));
                }
            }
            
            cmd.arg("-e").arg(ssh_opts);
            
            // Add source and destination
            let remote = format!(
                "{}@{}:{}",
                self.username,
                self.hostname,
                remote_path.to_string_lossy()
            );
            cmd.arg(remote);
            cmd.arg(local_path);
            
            // Execute command
            let output = cmd.output().map_err(|e| {
                TransferError::TransferFailed(format!("Failed to execute rsync: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(TransferError::TransferFailed(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
            
            Ok(())
        }
        
        fn list_files(
            &self,
            remote_dir: &Path
        ) -> Result<Vec<(String, bool)>, TransferError> {
            // We'll reuse the SSH implementation for listing files
            let ssh = SSHTransfer::new(
                self.hostname.clone(),
                self.username.clone(),
                self.port,
                self.use_key_auth,
                self.key_path.clone(),
            );
            
            ssh.list_files(remote_dir)
        }
        
        fn get_name(&self) -> &str {
            "Rsync Transfer"
        }
        
        fn get_description(&self) -> String {
            format!("Rsync transfer to {}@{} with options: {}", 
                self.username, 
                self.hostname, 
                self.options.join(" "))
        }
    }
    
    pub struct RsyncTransferFactory {
        hostname: String,
        username: String,
        port: u16,
        use_key_auth: bool,
        key_path: Option<PathBuf>,
        options: Vec<String>,
    }
    
    impl RsyncTransferFactory {
        pub fn new(
            hostname: String,
            username: String,
            port: u16,
            use_key_auth: bool,
            key_path: Option<String>,
            options: Vec<String>,
        ) -> Self {
            Self {
                hostname,
                username,
                port,
                use_key_auth,
                key_path: key_path.map(PathBuf::from),
                options,
            }
        }
    }
    
    impl TransferMethodFactory for RsyncTransferFactory {
        fn create_method(&self) -> Box<dyn TransferMethod> {
            Box::new(RsyncTransfer::new(
                self.hostname.clone(),
                self.username.clone(),
                self.port,
                self.use_key_auth,
                self.key_path.clone(),
                self.options.clone(),
            ))
        }
        
        fn get_name(&self) -> String {
            format!("Rsync to {}@{}", self.username, self.hostname)
        }
    }
}
