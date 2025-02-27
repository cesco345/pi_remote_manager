pub mod error;
pub mod image_utils;

// Re-export the types needed by other modules
pub use error::{
    AppError,
    AppResult,
    log_error
};

pub use image_utils::{
    is_image_file,
    get_image_format,
    find_images_in_dir,
    generate_output_filename
};