mod ui;
mod core;
mod transfer;
mod config;

use fltk::app;
use crate::ui::main_window::main_window::MainWindow;
use crate::config::Config;

fn main() {
    // Initialize the FLTK application
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    
    // Load application configuration
    let config = Config::load().unwrap_or_else(|err| {
        eprintln!("Warning: Failed to load config ({}), using defaults", err);
        Config::default()
    });
    
    // Create the main application window
    let mut main_window = MainWindow::new(
        "Pi Image Processor", 
        config.window_width,
        config.window_height
    );
    
    // Show the window and enter the application main loop
    main_window.show();
    
    // Run the application
    app.run().unwrap();
    
    // Save configuration on exit
    if let Err(err) = config.save() {
        eprintln!("Warning: Failed to save config: {}", err);
    }
}