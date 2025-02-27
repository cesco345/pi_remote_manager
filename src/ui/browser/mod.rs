pub mod file_browser;
pub mod remote_browser;

// Re-export the main file browser for compatibility
pub use crate::ui::file_browser::file_browser::FileBrowserPanel;