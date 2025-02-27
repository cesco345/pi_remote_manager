// This is an adapter that shows how to use the new PreviewPanel
// while maintaining backward compatibility with the current MainWindow

use fltk::{
    app,
    enums::{Shortcut, Event},
    menu::{MenuBar, MenuFlag},
    group::{Group, Tabs},
    window::Window,
    prelude::*,
};

use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;

use crate::core::image::{
    ImageProcessingService,
    JPEGProcessorFactory,
    PNGProcessorFactory,
};

use crate::config::Config;
use crate::transfer::ssh::SSHTransferFactory;

use crate::ui::file_browser::file_browser::FileBrowserPanel;
// Use the new preview panel
use crate::ui::preview::preview_panel::PreviewPanel;
use crate::ui::operations_panel::operations_panel::OperationsPanel;
use crate::ui::transfer_panel::transfer_panel::TransferPanel;
use crate::transfer::method::TransferMethodFactory;
use crate::ui::dialogs::dialogs;

// Use the new file type detection from core/file
use crate::core::file::file_type::is_image_file;

pub struct MainWindowAdapter {
    window: Window,
    config: Arc<Mutex<Config>>,
    image_service: Arc<Mutex<ImageProcessingService>>,
    local_browser: FileBrowserPanel,
    remote_browser_ref: Arc<Mutex<FileBrowserPanel>>, 
    // The new preview panel
    preview_panel: PreviewPanel,
    operations_panel: OperationsPanel,
    transfer_panel: TransferPanel,
    // Directory for temporary downloaded files
    temp_dir: PathBuf,
}

impl MainWindowAdapter {
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
        
        // Create remote file browser panel (right side)
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
        
        // Create preview panel (left side) - replaces image view
        let image_view_width = (width * 2) / 3;
        let preview_panel = PreviewPanel::new(
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
        
        // Setup temp directory for remote file previews
        let mut temp_dir = env::temp_dir();
        temp_dir.push("pi_image_processor_preview");
        
        // Create the temp directory if it doesn't exist
        if !temp_dir.exists() {
            let _ = fs::create_dir_all(&temp_dir);
        }
        
        // Finish the window
        window.end();
        window.make_resizable(true);
        
        // Create the main window struct
        let mut main_window = MainWindowAdapter {
            window,
            config,
            image_service,
            local_browser,
            remote_browser_ref,
            preview_panel,
            operations_panel,
            transfer_panel,
            temp_dir,
        };
        
        // Create a shared reference to the preview panel
        let preview_panel_ref = Arc::new(Mutex::new(main_window.preview_panel.clone()));
        
        // Setup menu with access to the remote browser and preview panel
        Self::setup_menu(
            &mut menu_bar, 
            main_window.config.clone(), 
            main_window.image_service.clone(),
            main_window.remote_browser_ref.clone(),
            preview_panel_ref.clone()
        );
        
        // Setup callbacks with the shared remote browser reference and preview panel
        main_window.setup_callbacks(tabs, content_y, preview_panel_ref);
        
        main_window
    }
    
    fn setup_menu(
        menu: &mut MenuBar, 
        config: Arc<Mutex<Config>>,
        image_service: Arc<Mutex<ImageProcessingService>>,
        remote_browser: Arc<Mutex<FileBrowserPanel>>,
        preview_panel: Arc<Mutex<PreviewPanel>>
    ) {
        // File menu
        let preview_panel_clone = preview_panel.clone();
        menu.add(
            "&File/&Open File...\t",
            Shortcut::Ctrl | 'o',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = dialogs::open_file_dialog("Open File", "") {
                    println!("Opening file: {}", path.display());
                    
                    // Get lock on the preview panel and preview the file
                    if let Ok(mut panel) = preview_panel_clone.lock() {
                        if panel.preview_file(&path) {
                            println!("Successfully previewed file: {}", path.display());
                        } else {
                            // Show error dialog if preview fails
                            dialogs::message_dialog(
                                "Error", 
                                &format!("Failed to preview file: {}", path.display())
                            );
                        }
                    }
                }
            },
        );
        
        // Add Save File menu item
        menu.add(
            "&File/&Save As...\t",
            Shortcut::Ctrl | 's',
            MenuFlag::Normal,
            |_| {
                if let Some(path) = dialogs::save_file_dialog("Save File As", "") {
                    println!("Save as: {}", path.display());
                    // Will be implemented later
                }
            },
        );
        
        // Exit menu item
        menu.add(
            "&File/&Exit\t",
            Shortcut::Ctrl | 'q',
            MenuFlag::Normal,
            |_| {
                app::quit();
            },
        );
        
        // Connection menu
        // ... (Connection menu items would be added here)
        
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
    
