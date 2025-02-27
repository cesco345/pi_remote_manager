// A hybrid main window that supports both the old ImageViewPanel
// and the new PreviewPanel for a gradual migration

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
// Include both ImageViewPanel and PreviewPanel
use crate::ui::image_view::image_view::ImageViewPanel;
use crate::ui::preview::preview_panel::PreviewPanel;
use crate::ui::operations_panel::operations_panel::OperationsPanel;
use crate::ui::transfer_panel::transfer_panel::TransferPanel;
use crate::transfer::method::TransferMethodFactory;
use crate::ui::dialogs::dialogs;

// Flag to control which preview system to use
// Set to true to use the new preview system
const USE_NEW_PREVIEW: bool = false;

pub struct HybridMainWindow {
    window: Window,
    config: Arc<Mutex<Config>>,
    image_service: Arc<Mutex<ImageProcessingService>>,
    local_browser: FileBrowserPanel,
    remote_browser_ref: Arc<Mutex<FileBrowserPanel>>, 
    // Keep both preview systems
    image_view: ImageViewPanel,
    preview_panel: Option<PreviewPanel>,
    operations_panel: OperationsPanel,
    transfer_panel: TransferPanel,
    // Directory for temporary downloaded files
    temp_dir: PathBuf,
}

impl HybridMainWindow {
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
        
        // Always create the ImageViewPanel
        let image_view = ImageViewPanel::new(
            0,
            content_y + 35,
            image_view_width,
            content_height - 35
        );
        
        // Optionally create the PreviewPanel
        let preview_panel = if USE_NEW_PREVIEW {
            // If using the new preview panel, create it and hide the old image view
            let panel = PreviewPanel::new(
                0,
                content_y + 35,
                image_view_width,
                content_height - 35
            );
            Some(panel)
        } else {
            None
        };
        
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
        let mut main_window = HybridMainWindow {
            window,
            config,
            image_service,
            local_browser,
            remote_browser_ref,
            image_view,
            preview_panel,
            operations_panel,
            transfer_panel,
            temp_dir,
        };
        
        // Create shared references
        let image_view_ref = Arc::new(Mutex::new(main_window.image_view.clone()));
        let preview_panel_ref = main_window.preview_panel.as_ref().map(|panel| {
            Arc::new(Mutex::new(panel.clone()))
        });
        
        // Setup menu with access to the remote browser and image view systems
        Self::setup_menu(
            &mut menu_bar, 
            main_window.config.clone(), 
            main_window.image_service.clone(),
            main_window.remote_browser_ref.clone(),
            image_view_ref.clone(),
            preview_panel_ref.clone()
        );
        
        // Setup callbacks with the shared remote browser reference and image view systems
        main_window.setup_callbacks(
            tabs, 
            content_y, 
            image_view_ref,
            preview_panel_ref
        );
        
