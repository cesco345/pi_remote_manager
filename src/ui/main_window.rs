// src/ui/main_window.rs - Main application window
pub mod main_window {
    use fltk::{
        app,
        enums::Shortcut,
        menu::{MenuBar, MenuFlag},
        group::{Group, Tabs},
        window::Window,
        prelude::*,
    };
    
    use std::sync::{Arc, Mutex};
    use std::path::PathBuf;
    
    use crate::core::image_processor::image_processor::{
        ImageProcessingService,
        JPEGProcessorFactory,
        PNGProcessorFactory,
    };
    
    

    use crate::config::Config;
    
    use crate::ui::file_browser::file_browser::FileBrowserPanel;
    use crate::ui::image_view::image_view::ImageViewPanel;
    use crate::ui::operations_panel::operations_panel::OperationsPanel;
    use crate::ui::transfer_panel::transfer_panel::TransferPanel;
    use crate::ui::dialogs::dialogs;
    
    pub struct MainWindow {
        window: Window,
        config: Arc<Mutex<Config>>,
        image_service: Arc<Mutex<ImageProcessingService>>,
        local_browser: FileBrowserPanel,
        remote_browser: FileBrowserPanel,
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
            Self::setup_menu(&mut menu_bar, config.clone(), image_service.clone());
            
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
                browser_height,  // Use browser_height here directly
                "Local Files"
            );
            
            // Create remote file browser panel (right side)
            let remote_browser = FileBrowserPanel::new(
                panel_width + 10, 
                content_y + 35, 
                panel_width, 
                browser_height,  // Use browser_height here directly
                "Raspberry Pi Files"
            );
            
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
                remote_browser,
                image_view,
                operations_panel,
                transfer_panel,
            };
            
            // Setup callbacks
            main_window.setup_callbacks();
            
            main_window
        }
        
        fn setup_menu(
            menu: &mut MenuBar, 
            config: Arc<Mutex<Config>>,
            image_service: Arc<Mutex<ImageProcessingService>>
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
            let config_clone = config.clone();
            menu.add(
                "&Connection/&Connect to Raspberry Pi...\t",
                Shortcut::Ctrl | 'r',
                MenuFlag::Normal,
                move |_| {
                    // Show connection dialog
                    if let Some(host) = dialogs::connection_dialog(config_clone.clone()) {
                        // Store connection in config
                        let mut config = config_clone.lock().unwrap();
                        
                        // Check if host already exists
                        if let Some(pos) = config.hosts.iter().position(|h| h.name == host.name) {
                            config.hosts[pos] = host.clone();
                        } else {
                            config.hosts.push(host.clone());
                        }
                        
                        // Save config
                        let _ = config.save();
                        
                        // Use the new connection
                        println!("Connecting to: {}", host.hostname);
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
            // Connect the transfer panel with file browsers
            let _local_browser = self.local_browser.clone();
            let _remote_browser = self.remote_browser.clone();
            self.transfer_panel.set_callback(move |source_is_local, source_path, dest_path| {
                if source_is_local {
                    // Upload from local to remote
                    println!("Upload: {} -> {}", source_path.display(), dest_path.display());
                    // Refresh remote browser after upload
                    // remote_browser.refresh();
                } else {
                    // Download from remote to local
                    println!("Download: {} -> {}", source_path.display(), dest_path.display());
                    // Refresh local browser after download
                    // local_browser.refresh();
                }
            });
            
            // Fix: Since we need Send + Sync for the closures, we'll use Arc<Mutex<>> instead of Rc<RefCell<>>
            let transfer_panel = Arc::new(Mutex::new(self.transfer_panel.clone()));
            
            let transfer_panel_clone = transfer_panel.clone();
            self.local_browser.set_callback(move |path, is_dir| {
                if !is_dir {
                    if let Ok(mut panel) = transfer_panel_clone.lock() {
                        panel.set_source_path(path, true);
                    }
                }
            });
            
            let transfer_panel_clone = transfer_panel.clone();
            self.remote_browser.set_callback(move |path, is_dir| {
                if !is_dir {
                    if let Ok(mut panel) = transfer_panel_clone.lock() {
                        panel.set_source_path(path, false);
                    }
                }
            });
            
            // TODO: Add more callbacks for image view, operations panel, etc.
        }
        
        pub fn show(&mut self) {
            self.window.show();
        }
    }
}