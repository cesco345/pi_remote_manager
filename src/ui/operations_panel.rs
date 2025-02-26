// ui/operations_panel.rs - Image operations panel
pub mod operations_panel {
    use fltk::{
        browser::MultiBrowser,
        button::Button,
        enums::{Color, FrameType},
        group::Group,
        prelude::*,
    };
    
    use std::sync::{Arc, Mutex};
    
    use crate::core::image_processor::image_processor::{
        ImageProcessingService,
        ImageProcessor,
        ImageProcessorFactory,
    };
    use crate::core::operations::operations::{
        ImageOperation,
        ResizeOperation,
        BrightnessOperation,
    };
    
    use crate::ui::dialogs::dialogs;
    
    pub struct OperationsPanel {
        group: Group,
        processor_browser: MultiBrowser,
        operations_browser: MultiBrowser,
        add_operation_button: Button,
        apply_button: Button,
        clear_button: Button,
        image_service: Arc<Mutex<ImageProcessingService>>,
    }
    
    impl OperationsPanel {
        pub fn new(
            x: i32, 
            y: i32, 
            w: i32, 
            h: i32,
            image_service: Arc<Mutex<ImageProcessingService>>
        ) -> Self {
            let mut group = Group::new(x, y, w, h, None);
            group.set_frame(FrameType::BorderBox);
            
            // Add panel components
            let padding = 10;
            let button_height = 30;
            let browser_height = (h - 4 * padding - 2 * button_height) / 2;
            
            // Processor selection section
            let mut processor_label = fltk::frame::Frame::new(
                x + padding, 
                y + padding, 
                w - 2 * padding, 
                20, 
                "Image Processors:"
            );
            processor_label.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);
            
            let processor_browser = MultiBrowser::new(
                x + padding,
                y + padding + 20,
                w - 2 * padding,
                browser_height,
                None
            );
            
            // Operations section
            let operations_y = y + padding + 20 + browser_height + padding;
            let mut operations_label = fltk::frame::Frame::new(
                x + padding, 
                operations_y, 
                w - 2 * padding, 
                20, 
                "Operations:"
            );
            operations_label.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);
            
            let operations_browser = MultiBrowser::new(
                x + padding,
                operations_y + 20,
                w - 2 * padding,
                browser_height,
                None
            );
            
            // Buttons section
            let buttons_y = operations_y + 20 + browser_height + padding;
            let button_width = (w - 2 * padding - 10) / 2;
            
            let add_operation_button = Button::new(
                x + padding,
                buttons_y,
                button_width,
                button_height,
                "Add Operation"
            );
            
            let clear_button = Button::new(
                x + padding + button_width + 10,
                buttons_y,
                button_width,
                button_height,
                "Clear Operations"
            );
            
            // Apply button
            let apply_y = buttons_y + button_height + padding;
            let mut apply_button = Button::new(
                x + w / 2 - 50,
                apply_y,
                100,
                button_height,
                "Apply"
            );
            apply_button.set_color(Color::from_rgb(0, 120, 255));
            apply_button.set_label_color(Color::White);
            
            group.end();
            
            let mut panel = OperationsPanel {
                group,
                processor_browser,
                operations_browser,
                add_operation_button,
                apply_button,
                clear_button,
                image_service,
            };
            
            // Initialize the panel
            panel.populate_processors();
            panel.setup_callbacks();
            
            panel
        }
        
        fn populate_processors(&mut self) {
            let service = self.image_service.lock().unwrap();
            
            self.processor_browser.clear();
            
            for (i, factory) in service.get_factories().iter().enumerate() {
                self.processor_browser.add(&format!("{}. {}", i + 1, factory.get_name()));
            }
            
            // Select the first processor by default
            if service.get_factories().len() > 0 {
                self.processor_browser.select(1);
            }
        }
        
        fn update_operations(&mut self) {
            let service = self.image_service.lock().unwrap();
            
            self.operations_browser.clear();
            
            for (i, operation) in service.get_operations().iter().enumerate() {
                self.operations_browser.add(&format!("{}. {}", i + 1, operation.get_description()));
            }
        }
        
        fn setup_callbacks(&mut self) {
            // Add operation button callback
            let image_service = self.image_service.clone();
            let mut operations_browser = self.operations_browser.clone();
            
            let mut add_button = self.add_operation_button.clone();
            add_button.set_callback(move |_| {
                // Show operation selection dialog
                let operations = [
                    "Resize",
                    "Brightness Adjustment",
                    // Add more operations as needed
                ];
                
                let choice = dialogs::choice_dialog(
                    "Select Operation",
                    "Choose an operation to add:",
                    &operations
                );
                
                match choice {
                    0 => { // Resize
                        if let Some((width, height)) = dialogs::resize_dialog() {
                            let operation = Box::new(ResizeOperation::new(width, height));
                            image_service.lock().unwrap().add_operation(operation);
                        }
                    },
                    1 => { // Brightness
                        if let Some(level) = dialogs::brightness_dialog() {
                            let operation = Box::new(BrightnessOperation::new(level));
                            image_service.lock().unwrap().add_operation(operation);
                        }
                    },
                    // Add more operation types as needed
                    _ => return,
                }
                
                // Update operations browser
                Self::update_operations_browser(&image_service, &mut operations_browser);
            });
            
            // Clear button callback
            let image_service = self.image_service.clone();
            let mut operations_browser = self.operations_browser.clone();
            
            let mut clear_button = self.clear_button.clone();
            clear_button.set_callback(move |_| {
                image_service.lock().unwrap().clear_operations();
                operations_browser.clear();
            });
            
            // Apply button callback
            let image_service = self.image_service.clone();
            let processor_browser = self.processor_browser.clone();
            
            let mut apply_button = self.apply_button.clone();
            apply_button.set_callback(move |_| {
                let selected = processor_browser.value();
                if selected <= 0 {
                    dialogs::message_dialog("Error", "Please select a processor first.");
                    return;
                }
                
                let processor_index = selected - 1;
                
                // In a real implementation, this would apply the operations to the current image
                println!("Applying operations with processor {}", processor_index);
                
                dialogs::message_dialog("Success", "Operations applied successfully.");
            });
        }
        
        fn update_operations_browser(
            image_service: &Arc<Mutex<ImageProcessingService>>,
            operations_browser: &mut MultiBrowser
        ) {
            let service = image_service.lock().unwrap();
            
            operations_browser.clear();
            
            for (i, operation) in service.get_operations().iter().enumerate() {
                operations_browser.add(&format!("{}. {}", i + 1, operation.get_description()));
            }
        }
    }
}