        main_window
    }
    
    fn setup_menu(
        menu: &mut MenuBar, 
        config: Arc<Mutex<Config>>,
        image_service: Arc<Mutex<ImageProcessingService>>,
        remote_browser: Arc<Mutex<FileBrowserPanel>>,
        image_view: Arc<Mutex<ImageViewPanel>>,
        preview_panel: Option<Arc<Mutex<PreviewPanel>>>
    ) {
        // File menu
        let image_view_clone = image_view.clone();
        let preview_panel_clone = preview_panel.clone();
        
        menu.add(
            "&File/&Open File...\t",
            Shortcut::Ctrl | 'o',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = dialogs::open_file_dialog("Open File", "") {
                    println!("Opening file: {}", path.display());
                    
                    let mut success = false;
                    
                    // Try to load with the new preview panel if available
                    if let Some(ref panel_ref) = preview_panel_clone {
                        if let Ok(mut panel) = panel_ref.lock() {
                            if panel.preview_file(&path) {
                                println!("Successfully previewed file with new preview panel");
                                success = true;
                            } else {
                                println!("Failed to preview with new preview panel");
                            }
                        }
                    }
                    
                    // If new preview failed or is not available, try with the old image view
                    if !success {
                        if let Ok(mut view) = image_view_clone.lock() {
                            if view.load_image(&path) {
                                println!("Successfully loaded image with old image view");
                                success = true;
                            } else {
                                println!("Failed to load image with old image view");
                            }
                        }
                    }
                    
                    // Show error if both methods failed
                    if !success {
                        dialogs::message_dialog(
                            "Error", 
                            &format!("Failed to open file: {}", path.display())
                        );
                    }
                }
            },
        );
        
        // Save File menu item
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
        
        // Connection menu would be added here
        
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
        image_view: Arc<Mutex<ImageViewPanel>>,
        preview_panel: Option<Arc<Mutex<PreviewPanel>>>
    ) {
        // Clone references for thread safety
        let local_browser = Arc::new(Mutex::new(self.local_browser.clone()));
        let remote_browser_clone = self.remote_browser_ref.clone();
        let temp_dir = self.temp_dir.clone();
        
        // Add a callback for tab selection
        let mut tabs_callback = tabs.clone();
        let image_view_tab_clone = image_view.clone();
        let preview_panel_tab_clone = preview_panel.clone();
        
        tabs.set_callback(move |tabs| {
            // Find which tab is selected
            if let Some(tab) = tabs.value() {
                let label = tab.label();
                println!("Selected tab: {}", label);
                
                // Check if the Image Processing tab is selected
                if label == "Image Processing" {
                    println!("Image Processing tab selected");
                    
                    // Refresh the preview panel if there's a current file
                    let mut refreshed = false;
                    
                    if let Some(ref panel_ref) = preview_panel_tab_clone {
                        if let Ok(panel) = panel_ref.lock() {
                            if let Some(current_path) = panel.get_current_file() {
                                println!("Refreshing current file in preview panel: {}", current_path.display());
                                refreshed = true;
                                app::redraw();
                            }
                        }
                    }
                    
                    // If preview panel didn't refresh, try with the old image view
                    if !refreshed {
                        if let Ok(view) = image_view_tab_clone.lock() {
                            if let Some(current_path) = view.get_current_image() {
                                println!("Refreshing current image in image view: {}", current_path.display());
                                app::redraw();
                            }
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
        let image_view_clone = image_view.clone();
        let preview_panel_clone = preview_panel.clone();
        
        self.local_browser.set_callback(move |path, is_dir| {
            if !is_dir {
                println!("Local file selected: {}", path.display());
                
                // Set the source path for transfer
                if let Ok(mut panel) = transfer_panel_clone.lock() {
                    panel.set_source_path(path.clone(), true);
                }
                
                let mut success = false;
                
                // Try to preview with the new preview panel if available
                if let Some(ref panel_ref) = preview_panel_clone {
                    if let Ok(mut panel) = panel_ref.lock() {
                        if panel.preview_file(&path) {
                            println!("Successfully previewed file with new preview panel");
                            success = true;
                        }
                    }
                }
                
                // If new preview failed or is not available, try with the old image view
                // but only if it's an image file
                if !success && FileBrowserPanel::is_image_file(&path) {
                    if let Ok(mut view) = image_view_clone.lock() {
                        if view.load_image(&path) {
                            println!("Successfully loaded image with old image view");
                        } else {
                            println!("Failed to load image with old image view");
                        }
                    }
                }
            }
        });
        
        // Remote browser file selection callback 
        let transfer_panel_clone = transfer_panel.clone();
        let remote_browser_clone = self.remote_browser_ref.clone();
        let image_view_clone = image_view.clone();
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
                    
                    // For remote files, check if they exist locally first
                    if path.exists() {
                        let mut success = false;
                        
                        // Try to preview with the new preview panel if available
                        if let Some(ref panel_ref) = preview_panel_clone {
                            if let Ok(mut panel) = panel_ref.lock() {
                                if panel.preview_file(&path) {
                                    println!("Successfully previewed file with new preview panel");
                                    success = true;
                                }
                            }
                        }
                        
                        // If new preview failed or is not available, try with the old image view
                        // but only if it's an image file
                        if !success && FileBrowserPanel::is_image_file(&path) {
                            if let Ok(mut view) = image_view_clone.lock() {
                                if view.load_image(&path) {
                                    println!("Successfully loaded image with old image view");
                                }
                            }
                        }
                    } else {
                        // File doesn't exist locally, need to download for preview
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
                                        
                                        let mut success = false;
                                        
                                        // Try to preview with the new preview panel if available
                                        if let Some(ref panel_ref) = preview_panel_clone {
                                            if let Ok(mut panel) = panel_ref.lock() {
                                                if panel.preview_file(&temp_file) {
                                                    println!("Successfully previewed downloaded file with new preview panel");
                                                    success = true;
                                                }
                                            }
                                        }
                                        
                                        // If new preview failed or is not available, try with the old image view
                                        // but only if it's an image file
                                        if !success && FileBrowserPanel::is_image_file(&temp_file) {
                                            if let Ok(mut view) = image_view_clone.lock() {
                                                if view.load_image(&temp_file) {
                                                    println!("Successfully loaded downloaded image with old image view");
                                                }
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