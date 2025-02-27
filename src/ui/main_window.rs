// src/ui/main_window.rs - Main application window
pub mod main_window {
    use fltk::{
        app,
        enums::{Shortcut, Event},
        menu::{MenuBar, MenuFlag},
        group::{Group, Tabs},
        window::Window,
        prelude::*,
    };
    // Removed unused import: fltk::dialog::password
    
    use std::sync::{Arc, Mutex};
    use std::path::PathBuf;
    
    use crate::core::image::{
        ImageProcessingService,
        JPEGProcessorFactory,
        PNGProcessorFactory,
    };
    
    use crate::config::Config;
    use crate::transfer::ssh::SSHTransferFactory;
    // Removed unused import: crate::transfer::ssh::SSHTransfer
    
    use crate::ui::file_browser::file_browser::FileBrowserPanel;
    use crate::ui::image_view::image_view::ImageViewPanel;
    use crate::ui::operations_panel::operations_panel::OperationsPanel;
    use crate::ui::transfer_panel::transfer_panel::TransferPanel;
    use crate::transfer::method::TransferMethodFactory;
    use crate::ui::dialogs::dialogs;
    
    pub struct MainWindow {
        window: Window,
        config: Arc<Mutex<Config>>,
        image_service: Arc<Mutex<ImageProcessingService>>,
        local_browser: FileBrowserPanel,
        // Store a reference to the actual browser instance
        remote_browser_ref: Arc<Mutex<FileBrowserPanel>>, 
        image_view: ImageViewPanel,
        operations_panel: OperationsPanel,
        transfer_panel: TransferPanel,
    }
    
    impl MainWindow {
        pub fn new(title: &str, width: i32, height: i32) -> Self {
            // Create main window
            let mut window = Window::new(100, 100, width, height, title);
            
            // Load configuration
            let config = Arc::new(Mutex::new(Config::load().unwrap_or_else(|_| Config::default())));
            
            // Create image processing service
            let mut image_service = ImageProcessingService::new();
            
            // Register image processor factories
            image_service.register_factory(Box::new(JPEGProcessorFactory::new(85)));
            image_service.register_factory(Box::new(PNGProcessorFactory::new(6)));
            // Add more factories as needed
            
            let image_service = Arc::new(Mutex::new(image_service));
            
            // Create menu bar
            let mut menu_bar = MenuBar::new(0, 0, width, 30, "");
            
            // Create main layout
            let content_y = 30; // Below menu bar
            let content_height = height - content_y;
            
            // Create tabs
            let tabs = Tabs::new(0, content_y, width, content_height, "");
            
            // Add tabs
            tabs.begin();
            
            // File Browser Tab
            let browser_tab = Group::new(0, content_y + 30, width, content_height - 30, "File Browser");
            browser_tab.begin();
            
            // Split the browser tab horizontally
            let panel_width = width / 2 - 5;
            
            // Create transfer panel (at the bottom first to get height)
            let transfer_panel_height = 120;
            let browser_height = content_height - 35 - transfer_panel_height - 10;
            
            // Create local file browser panel (left side)
            let mut local_browser = FileBrowserPanel::new(
                0, 
                content_y + 35, 
                panel_width, 
                browser_height,
                "Local Files"
            );
            
            // Create remote file browser panel (right side) and immediately wrap in Arc<Mutex<>>
            let remote_browser = FileBrowserPanel::new(
                panel_width + 10, 
                content_y + 35, 
                panel_width, 
                browser_height,
                "Raspberry Pi Files"
            );
            
            let remote_browser_ref = Arc::new(Mutex::new(remote_browser));
            
            let transfer_panel = TransferPanel::new(
                0,
                content_y + 35 + browser_height + 5,
                width,
                transfer_panel_height,
                config.clone()
            );
            
            browser_tab.end();
            
            // Image Processing Tab
            let image_tab = Group::new(0, content_y + 30, width, content_height - 30, "Image Processing");
            image_tab.begin();
            
            // Create image view panel (left side)
            let image_view_width = (width * 2) / 3;
            let image_view = ImageViewPanel::new(
                0,
                content_y + 35,
                image_view_width,
                content_height - 35
            );
            
            // Create operations panel (right side)
            let operations_width = width - image_view_width - 5;
            let operations_panel = OperationsPanel::new(
                image_view_width + 5,
                content_y + 35,
                operations_width,
                content_height - 35,
                image_service.clone()
            );
            
            image_tab.end();
            
            tabs.end();
            
            // Set initial directory for file browsers
            let default_dir = config.lock().unwrap().default_local_dir.clone();
            local_browser.set_directory(&PathBuf::from(&default_dir));
            
            // Finish the window
            window.end();
            window.make_resizable(true);
            
            // Create the main window struct
            let mut main_window = MainWindow {
                window,
                config,
                image_service,
                local_browser,
                remote_browser_ref,
                image_view,
                operations_panel,
                transfer_panel,
            };
            
            // Setup callbacks with the shared remote browser reference
            main_window.setup_callbacks();
            
            // Setup menu with access to the remote browser
            Self::setup_menu(
                &mut menu_bar, 
                main_window.config.clone(), 
                main_window.image_service.clone(),
                main_window.remote_browser_ref.clone()
            );
            
            main_window
        }
        
