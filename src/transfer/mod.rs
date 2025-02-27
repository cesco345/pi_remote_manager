pub mod method;
pub mod ssh;
pub mod rsync;

// Re-export the types needed by other modules
pub use method::{TransferMethod, TransferMethodFactory, TransferError};
pub use ssh::{SSHTransfer, SSHTransferFactory};
pub use rsync::{RsyncTransfer, RsyncTransferFactory};
