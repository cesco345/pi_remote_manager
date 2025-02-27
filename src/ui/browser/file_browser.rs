// This is a compatibility wrapper that re-exports the current file browser implementation
// This will allow us to gradually migrate to a new implementation

pub use crate::ui::file_browser::file_browser::FileBrowserPanel;
pub use crate::ui::file_browser::file_browser::FileEntry;

// In future versions, we'll implement a new file browser here and 
// maintain compatibility with the existing API