        fn setup_menu(
            menu: &mut MenuBar, 
            config: Arc<Mutex<Config>>,
            image_service: Arc<Mutex<ImageProcessingService>>,
            remote_browser: Arc<Mutex<FileBrowserPanel>>
        ) {
            // File menu
            menu.add(
                "&File/&Open Image...\t",
                Shortcut::Ctrl | 'o',
                MenuFlag::Normal,
                move |_| {
                    if let Some(path) = dialogs::open_file_dialog("Open Image", "") {
                        // Handle opening the image
                        println!("Opening image: {}", path.display());
                    }
                },
            );
            
            menu.add(
                "&File/&Save Image As...\t",
                Shortcut::Ctrl | 's',
                MenuFlag::Normal,
                |_| {
                    if let Some(path) = dialogs::save_file_dialog("Save Image As", "") {
                        // Handle saving the image
                        println!("Saving image to: {}", path.display());
                    }
                },
            );
            
            menu.add(
                "&File/&Exit\t",
                Shortcut::Ctrl | 'q',
                MenuFlag::Normal,
                |_| {
                    app::quit();
                },
            );
            
            // Connection menu
            let config_clone1 = config.clone();
            let remote_browser_clone1 = remote_browser.clone();

            menu.add(
                "&Connection/&Connect to Raspberry Pi...\t",
                Shortcut::Ctrl | 'r',
                MenuFlag::Normal,
                move |_| {
                    // Show connection dialog without locking anything first
                    if let Some(host) = dialogs::connection_dialog(config_clone1.clone()) {
                        // Now we have a host, update config
                        {
                            let mut config = config_clone1.lock().unwrap();
                            
                            // Check if host already exists
                            if let Some(pos) = config.hosts.iter().position(|h| h.name == host.name) {
                                config.hosts[pos] = host.clone();
                            } else {
                                config.hosts.push(host.clone());
                            }
                            
                            // Save config
                            let _ = config.save();
                        }
                        
                        // If using password auth, prompt for password
                        let mut password_opt = None;
                        if !host.use_key_auth {
                            password_opt = dialogs::password_dialog(
                                "SSH Password",
                                &format!("Enter password for {}@{}:", host.username, host.hostname)
                            );
                        }
                        
                        // Create SSH connection to list remote files
                        let factory = SSHTransferFactory::new(
                            host.hostname.clone(),
                            host.username.clone(),
                            host.port,
                            host.use_key_auth,
                            host.key_path.clone(),
                        );
                        
                        let mut transfer_method = factory.create_method();
                        
                        // If password was provided, set it in the transfer method
                        if let Some(password) = &password_opt {
                            transfer_method.set_password(password);
                        }
                        
                        // Set initial remote directory (usually /home/username)
                        let remote_home = PathBuf::from(format!("/home/{}", host.username));
                        
                        println!("DEBUG: About to set remote directory with path: {}", remote_home.display());
                        println!("DEBUG: Transfer method: {}", transfer_method.get_name());
                        
                        // Get a mutable reference to the actual remote browser through the mutex
                        if let Ok(mut browser) = remote_browser_clone1.lock() {
                            // Store credentials for future use
                            browser.current_hostname = Some(host.hostname.clone());
                            browser.current_username = Some(host.username.clone());
                            browser.current_password = password_opt;
                            
                            // Configure the remote browser with the transfer method and initial path
                            browser.set_remote_directory(&remote_home, transfer_method);
                            
                            // Force a UI refresh after setting up the connection
                            app::flush();  // Flush pending UI events
                            app::awake();  // Wake up the UI thread
                            app::redraw(); // Force complete redraw
                            
                            // Print debug status after connection
                            browser.print_debug_status();
                            
                            println!("DEBUG: Set remote directory successfully");
                            println!("Connected to: {} and set remote home to: {}", 
                                    host.hostname, remote_home.display());
                        } else {
                            println!("Error: Could not lock remote browser");
                        }
                    }
                },
            );

            // Add a new menu item to directly show Raspberry Pi files
            let config_clone2 = config.clone();
            let remote_browser_clone2 = remote_browser.clone();

            menu.add(
                "&Connection/&Show Raspberry Pi Files\t",
                Shortcut::None,
                MenuFlag::Normal,
                move |_| {
                    println!("DEBUG: Show Raspberry Pi Files clicked");
                    
                    // Ask for password first since we need it for the connection
                    let password = dialogs::password_dialog("SSH Password", "Enter password for Raspberry Pi:");
                    
                    // First get the saved config to use stored credentials
                    if let Ok(config) = config_clone2.lock() {
                        // Find a Raspberry Pi host in saved hosts
                        let host = config.hosts.iter().find(|h| 
                            h.hostname.contains("raspberry") || 
                            h.hostname.contains("pi") || 
                            h.name.contains("Raspberry") || 
                            h.name.contains("Pi")
                        );
                        
                        let (hostname, username, port) = if let Some(pi_host) = host {
                            println!("Using saved Raspberry Pi connection: {}", pi_host.name);
                            (
                                pi_host.hostname.clone(),
                                pi_host.username.clone(),
                                pi_host.port
                            )
                        } else {
                            println!("No saved Raspberry Pi host found, using defaults");
                            ("raspberrypi.local".to_string(), "pi".to_string(), 22)
                        };
                        
                        if let Ok(mut browser) = remote_browser_clone2.lock() {
                            // Print current status
                            browser.print_debug_status();
                            
                            // Create SSH connection with password
                            let factory = SSHTransferFactory::new(
                                hostname.clone(),
                                username.clone(),
                                port,
                                false,      // Use password auth
                                None,       // No key path
                            );
                            
                            let mut transfer_method = factory.create_method();
                            
                            // Set the password directly in the transfer method
                            if let Some(pwd) = &password {
                                transfer_method.set_password(pwd);
                                println!("Set password for SSH connection");
                                
                                // Also store it in the browser for later use
                                browser.current_password = password.clone();
                            }
                            
                            let remote_home = PathBuf::from(format!("/home/{}", username));
                            
                            println!("Setting up direct connection to Raspberry Pi at {}", remote_home.display());
                            
                            // Store credentials
                            browser.current_hostname = Some(hostname.clone());
                            browser.current_username = Some(username.clone());
                            browser.current_password = password.clone();
                            
                            // Force it into remote mode with the new connection
                            browser.set_remote_directory(&remote_home, transfer_method);
                            
                            // Force UI update
                            app::flush();
                            app::awake();
                            app::redraw();
                            
                            // Print status again
                            browser.print_debug_status();
                            
                            println!("DEBUG: Show Raspberry Pi Files complete");
                        } else {
                            println!("ERROR: Could not lock remote browser");
                        }
                    } else {
                        println!("ERROR: Could not get config");
                    }
                },
            );

            // Add a special debug menu item to force remote refresh
            let remote_browser_clone3 = remote_browser.clone();
            menu.add(
                "&Connection/&Force Remote Refresh\t",
                Shortcut::None,
                MenuFlag::Normal,
                move |_| {
                    println!("DEBUG: Force Remote Refresh menu clicked");
                    
                    if let Ok(mut browser) = remote_browser_clone3.lock() {
                        // Check if we're in remote mode
                        println!("DEBUG: Remote mode: {}", browser.is_remote());
                        println!("DEBUG: Has transfer method: {}", browser.has_transfer_method());
                        
                        if browser.is_remote() && browser.has_transfer_method() {
                            println!("DEBUG: Remote mode confirmed, refreshing browser");
                            browser.refresh();
                        } else if browser.is_remote() && !browser.has_transfer_method() {
                            println!("DEBUG: In remote mode but no transfer method! Forcing remote mode...");
                            browser.force_remote_mode(); 
                        } else {
                            println!("DEBUG: Not in remote mode, forcing it");
                            browser.force_remote_mode();
                        }
                        
                        // Explicitly refresh and force UI update
                        app::flush();
                        app::awake();
                        app::redraw();
                        
                        // Print debug status
                        browser.print_debug_status();
                        
                        println!("DEBUG: Remote refresh complete");
                    } else {
                        println!("ERROR: Could not lock remote browser");
                    }
                },
            );

            // Add a debug info menu item
            let remote_browser_clone4 = remote_browser.clone();
            menu.add(
                "&Connection/&Show Debug Info\t",
                Shortcut::None,
                MenuFlag::Normal,
                move |_| {
                    if let Ok(browser) = remote_browser_clone4.lock() {
                        browser.print_debug_status();
                        dialogs::message_dialog(
                            "Browser Status", 
                            &format!(
                                "Remote mode: {}\nHas transfer: {}", 
                                browser.is_remote(),
                                browser.has_transfer_method()
                                // Removed private field access to current_dir
                            )
                        );
                    } else {
                        println!("ERROR: Could not lock remote browser");
                    }
                },
            );
            
            // Processing menu - Fix: Clone image_service for each closure
            let image_service_clone1 = image_service.clone();
            menu.add(
                "&Processing/&Apply Operations\t",
                Shortcut::Ctrl | 'a',
                MenuFlag::Normal,
                move |_| {
                    // Apply image processing operations
                    let service_guard = image_service_clone1.lock().unwrap();
                    let operations = service_guard.get_operations();
                    println!("Applying {} operations", operations.len());
                    // Actually apply operations to the current image
                },
            );
            
            let image_service_clone2 = image_service.clone();
            menu.add(
                "&Processing/&Reset Operations\t",
                Shortcut::Ctrl | 'r',
                MenuFlag::Normal,
                move |_| {
                    // Reset all operations
                    image_service_clone2.lock().unwrap().clear_operations();
                    println!("Reset all operations");
                },
            );
            
            // Help menu
            menu.add(
                "&Help/&About\t",
                Shortcut::None,
                MenuFlag::Normal,
                |_| {
                    dialogs::message_dialog(
                        "About Pi Image Processor",
                        "Pi Image Processor\nA tool for processing images on Raspberry Pi\n\nVersion 1.0.0"
                    );
                },
            );
        }
        
