#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use spellchecker::gui::SpellCheckerApp;

fn main() -> Result<(), eframe::Error> {
    // Set up native options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("AtomSpell - IDE-Inspired Multilingual Spell Checker")
            .with_active(true)
            .with_resizable(true),
        centered: true,
        default_theme: eframe::Theme::Dark,
        run_and_return: false,
        renderer: eframe::Renderer::default(),
        follow_system_theme: true,
        ..Default::default()
    };
    
    eframe::run_native(
        "AtomSpell",
        options,
        Box::new(|cc| {
            // Configure visuals with default dark theme
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            // Load custom font - FIXED: No shadows, proper loading
            let mut fonts = egui::FontDefinitions::default();
            
            // Try multiple font locations
            let font_paths = [
                "assets/fonts/FiraCode-Regular.ttf",
                "./assets/fonts/FiraCode-Regular.ttf",
                "../assets/fonts/FiraCode-Regular.ttf",
                "FiraCode-Regular.ttf"
            ];
            
            let mut font_loaded = false;
            for font_path in font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "FiraCode".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    
                    // Replace monospace family entirely with FiraCode
                    fonts.families.insert(
                        egui::FontFamily::Monospace,
                        vec!["FiraCode".to_owned()]
                    );
                    
                    // Also add to proportional for consistent UI
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        family.insert(0, "FiraCode".to_owned());
                    }
                    
                    font_loaded = true;
                    println!("Loaded FiraCode font from: {}", font_path);
                    break;
                }
            }
            
            if !font_loaded {
                eprintln!("Warning: Could not load FiraCode font. Using default monospace font.");
                // Add a default monospace fallback
                fonts.families.insert(
                    egui::FontFamily::Monospace,
                    vec!["Hack".to_owned(), "Consolas".to_owned(), "Monospace".to_owned()]
                );
            }
            
            cc.egui_ctx.set_fonts(fonts);
            
            // Create and return the app
            Box::new(SpellCheckerApp::new(cc))
        }),
    )
}