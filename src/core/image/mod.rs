pub mod processor;
pub mod operations;

// Re-export the types needed by other modules
pub use processor::{
    ImageFormat,
    ImageProcessor,
    ImageProcessorFactory,
    ImageProcessingService,
    ProcessingError,
    JPEGProcessor,
    JPEGProcessorFactory,
    PNGProcessor,
    PNGProcessorFactory
};

pub use operations::{
    ImageOperation,
    OperationError,
    ResizeOperation,
    BrightnessOperation
};