        fn setup_callbacks(&mut self) {
            // Clone for local browser
            let local_browser = Arc::new(Mutex::new(self.local_browser.clone()));
            
            // Use the existing remote browser reference
            let remote_browser_clone = self.remote_browser_ref.clone();
            
            // Connect the transfer panel with file browsers
            self.transfer_panel.set_callback(move |source_is_local, source_path, dest_path| {
                if source_is_local {
                    // Upload from local to remote
                    println!("Upload: {} -> {}", source_path.display(), dest_path.display());
                    // Refresh remote browser after upload
                    if let Ok(mut browser) = remote_browser_clone.lock() {
                        browser.refresh();
                        
                        // Force a UI refresh after the refresh operation
                        app::flush();
                        app::awake();
                        app::redraw(); // Add redraw for better UI update
                    }
                } else {
                    // Download from remote to local
                    println!("Download: {} -> {}", source_path.display(), dest_path.display());
                    // Refresh local browser after download
                    if let Ok(mut browser) = local_browser.lock() {
                        browser.refresh();
                        
                        // Force UI update here too
                        app::flush();
                        app::awake();
                        app::redraw();
                    }
                }
            });
            
            // Create a thread-safe reference to the transfer panel
            let transfer_panel = Arc::new(Mutex::new(self.transfer_panel.clone()));
            
            // Local browser file selection callback
            let transfer_panel_clone = transfer_panel.clone();
            self.local_browser.set_callback(move |path, is_dir| {
                if !is_dir {
                    println!("Local file selected: {}", path.display());
                    if let Ok(mut panel) = transfer_panel_clone.lock() {
                        panel.set_source_path(path, true);
                    }
                }
            });
            
            // Remote browser file selection callback 
            let transfer_panel_clone = transfer_panel.clone();
            let remote_browser_clone = self.remote_browser_ref.clone();
            
            // First get a lock on the remote browser to set its callback
            if let Ok(mut remote_browser) = remote_browser_clone.lock() {
                remote_browser.set_callback(move |path, is_dir| {
                    if !is_dir {
                        println!("Remote file selected: {}", path.display());
                        if let Ok(mut panel) = transfer_panel_clone.lock() {
                            panel.set_source_path(path, false);
                        }
                    }
                });
            } else {
                println!("ERROR: Could not lock remote browser to set callback");
            }
            
            // Add a handler to watch for events
            // Note: Modified to use FLTK's event handling mechanism correctly
            let remote_browser_clone = self.remote_browser_ref.clone();
            let mut window = self.window.clone();
            
            window.handle(move |_, ev| {
                match ev {
                    Event::Close => {
                        println!("Window close event received");
                        if let Ok(browser) = remote_browser_clone.lock() {
                            browser.print_debug_status();
                        }
                        false // Allow default handling to continue
                    },
                    Event::Focus => {
                        println!("Window focus event received");
                        if let Ok(browser) = remote_browser_clone.lock() {
                            browser.print_debug_status();
                        }
                        false // Allow default handling to continue
                    },
                    _ => false,
                }
            });
        }
        
        pub fn show(&mut self) {
            self.window.show();
        }
    }
}