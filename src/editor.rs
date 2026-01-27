use crate::checker::{DocumentAnalysis, WordCheck};
use eframe::egui;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TextEditor {
    line_height: f32,
    font_size: f32,
    show_whitespace: bool,
    wrap_lines: bool,
    error_cache: HashMap<usize, WordCheck>,
    last_analysis: Option<DocumentAnalysis>,
    programming_language: Option<String>,
    syntax_highlighting: bool,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            line_height: 24.0,
            font_size: 14.0,
            show_whitespace: false,
            wrap_lines: true,
            error_cache: HashMap::new(),
            last_analysis: None,
            programming_language: None,
            syntax_highlighting: true,
        }
    }
    
    pub fn set_analysis(&mut self, analysis: DocumentAnalysis) {
        self.last_analysis = Some(analysis.clone());
        
        // Build error cache for quick lookup
        self.error_cache.clear();
        for word in &analysis.words {
            if !word.is_correct {
                self.error_cache.insert(word.start, word.clone());
            }
        }
    }
    
    pub fn detect_programming_language(&mut self, filename: &str) {
        self.programming_language = match filename.rsplit('.').next() {
            Some("rs") => Some("rust".to_string()),
            Some("py") => Some("python".to_string()),
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => Some("javascript".to_string()),
            Some("java") => Some("java".to_string()),
            Some("cpp") | Some("cc") | Some("cxx") | Some("c") => Some("cpp".to_string()),
            Some("go") => Some("go".to_string()),
            Some("rb") => Some("ruby".to_string()),
            Some("php") => Some("php".to_string()),
            Some("html") | Some("htm") => Some("html".to_string()),
            Some("css") => Some("css".to_string()),
            Some("md") => Some("markdown".to_string()),
            Some("json") => Some("json".to_string()),
            Some("toml") => Some("toml".to_string()),
            Some("yaml") | Some("yml") => Some("yaml".to_string()),
            _ => None,
        };
    }
    
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        content: &mut String,
        modified: &mut bool,
        show_line_numbers: bool,
        analysis: &Option<DocumentAnalysis>,
    ) -> egui::Response {
        // Update analysis if provided
        if let Some(analysis) = analysis {
            self.set_analysis(analysis.clone());
        }
        
        let available_rect = ui.available_rect_before_wrap();
        
        // Calculate line numbers width if needed
        let line_numbers_width = if show_line_numbers {
            let line_count = content.lines().count().max(1);
            let max_digits = line_count.to_string().len();
            (max_digits as f32 * self.font_size * 0.55) + 20.0
        } else {
            0.0
        };
        
        // Create a custom widget for the editor
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_rect.width(), available_rect.height()),
            egui::Sense::click_and_drag(),
        );
        
        // Draw the editor background
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(2.0),
            ui.visuals().window_fill,
        );
        
        // Draw line numbers if enabled
        if show_line_numbers {
            self.draw_line_numbers(ui, rect, content);
        }
        
        // Draw text content with error highlighting
        self.draw_text_with_errors(ui, rect, content, line_numbers_width);
        
        // Add a text edit widget for editing
        let text_edit_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + line_numbers_width, rect.top()),
            egui::vec2(rect.width() - line_numbers_width, rect.height()),
        );
        
        ui.allocate_ui_at_rect(text_edit_rect, |ui| {
            // Make text edit background transparent
            let visuals = ui.visuals().clone();
            ui.set_visuals(visuals);
            
            let mut text_edit = egui::TextEdit::multiline(content)
                .desired_width(f32::INFINITY)
                .desired_rows(10)
                .font(egui::FontId::monospace(self.font_size))
                .frame(false)
                .text_color(ui.visuals().text_color());
            
            if self.wrap_lines {
                text_edit = text_edit.desired_rows(10);
            }
            
            let edit_response = ui.add(text_edit);
            if edit_response.changed() {
                *modified = true;
            }
            
            // Handle right-click for suggestions
            if edit_response.context_menu(|ui| {
                self.show_context_menu(ui, content, &edit_response)
            }).response.clicked_elsewhere() {
                // Close context menu
            }
            
            edit_response
        }).inner
    }
    
    fn show_context_menu(&self, ui: &mut egui::Ui, content: &str, response: &egui::Response) -> bool {
        let mut handled = false;
        
        if let Some(cursor_pos) = response.hover_pos() {
            // Convert cursor position to text index
            // This is simplified - in a real implementation, you'd need to map
            // screen coordinates to text positions
            
            ui.label("Spelling suggestions would appear here");
            ui.separator();
            
            if ui.button("Add to dictionary").clicked() {
                handled = true;
            }
            if ui.button("Ignore word").clicked() {
                handled = true;
            }
        }
        
        handled
    }
    
    fn draw_line_numbers(&self, ui: &egui::Ui, rect: egui::Rect, content: &str) {
        let painter = ui.painter();
        let line_count = content.lines().count().max(1);
        let line_number_color = ui.visuals().weak_text_color().gamma_multiply(0.7);
        
        for i in 0..line_count {
            let line_y = rect.top() + (i as f32 * self.line_height) + (self.line_height * 0.7);
            let line_num = (i + 1).to_string();
            
            painter.text(
                egui::pos2(rect.left() + 8.0, line_y),
                egui::Align2::RIGHT_CENTER,
                line_num,
                egui::FontId::monospace(self.font_size * 0.9),
                line_number_color,
            );
            
            // Draw subtle separator line
            if i < line_count - 1 {
                painter.line_segment(
                    [
                        egui::pos2(rect.left() + line_numbers_width(rect, content) - 5.0, line_y + self.line_height * 0.5),
                        egui::pos2(rect.right(), line_y + self.line_height * 0.5),
                    ],
                    egui::Stroke::new(0.5, ui.visuals().window_stroke.color.gamma_multiply(0.3)),
                );
            }
        }
        
        // Draw line number background
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.left() + line_numbers_width(rect, content), rect.bottom()),
            ),
            0.0,
            ui.visuals().faint_bg_color,
        );
    }
    
    fn draw_text_with_errors(&self, ui: &egui::Ui, rect: egui::Rect, content: &str, line_numbers_width: f32) {
        let painter = ui.painter();
        let lines: Vec<&str> = content.lines().collect();
        let text_color = ui.visuals().text_color();
        let error_color = ui.visuals().error_fg_color;
        let warning_color = egui::Color32::from_rgb(255, 165, 0); // Orange for warnings
        
        // Calculate character width for positioning
        let char_width = self.font_size * 0.6;
        
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = rect.top() + (line_idx as f32 * self.line_height);
            let text_x = rect.left() + line_numbers_width + 8.0;
            
            // Apply syntax highlighting for programming languages
            if let Some(lang) = &self.programming_language {
                self.highlight_syntax(painter, line, text_x, line_y, lang);
            } else {
                // Draw regular text
                painter.text(
                    egui::pos2(text_x, line_y + (self.line_height * 0.7)),
                    egui::Align2::LEFT_CENTER,
                    *line,
                    egui::FontId::monospace(self.font_size),
                    text_color,
                );
            }
            
            // Draw error highlights for this line
            if let Some(analysis) = &self.last_analysis {
                let line_num = line_idx + 1;
                let line_errors: Vec<&WordCheck> = analysis.words
                    .iter()
                    .filter(|w| !w.is_correct && w.line == line_num)
                    .collect();
                
                for error in line_errors {
                    // Calculate position of error in the line
                    let error_start_in_line = error.column.saturating_sub(1);
                    let error_x = text_x + (error_start_in_line as f32 * char_width);
                    let error_width = error.word.len() as f32 * char_width;
                    
                    // Draw wavy underline for the error
                    self.draw_wavy_underline(
                        painter,
                        error_x,
                        line_y + self.line_height - 2.0,
                        error_width,
                        error_color,
                    );
                }
            }
        }
    }
    
    fn highlight_syntax(&self, painter: &egui::Painter, line: &str, x: f32, y: f32, language: &str) {
        let char_width = self.font_size * 0.6;
        let mut current_x = x;
        
        // Simple keyword highlighting (can be expanded)
        let keywords = match language {
            "rust" => vec!["fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait", "match", "if", "else", "for", "while", "loop", "return", "break", "continue", "self", "super", "crate"],
            "python" => vec!["def", "class", "import", "from", "as", "if", "elif", "else", "for", "while", "try", "except", "finally", "with", "as", "pass", "return", "yield", "lambda", "True", "False", "None"],
            "javascript" => vec!["function", "const", "let", "var", "if", "else", "for", "while", "return", "class", "import", "export", "from", "async", "await", "try", "catch", "finally", "throw"],
            "java" => vec!["public", "private", "protected", "class", "interface", "extends", "implements", "static", "void", "int", "String", "boolean", "if", "else", "for", "while", "return", "try", "catch", "finally", "new"],
            _ => vec![],
        };
        
        let words: Vec<&str> = line.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_').collect();
        
        for word in words {
            let trimmed = word.trim();
            let word_color = if keywords.contains(&trimmed) {
                egui::Color32::from_rgb(86, 156, 214) // Blue for keywords
            } else if trimmed.starts_with('"') || trimmed.starts_with('\'') {
                egui::Color32::from_rgb(206, 145, 120) // Orange for strings
            } else if trimmed.chars().all(|c| c.is_numeric()) {
                egui::Color32::from_rgb(181, 206, 168) // Green for numbers
            } else if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                egui::Color32::from_rgb(87, 166, 74) // Green for comments
            } else {
                ui.visuals().text_color()
            };
            
            painter.text(
                egui::pos2(current_x, y + (self.line_height * 0.7)),
                egui::Align2::LEFT_CENTER,
                word,
                egui::FontId::monospace(self.font_size),
                word_color,
            );
            
            current_x += word.len() as f32 * char_width;
        }
    }
    
    fn draw_wavy_underline(
        &self,
        painter: &egui::Painter,
        x: f32,
        y: f32,
        width: f32,
        color: egui::Color32,
    ) {
        let wave_height = 1.5;
        let wave_length = 4.0;
        let segments = (width / wave_length).ceil() as usize;
        
        for i in 0..segments {
            let segment_x = x + i as f32 * wave_length;
            let segment_end_x = (segment_x + wave_length).min(x + width);
            let segment_width = segment_end_x - segment_x;
            
            if segment_width > 0.0 {
                let y_offset = if i % 2 == 0 { 0.0 } else { wave_height };
                
                painter.line_segment(
                    [
                        egui::pos2(segment_x, y + y_offset),
                        egui::pos2(segment_end_x, y + y_offset),
                    ],
                    egui::Stroke::new(1.0, color),
                );
            }
        }
    }
    
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.max(8.0).min(36.0);
        self.line_height = size * 1.6;
    }
    
    pub fn set_wrap_lines(&mut self, wrap: bool) {
        self.wrap_lines = wrap;
    }
    
    pub fn set_show_whitespace(&mut self, show: bool) {
        self.show_whitespace = show;
    }
    
    pub fn get_error_at_position(&self, line: usize, column: usize) -> Option<&WordCheck> {
        if let Some(analysis) = &self.last_analysis {
            analysis.words.iter()
                .find(|w| !w.is_correct && w.line == line && w.column <= column && column <= w.column + w.word.len())
        } else {
            None
        }
    }
}

fn line_numbers_width(rect: egui::Rect, content: &str) -> f32 {
    let line_count = content.lines().count().max(1);
    let max_digits = line_count.to_string().len();
    (max_digits as f32 * 14.0 * 0.55) + 20.0
}