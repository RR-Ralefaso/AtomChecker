use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtomTheme {
    OneDark,
    OneLight,
    SolarizedDark,
    SolarizedLight,
    Monokai,
    Dracula,
    GruvboxDark,
    Nord,
}

impl AtomTheme {
    pub fn all() -> Vec<AtomTheme> {
        vec![
            AtomTheme::OneDark,
            AtomTheme::OneLight,
            AtomTheme::SolarizedDark,
            AtomTheme::SolarizedLight,
            AtomTheme::Monokai,
            AtomTheme::Dracula,
            AtomTheme::GruvboxDark,
            AtomTheme::Nord,
        ]
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            AtomTheme::OneDark => "One Dark",
            AtomTheme::OneLight => "One Light",
            AtomTheme::SolarizedDark => "Solarized Dark",
            AtomTheme::SolarizedLight => "Solarized Light",
            AtomTheme::Monokai => "Monokai",
            AtomTheme::Dracula => "Dracula",
            AtomTheme::GruvboxDark => "Gruvbox Dark",
            AtomTheme::Nord => "Nord",
        }
    }
    
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = match self {
            AtomTheme::OneDark => egui::Visuals::dark(),
            AtomTheme::OneLight => egui::Visuals::light(),
            AtomTheme::SolarizedDark => solarized_dark(),
            AtomTheme::SolarizedLight => solarized_light(),
            AtomTheme::Monokai => monokai(),
            AtomTheme::Dracula => dracula(),
            AtomTheme::GruvboxDark => gruvbox_dark(),
            AtomTheme::Nord => nord(),
        };
        
        match self {
            AtomTheme::SolarizedDark => {
                visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(131, 148, 150);
                visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(131, 148, 150);
                visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(131, 148, 150);
                visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(147, 161, 161);
            }
            AtomTheme::SolarizedLight => {
                visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(101, 123, 131);
                visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(101, 123, 131);
                visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(101, 123, 131);
                visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(88, 110, 117);
            }
            AtomTheme::Monokai => {
                visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(248, 248, 242);
                visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(248, 248, 242);
                visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(248, 248, 242);
                visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(249, 238, 152);
            }
            AtomTheme::Dracula => {
                visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(248, 248, 242);
                visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(248, 248, 242);
                visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(248, 248, 242);
                visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(139, 233, 253);
            }
            AtomTheme::GruvboxDark => {
                visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(235, 219, 178);
                visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(235, 219, 178);
                visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(235, 219, 178);
                visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(251, 241, 199);
            }
            AtomTheme::Nord => {
                visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(236, 239, 244);
                visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(236, 239, 244);
                visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(236, 239, 244);
                visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(216, 222, 233);
            }
            _ => {}
        }
        
        ctx.set_visuals(visuals);
    }
    
    pub fn is_dark(&self) -> bool {
        match self {
            AtomTheme::OneDark | AtomTheme::SolarizedDark | 
            AtomTheme::Monokai | AtomTheme::Dracula |
            AtomTheme::GruvboxDark | AtomTheme::Nord => true,
            AtomTheme::OneLight | AtomTheme::SolarizedLight => false,
        }
    }
}

fn solarized_dark() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(0, 43, 54);
    visuals.panel_fill = egui::Color32::from_rgb(7, 54, 66);
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(88, 110, 117));
    visuals.faint_bg_color = egui::Color32::from_rgb(88, 110, 117);
    visuals.extreme_bg_color = egui::Color32::from_rgb(0, 43, 54);
    visuals.code_bg_color = egui::Color32::from_rgb(7, 54, 66);
    visuals.warn_fg_color = egui::Color32::from_rgb(181, 137, 0);
    visuals.error_fg_color = egui::Color32::from_rgb(220, 50, 47);
    visuals.hyperlink_color = egui::Color32::from_rgb(38, 139, 210);
    visuals
}

fn solarized_light() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = egui::Color32::from_rgb(253, 246, 227);
    visuals.panel_fill = egui::Color32::from_rgb(238, 232, 213);
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(147, 161, 161));
    visuals.faint_bg_color = egui::Color32::from_rgb(147, 161, 161);
    visuals.extreme_bg_color = egui::Color32::from_rgb(253, 246, 227);
    visuals.code_bg_color = egui::Color32::from_rgb(238, 232, 213);
    visuals.warn_fg_color = egui::Color32::from_rgb(181, 137, 0);
    visuals.error_fg_color = egui::Color32::from_rgb(220, 50, 47);
    visuals.hyperlink_color = egui::Color32::from_rgb(38, 139, 210);
    visuals
}

fn monokai() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(39, 40, 34);
    visuals.panel_fill = egui::Color32::from_rgb(39, 40, 34);
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(73, 72, 62));
    visuals.faint_bg_color = egui::Color32::from_rgb(73, 72, 62);
    visuals.extreme_bg_color = egui::Color32::from_rgb(39, 40, 34);
    visuals.code_bg_color = egui::Color32::from_rgb(73, 72, 62);
    visuals.warn_fg_color = egui::Color32::from_rgb(249, 238, 152);
    visuals.error_fg_color = egui::Color32::from_rgb(249, 38, 114);
    visuals.hyperlink_color = egui::Color32::from_rgb(102, 217, 239);
    visuals
}

fn dracula() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(40, 42, 54);
    visuals.panel_fill = egui::Color32::from_rgb(40, 42, 54);
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(68, 71, 90));
    visuals.faint_bg_color = egui::Color32::from_rgb(68, 71, 90);
    visuals.extreme_bg_color = egui::Color32::from_rgb(40, 42, 54);
    visuals.code_bg_color = egui::Color32::from_rgb(68, 71, 90);
    visuals.warn_fg_color = egui::Color32::from_rgb(241, 250, 140);
    visuals.error_fg_color = egui::Color32::from_rgb(255, 85, 85);
    visuals.hyperlink_color = egui::Color32::from_rgb(139, 233, 253);
    visuals
}

fn gruvbox_dark() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(40, 40, 40);
    visuals.panel_fill = egui::Color32::from_rgb(60, 56, 54);
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(124, 111, 100));
    visuals.faint_bg_color = egui::Color32::from_rgb(124, 111, 100);
    visuals.extreme_bg_color = egui::Color32::from_rgb(40, 40, 40);
    visuals.code_bg_color = egui::Color32::from_rgb(60, 56, 54);
    visuals.warn_fg_color = egui::Color32::from_rgb(215, 153, 33);
    visuals.error_fg_color = egui::Color32::from_rgb(204, 36, 29);
    visuals.hyperlink_color = egui::Color32::from_rgb(69, 133, 136);
    visuals
}

fn nord() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(46, 52, 64);
    visuals.panel_fill = egui::Color32::from_rgb(59, 66, 82);
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(76, 86, 106));
    visuals.faint_bg_color = egui::Color32::from_rgb(76, 86, 106);
    visuals.extreme_bg_color = egui::Color32::from_rgb(46, 52, 64);
    visuals.code_bg_color = egui::Color32::from_rgb(59, 66, 82);
    visuals.warn_fg_color = egui::Color32::from_rgb(235, 203, 139);
    visuals.error_fg_color = egui::Color32::from_rgb(191, 97, 106);
    visuals.hyperlink_color = egui::Color32::from_rgb(136, 192, 208);
    visuals
}