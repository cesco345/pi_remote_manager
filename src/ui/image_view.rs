// ui/image_view.rs - Image view panel
pub mod image_view {
    use fltk::{
        enums::{Color, FrameType},
        group::Group,
        image::{JpegImage, PngImage},
        prelude::*,
    };
    
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    
    pub struct ImageViewPanel {
        group: Group,
        display: fltk::frame::Frame,
        current_image: Arc<Mutex<Option<PathBuf>>>,
    }
    
    impl Clone for ImageViewPanel {
        fn clone(&self) -> Self {
            Self {
                group: self.group.clone(),
                display: self.display.clone(),
                current_image: self.current_image.clone(),
            }
        }
    }
    
    impl ImageViewPanel {
        pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
            let mut group = Group::new(x, y, w, h, None);
            group.set_frame(FrameType::BorderBox);
            
            // Add image display area
            let padding = 5;
            let display_x = x + padding;
            let display_y = y + padding;
            let display_w = w - 2 * padding;
            let display_h = h - 2 * padding;
            
            let mut display = fltk::frame::Frame::new(
                display_x,
                display_y,
                display_w,
                display_h,
                None
            );
            display.set_frame(FrameType::BorderFrame);
            display.set_color(Color::from_rgb(240, 240, 240));
            
            group.end();
            
            ImageViewPanel {
                group,
                display,
                current_image: Arc::new(Mutex::new(None)),
            }
        }
        
        pub fn load_image(&mut self, path: &Path) -> bool {
            if !path.exists() {
                return false;
            }
            
            // Clear any previous image first
            self.clear();
            
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();
                
            let result = match extension.as_str() {
                "jpg" | "jpeg" => self.load_jpeg(path),
                "png" => self.load_png(path),
                // Add more formats as needed
                _ => false,
            };
            
            if result {
                // Store the current image path
                let mut current = self.current_image.lock().unwrap();
                *current = Some(path.to_path_buf());
                println!("Successfully loaded image: {}", path.display());
            } else {
                println!("Failed to load image: {}", path.display());
            }
            
            // Force a redraw of the entire component
            self.group.redraw();
            
            result
        }
        
        fn load_jpeg(&mut self, path: &Path) -> bool {
            if let Ok(mut img) = JpegImage::load(path) {
                // Scale image to fit display
                self.scale_and_set_image(&mut img);
                true
            } else {
                false
            }
        }
        
        fn load_png(&mut self, path: &Path) -> bool {
            if let Ok(mut img) = PngImage::load(path) {
                // Scale image to fit display
                self.scale_and_set_image(&mut img);
                true
            } else {
                false
            }
        }
        
        fn scale_and_set_image<I: ImageExt + Clone>(&mut self, img: &mut I) {
            // Clear any existing image first
            self.display.set_image::<I>(None);
            
            // Reset the background 
            self.display.set_color(Color::from_rgb(240, 240, 240));
            
            // Get display dimensions
            let display_w = self.display.width();
            let display_h = self.display.height();
            
            // Get image dimensions
            let img_w = img.width();
            let img_h = img.height();
            
            // Calculate scale factor to fit image in display
            let scale_w = display_w as f64 / img_w as f64;
            let scale_h = display_h as f64 / img_h as f64;
            let scale = scale_w.min(scale_h);
            
            // Scale image to fit display (whether smaller or larger)
            let new_w = (img_w as f64 * scale) as i32;
            let new_h = (img_h as f64 * scale) as i32;
            img.scale(new_w, new_h, true, true);
            
            // Set image to display
            self.display.set_image(Some(img.clone()));
            
            // Force complete redraw
            self.display.redraw();
            
            // Make sure the parent is also redrawn if it exists
            if let Some(mut parent) = self.display.parent() {
                // We can't modify parent, just request a redraw
                parent.redraw();
            }
        }
        
        pub fn get_current_image(&self) -> Option<PathBuf> {
            let current = self.current_image.lock().unwrap();
            current.clone()
        }
        
        pub fn clear(&mut self) {
            // Clear the image
            self.display.set_image::<PngImage>(None);
            
            // Reset color to original
            self.display.set_color(Color::from_rgb(240, 240, 240));
            
            // Clear the path reference
            let mut current = self.current_image.lock().unwrap();
            *current = None;
            
            // Force a redraw
            self.display.redraw();
            self.group.redraw();
        }
    }
}