    fn setup_callbacks(
        &mut self, 
        mut tabs: Tabs, 
        content_y: i32, 
        preview_panel: Arc<Mutex<PreviewPanel>>
    ) {
        // Clone references for thread safety
        let local_browser = Arc::new(Mutex::new(self.local_browser.clone()));
        let remote_browser_clone = self.remote_browser_ref.clone();
        let temp_dir = self.temp_dir.clone();
        
        // Add a callback for tab selection
        let mut tabs_callback = tabs.clone();
        let preview_panel_tab_clone = preview_panel.clone();
        
        tabs.set_callback(move |tabs| {
            // Find which tab is selected by checking all child groups
            if let Some(tab) = tabs.value() {
                // The label() method returns a String, not an Option<String>
                let label = tab.label();
                println!("Selected tab: {}", label);
                
                // Check if the Image Processing tab is selected
                if label == "Image Processing" {
                    println!("Image Processing tab selected");
                    
                    // Refresh the preview panel if there's a current file
                    if let Ok(panel) = preview_panel_tab_clone.lock() {
                        if let Some(current_path) = panel.get_current_file() {
                            println!("Refreshing current file: {}", current_path.display());
                            // Force a redraw
                            app::redraw();
                        }
                    }
                }
            }
        });
        
        // Window resize callback
        let mut window_clone = self.window.clone();
        window_clone.resize_callback(move |_, _x, _y, w, h| {
            // Update the tabs size when the window is resized
            tabs_callback.resize(0, content_y, w, h - content_y);
            app::redraw();
        });
        
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
                    app::redraw();
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
        let preview_panel_clone = preview_panel.clone();
        self.local_browser.set_callback(move |path, is_dir| {
            if !is_dir {
                println!("Local file selected: {}", path.display());
                
                // Set the source path for transfer
                if let Ok(mut panel) = transfer_panel_clone.lock() {
                    panel.set_source_path(path.clone(), true);
                }
                
                // Preview the file regardless of type
                if let Ok(mut panel) = preview_panel_clone.lock() {
                    if panel.preview_file(&path) {
                        println!("Successfully previewed file");
                    } else {
                        println!("Failed to preview file");
                    }
                }
            }
        });
        
        // Remote browser file selection callback 
        let transfer_panel_clone = transfer_panel.clone();
        let remote_browser_clone = self.remote_browser_ref.clone();
        let preview_panel_clone = preview_panel.clone();
        let temp_dir_clone = temp_dir.clone();
        
        // First get a lock on the remote browser to set its callback
        if let Ok(mut remote_browser) = remote_browser_clone.lock() {
            remote_browser.set_callback(move |path, is_dir| {
                if !is_dir {
                    println!("Remote file selected: {}", path.display());
                    
                    // Set source path for transfer
                    if let Ok(mut panel) = transfer_panel_clone.lock() {
                        panel.set_source_path(path.clone(), false);
                    }
                    
                    // For remote files, we try to preview them
                    if path.exists() {
                        // File exists locally, preview it directly
                        println!("File exists locally, attempting preview");
                        if let Ok(mut panel) = preview_panel_clone.lock() {
                            if panel.preview_file(&path) {
                                println!("Successfully previewed remote file");
                            } else {
                                println!("Failed to preview remote file");
                            }
                        }
                    } else {
                        // Need to download the file to a temporary location for preview
                        println!("Remote file not available locally, downloading for preview");
                        
                        // Create a path in the temp directory
                        let mut temp_file = temp_dir_clone.clone();
                        if let Some(file_name) = path.file_name() {
                            temp_file.push(file_name);
                            
                            // Use the browser to download the file
                            if let Ok(browser) = remote_browser_clone.lock() {
                                match browser.download_remote_file(&path, &temp_file) {
                                    Ok(_) => {
                                        println!("Successfully downloaded to: {}", temp_file.display());
                                        
                                        // Now preview the downloaded file
                                        if let Ok(mut panel) = preview_panel_clone.lock() {
                                            if panel.preview_file(&temp_file) {
                                                println!("Successfully previewed downloaded file");
                                            } else {
                                                println!("Failed to preview downloaded file");
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        println!("Failed to download file for preview: {}", e);
                                        dialogs::message_dialog(
                                            "Download Error",
                                            &format!("Failed to download remote file: {}", e)
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            });
        } else {
            println!("ERROR: Could not lock remote browser to set callback");
        }
        
        // Add a handler to watch for events
        let remote_browser_clone = self.remote_browser_ref.clone();
        let mut window = self.window.clone();
        
        window.handle(move |_, ev| {
            match ev {
                Event::Close => {
                    println!("Window close event received");
                    if let Ok(browser) = remote_browser_clone.lock() {
                        browser.print_debug_status();
                    }
                    
                    // Clean up temp files when closing
                    Self::cleanup_temp_files(&temp_dir);
                    
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
    
    // Helper method to clean up temporary downloaded files
    fn cleanup_temp_files(temp_dir: &Path) {
        if temp_dir.exists() {
            if let Ok(entries) = fs::read_dir(temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Err(e) = fs::remove_file(&path) {
                            println!("Failed to remove temp file {}: {}", path.display(), e);
                        } else {
                            println!("Removed temp file: {}", path.display());
                        }
                    }
                }
            }
        }
    }
    
    pub fn show(&mut self) {
        self.window.show();
    }
}