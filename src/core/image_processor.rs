// core/image_processor.rs - Image processor implementation
pub mod image_processor {
    use std::path::Path;
    use std::error::Error;
    use std::fmt;
    
    use crate::core::operations::operations::{ImageOperation, OperationError};

    // Define image format types
    #[derive(Debug, Clone, PartialEq)]
    pub enum ImageFormat {
        JPEG,
        PNG,
        GIF,
        BMP,
        TIFF,
        WebP,
        Unknown,
    }
    
    impl ImageFormat {
        pub fn from_extension(ext: &str) -> Self {
            match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => Self::JPEG,
                "png" => Self::PNG,
                "gif" => Self::GIF,
                "bmp" => Self::BMP,
                "tiff" | "tif" => Self::TIFF,
                "webp" => Self::WebP,
                _ => Self::Unknown,
            }
        }
        
        pub fn extension(&self) -> &'static str {
            match self {
                Self::JPEG => "jpg",
                Self::PNG => "png",
                Self::GIF => "gif",
                Self::BMP => "bmp",
                Self::TIFF => "tiff",
                Self::WebP => "webp",
                Self::Unknown => "",
            }
        }
    }

    // Image processor trait - this is the "Product" in our Factory Method pattern
    pub trait ImageProcessor {
        fn process_image(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>>;
        fn get_name(&self) -> &str;
        fn get_format(&self) -> ImageFormat;
        fn get_description(&self) -> String;
    }

    // Concrete image processors
    pub struct JPEGProcessor {
        quality: u8,
    }
    
    impl JPEGProcessor {
        pub fn new(quality: u8) -> Self {
            Self { 
                quality: quality.min(100),
            }
        }
    }
    
    impl ImageProcessor for JPEGProcessor {
        fn process_image(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
            // This would use a real image processing library
            println!("Processing JPEG: {} -> {}", input_path.display(), output_path.display());
            println!("Using quality setting: {}", self.quality);
            
            // Simulate processing
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            Ok(())
        }
        
        fn get_name(&self) -> &str {
            "JPEG Processor"
        }
        
        fn get_format(&self) -> ImageFormat {
            ImageFormat::JPEG
        }
        
        fn get_description(&self) -> String {
            format!("JPEG image processor (Quality: {}%)", self.quality)
        }
    }

    pub struct PNGProcessor {
        compression_level: u8,
    }
    
    impl PNGProcessor {
        pub fn new(compression_level: u8) -> Self {
            Self { 
                compression_level: compression_level.min(9),
            }
        }
    }
    
    impl ImageProcessor for PNGProcessor {
        fn process_image(&self, input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
            println!("Processing PNG: {} -> {}", input_path.display(), output_path.display());
            println!("Using compression level: {}", self.compression_level);
            
            // Simulate processing
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            Ok(())
        }
        
        fn get_name(&self) -> &str {
            "PNG Processor"
        }
        
        fn get_format(&self) -> ImageFormat {
            ImageFormat::PNG
        }
        
        fn get_description(&self) -> String {
            format!("PNG image processor (Compression: {})", self.compression_level)
        }
    }

    // Additional processor types for other formats would go here

    // ImageProcessorFactory trait - this is the "Creator" in our Factory Method pattern
    pub trait ImageProcessorFactory {
        fn create_processor(&self) -> Box<dyn ImageProcessor>;
        fn get_name(&self) -> String;
    }

    // Concrete factories for each image processor type
    pub struct JPEGProcessorFactory {
        quality: u8,
    }
    
    impl JPEGProcessorFactory {
        pub fn new(quality: u8) -> Self {
            Self { quality }
        }
    }
    
    impl ImageProcessorFactory for JPEGProcessorFactory {
        fn create_processor(&self) -> Box<dyn ImageProcessor> {
            Box::new(JPEGProcessor::new(self.quality))
        }
        
        fn get_name(&self) -> String {
            format!("JPEG Processor (Quality: {}%)", self.quality)
        }
    }

    pub struct PNGProcessorFactory {
        compression_level: u8,
    }
    
    impl PNGProcessorFactory {
        pub fn new(compression_level: u8) -> Self {
            Self { compression_level }
        }
    }
    
    impl ImageProcessorFactory for PNGProcessorFactory {
        fn create_processor(&self) -> Box<dyn ImageProcessor> {
            Box::new(PNGProcessor::new(self.compression_level))
        }
        
        fn get_name(&self) -> String {
            format!("PNG Processor (Compression: {})", self.compression_level)
        }
    }

    // Image processing service that manages processors and applies operations
    pub struct ImageProcessingService {
        factories: Vec<Box<dyn ImageProcessorFactory>>,
        operations: Vec<Box<dyn ImageOperation>>,
    }
    
    impl ImageProcessingService {
        pub fn new() -> Self {
            Self {
                factories: Vec::new(),
                operations: Vec::new(),
            }
        }
        
        pub fn register_factory(&mut self, factory: Box<dyn ImageProcessorFactory>) {
            self.factories.push(factory);
        }
        
        pub fn add_operation(&mut self, operation: Box<dyn ImageOperation>) {
            self.operations.push(operation);
        }
        
        pub fn clear_operations(&mut self) {
            self.operations.clear();
        }
        
        pub fn get_operations(&self) -> &[Box<dyn ImageOperation>] {
            &self.operations
        }
        
        pub fn get_factories(&self) -> &[Box<dyn ImageProcessorFactory>] {
            &self.factories
        }
        
        pub fn process_image(
            &self, 
            input_path: &Path, 
            output_path: &Path, 
            factory_index: usize
        ) -> Result<(), ProcessingError> {
            if factory_index >= self.factories.len() {
                return Err(ProcessingError::NoProcessorAvailable);
            }
            
            let factory = &self.factories[factory_index];
            let processor = factory.create_processor();
            
            // Apply operations
            for operation in &self.operations {
                if let Err(err) = operation.apply(input_path) {
                    return Err(ProcessingError::OperationFailed(err));
                }
            }
            
            // Process the image
            processor.process_image(input_path, output_path)
                .map_err(|e| ProcessingError::ProcessingFailed(e.to_string()))
        }
    }
    
    // Error type for image processing
    #[derive(Debug)]
    pub enum ProcessingError {
        NoProcessorAvailable,
        OperationFailed(OperationError),
        ProcessingFailed(String),
    }
    
    impl fmt::Display for ProcessingError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NoProcessorAvailable => write!(f, "No suitable image processor available"),
                Self::OperationFailed(err) => write!(f, "Operation failed: {}", err),
                Self::ProcessingFailed(msg) => write!(f, "Processing failed: {}", msg),
            }
        }
    }
    
    impl Error for ProcessingError {}
}