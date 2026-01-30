#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use spellchecker::gui::SpellCheckerApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("AtomSpell - IDE-Inspired Multilingual Spell Checker")
            .with_icon(
                load_icon().expect("Failed to load app icon"),
            ),
        centered: true,
        ..Default::default()
    };
    
    eframe::run_native(
        "AtomSpell",
        options,
        Box::new(|cc| {
            // Set dark theme by default
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            // Load custom font from assets
            let mut fonts = egui::FontDefinitions::default();
            
            // Load FiraCode font from assets
            let font_paths = [
                "assets/fonts/FiraCode-Regular.ttf",
                "./assets/fonts/FiraCode-Regular.ttf",
                "../assets/fonts/FiraCode-Regular.ttf",
                "../../assets/fonts/FiraCode-Regular.ttf",
            ];
            
            for font_path in font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "FiraCode".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    
                    // Add to monospace family
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                        family.insert(0, "FiraCode".to_owned());
                    } else {
                        fonts.families.insert(
                            egui::FontFamily::Monospace,
                            vec!["FiraCode".to_owned()]
                        );
                    }
                    
                    // Also add to proportional family for UI elements
                    fonts.families.entry(egui::FontFamily::Proportional)
                        .or_insert_with(Vec::new)
                        .push("FiraCode".to_owned());
                    
                    println!("Loaded FiraCode font from: {}", font_path);
                    break;
                }
            }
            
            // If font not found, use system default but with better fallback
            if !fonts.font_data.contains_key("FiraCode") {
                println!("Warning: FiraCode font not found. Using system default.");
                // Add a better default monospace font
                fonts.families.insert(
                    egui::FontFamily::Monospace,
                    vec!["Monospace".to_owned(), "Courier New".to_owned()]
                );
            }
            
            cc.egui_ctx.set_fonts(fonts);
            
            // Set default font size
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            );
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            );
            cc.egui_ctx.set_style(style);
            
            Box::new(SpellCheckerApp::new(cc))
        }),
    )
}

fn load_icon() -> Result<eframe::IconData, Box<dyn std::error::Error>> {
    let icon_paths = [
        "assets/icons/icon.png",
        "./assets/icons/icon.png",
        "../assets/icons/icon.png",
        "../../assets/icons/icon.png",
        "icon.png",
    ];
    
    for path in icon_paths {
        if let Ok(image) = image::open(path) {
            let image = image.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            
            println!("Loaded icon from: {}", path);
            return Ok(eframe::IconData {
                rgba,
                width,
                height,
            });
        }
    }
    
    // Fallback: create a simple icon programmatically
    println!("Warning: icon.png not found. Using default icon.");
    let width = 256;
    let height = 256;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 / width as f32 - 0.5;
            let dy = y as f32 / height as f32 - 0.5;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < 0.45 {
                // Blue circle with 'A' in the center
                let is_a = x > width / 3 && x < 2 * width / 3 && 
                          y > height / 3 && y < 2 * height / 3;
                
                if is_a {
                    rgba.extend_from_slice(&[255, 255, 255, 255]); // White for 'A'
                } else {
                    rgba.extend_from_slice(&[0, 122, 204, 255]); // Blue background
                }
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]); // Transparent
            }
        }
    }
    
    Ok(eframe::IconData {
        rgba,
        width,
        height,
    })
}