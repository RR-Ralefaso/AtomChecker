use crate::checker::{DocumentAnalysis, SpellChecker, WordType};
use eframe::egui;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Sidebar {
    pub show_dictionary: bool,
    pub show_errors: bool,
    pub show_stats: bool,
    pub show_find: bool,
    pub show_replace: bool,
    pub selected_error_index: usize,
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive_find: bool,
    whole_word_find: bool,
    pub visible: bool,
    pub dictionary_filter: String,
    pub show_ignored_words: bool,
    pub error_filter: ErrorFilter,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ErrorFilter {
    All,
    HighConfidence,
    CodeIdentifiers,
    ProperNouns,
    Numbers,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            show_dictionary: true,
            show_errors: false,
            show_stats: false,
            show_find: false,
            show_replace: false,
            selected_error_index: 0,
            find_text: String::new(),
            replace_text: String::new(),
            case_sensitive_find: false,
            whole_word_find: false,
            visible: true,
            dictionary_filter: String::new(),
            show_ignored_words: false,
            error_filter: ErrorFilter::All,
        }
    }
    
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        spell_checker: &SpellChecker,
        analysis: &Option<DocumentAnalysis>,
        content: &str,
        on_add_word: &mut Option<String>,
        on_ignore_word: &mut Option<String>,
        on_replace: &mut Option<(String, String)>,
        on_import_dict: &mut bool,
        on_export_dict: &mut bool,
        on_clear_ignored: &mut bool,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.show_dictionary, "📚 Dictionary").clicked() {
                    self.reset_tabs();
                    self.show_dictionary = true;
                }
                
                if ui.selectable_label(self.show_errors, "❌ Errors").clicked() {
                    self.reset_tabs();
                    self.show_errors = true;
                }
                
                if ui.selectable_label(self.show_stats, "📊 Stats").clicked() {
                    self.reset_tabs();
                    self.show_stats = true;
                }
                
                if ui.selectable_label(self.show_find, "🔍 Find").clicked() {
                    self.reset_tabs();
                    self.show_find = true;
                }
                
                if ui.selectable_label(self.show_replace, "🔄 Replace").clicked() {
                    self.reset_tabs();
                    self.show_replace = true;
                }
            });
            
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);
            
            if self.show_dictionary {
                self.show_dictionary_view(ui, spell_checker, on_add_word, on_ignore_word, 
                    on_import_dict, on_export_dict, on_clear_ignored);
            } else if self.show_errors {
                self.show_errors_view(ui, analysis, on_replace);
            } else if self.show_stats {
                self.show_stats_view(ui, analysis, spell_checker);
            } else if self.show_find {
                self.show_find_view(ui, content);
            } else if self.show_replace {
                self.show_replace_view(ui, content, on_replace);
            }
        });
    }
    
    fn reset_tabs(&mut self) {
        self.show_dictionary = false;
        self.show_errors = false;
        self.show_stats = false;
        self.show_find = false;
        self.show_replace = false;
    }
    
    fn show_dictionary_view(
        &mut self,
        ui: &mut egui::Ui,
        spell_checker: &SpellChecker,
        on_add_word: &mut Option<String>,
        on_ignore_word: &mut Option<String>,
        on_import_dict: &mut bool,
        on_export_dict: &mut bool,
        on_clear_ignored: &mut bool,
    ) {
        ui.heading("Dictionary");
        
        ui.horizontal(|ui| {
            ui.label("Language:");
            ui.label(spell_checker.current_language().name());
            ui.label(spell_checker.current_language().flag_emoji());
        });
        
        ui.horizontal(|ui| {
            ui.label("Dictionary words:");
            ui.label(format!("{}", spell_checker.word_count()));
        });
        
        ui.horizontal(|ui| {
            ui.label("User words:");
            ui.label(format!("{}", spell_checker.user_word_count()));
        });
        
        ui.horizontal(|ui| {
            ui.label("Ignored words:");
            ui.label(format!("{}", spell_checker.ignored_word_count()));
        });
        
        ui.separator();
        
        ui.heading("Add Word");
        ui.horizontal(|ui| {
            let mut new_word = String::new();
            let response = ui.text_edit_singleline(&mut new_word);
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !new_word.is_empty() {
                if crate::util::is_valid_word(&new_word) {
                    *on_add_word = Some(new_word.clone());
                }
            }
            
            let add_enabled = !new_word.is_empty() && crate::util::is_valid_word(&new_word);
            if ui.add_enabled(add_enabled, egui::Button::new("Add")).clicked() {
                *on_add_word = Some(new_word.clone());
            }
        });
        
        ui.label("Adds word to user dictionary permanently");
        
        ui.separator();
        
        ui.heading("Ignore Word");
        ui.horizontal(|ui| {
            let mut ignore_word = String::new();
            let response = ui.text_edit_singleline(&mut ignore_word);
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !ignore_word.is_empty() {
                if crate::util::is_valid_word(&ignore_word) {
                    *on_ignore_word = Some(ignore_word.clone());
                }
            }
            
            let ignore_enabled = !ignore_word.is_empty() && crate::util::is_valid_word(&ignore_word);
            if ui.add_enabled(ignore_enabled, egui::Button::new("Ignore")).clicked() {
                *on_ignore_word = Some(ignore_word.clone());
            }
        });
        
        ui.label("Ignores word for current session only");
        
        ui.separator();
        
        ui.heading("Dictionary Management");
        ui.horizontal_wrapped(|ui| {
            if ui.button("📥 Import").clicked() {
                *on_import_dict = true;
            }
            if ui.button("📤 Export").clicked() {
                *on_export_dict = true;
            }
            if ui.button("🗑️ Clear Ignored").clicked() {
                *on_clear_ignored = true;
            }
        });
        
        ui.checkbox(&mut self.show_ignored_words, "Show ignored words");
        
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.dictionary_filter);
        });
        
        ui.separator();
        
        ui.label("ℹ️ Added words are saved permanently");
        ui.label("Ignored words are session-only");
    }
    
    fn show_errors_view(
        &mut self,
        ui: &mut egui::Ui,
        analysis: &Option<DocumentAnalysis>,
        on_replace: &mut Option<(String, String)>,
    ) {
        ui.heading("Spelling Errors");
        
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.radio_value(&mut self.error_filter, ErrorFilter::All, "All");
            ui.radio_value(&mut self.error_filter, ErrorFilter::HighConfidence, "High Confidence");
            ui.radio_value(&mut self.error_filter, ErrorFilter::CodeIdentifiers, "Code");
            ui.radio_value(&mut self.error_filter, ErrorFilter::ProperNouns, "Proper Nouns");
        });
        
        if let Some(analysis) = analysis {
            if analysis.misspelled_words == 0 {
                ui.colored_label(egui::Color32::GREEN, "✅ No spelling errors found!");
                return;
            }
            
            let filtered_errors: Vec<&crate::checker::WordCheck> = analysis.words
                .iter()
                .filter(|w| !w.is_correct)
                .filter(|w| match self.error_filter {
                    ErrorFilter::All => true,
                    ErrorFilter::HighConfidence => w.confidence >= 0.8,
                    ErrorFilter::CodeIdentifiers => matches!(w.word_type, WordType::CodeIdentifier),
                    ErrorFilter::ProperNouns => matches!(w.word_type, WordType::ProperNoun),
                    ErrorFilter::Numbers => matches!(w.word_type, WordType::Number),
                })
                .collect();
            
            if filtered_errors.is_empty() {
                ui.label("No errors match the current filter");
                return;
            }
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, word) in filtered_errors.iter().enumerate() {
                    let is_selected = idx == self.selected_error_index;
                    
                    ui.horizontal(|ui| {
                        let color = match word.word_type {
                            WordType::CodeIdentifier => egui::Color32::BLUE,
                            WordType::ProperNoun => egui::Color32::YELLOW,
                            WordType::Acronym => egui::Color32::LIGHT_BLUE,
                            _ => egui::Color32::RED,
                        };
                        
                        ui.colored_label(color, "✗");
                        
                        if ui.selectable_label(is_selected, &word.word).clicked() {
                            self.selected_error_index = idx;
                        }
                        
                        ui.label(format!("(L{}:C{})", word.line, word.column));
                        
                        ui.colored_label(
                            egui::Color32::GRAY,
                            format!("{:.0}%", word.confidence * 100.0)
                        );
                    });
                    
                    if !word.suggestions.is_empty() {
                        ui.indent("suggestions", |ui| {
                            ui.label("Suggestions:");
                            for suggestion in &word.suggestions {
                                ui.horizontal(|ui| {
                                    if ui.button("Use").clicked() {
                                        *on_replace = Some((word.word.clone(), suggestion.clone()));
                                    }
                                    ui.label(suggestion);
                                });
                            }
                        });
                    }
                    
                    ui.separator();
                }
            });
            
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Errors: {}/{}", filtered_errors.len(), analysis.misspelled_words));
                if analysis.misspelled_words > 0 {
                    if ui.button("▶️ Fix All").clicked() {
                        ui.label("Feature coming soon...");
                    }
                }
            });
        } else {
            ui.label("No document loaded or checked.");
        }
    }
    
    fn show_stats_view(
        &mut self,
        ui: &mut egui::Ui,
        analysis: &Option<DocumentAnalysis>,
        spell_checker: &SpellChecker,
    ) {
        ui.heading("Document Statistics");
        
        if let Some(analysis) = analysis {
            ui.horizontal(|ui| {
                ui.label("Accuracy:");
                let gauge = egui::widgets::ProgressBar::new(analysis.accuracy / 100.0)
                    .show_percentage()
                    .desired_width(150.0);
                ui.add(gauge);
            });
            
            ui.separator();
            
            egui::Grid::new("stats_grid")
                .num_columns(2)
                .spacing([10.0, 5.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Total words:");
                    ui.label(format!("{}", analysis.total_words));
                    ui.end_row();
                    
                    ui.label("Unique words:");
                    ui.label(format!("{}", analysis.unique_words));
                    ui.end_row();
                    
                    ui.label("Misspelled:");
                    ui.colored_label(egui::Color32::RED, format!("{}", analysis.misspelled_words));
                    ui.end_row();
                    
                    ui.label("Accuracy:");
                    ui.label(format!("{:.1}%", analysis.accuracy));
                    ui.end_row();
                    
                    ui.label("Suggestions:");
                    ui.label(format!("{}", analysis.suggestions_count));
                    ui.end_row();
                    
                    ui.label("Lines checked:");
                    ui.label(format!("{}", analysis.lines_checked));
                    ui.end_row();
                    
                    ui.label("Language:");
                    ui.label(format!("{} {}", 
                        analysis.language.flag_emoji(),
                        analysis.language.name()
                    ));
                    ui.end_row();
                    
                    ui.label("Dictionary size:");
                    ui.label(format!("{} words", spell_checker.word_count()));
                    ui.end_row();
                    
                    ui.label("User dictionary:");
                    ui.label(format!("{} words", spell_checker.user_word_count()));
                    ui.end_row();
                    
                    ui.label("Check time:");
                    ui.label(format!("{}ms", analysis.check_duration_ms));
                    ui.end_row();
                    
                    if analysis.likely_code {
                        ui.label("File type:");
                        ui.colored_label(egui::Color32::BLUE, "Code");
                        ui.end_row();
                    }
                });
            
            if analysis.total_words > 0 {
                ui.separator();
                let minutes = analysis.total_words / 200;
                let seconds = ((analysis.total_words % 200) * 60) / 200;
                ui.label(format!("📖 Reading time: {} min {} sec", minutes, seconds));
                
                let characters = analysis.words.iter().map(|w| w.word.len()).sum::<usize>();
                ui.label(format!("🔤 Average word length: {:.1} chars", 
                    characters as f32 / analysis.total_words as f32));
            }
        } else {
            ui.label("No statistics available. Load a document first.");
        }
    }
    
    fn show_find_view(&mut self, ui: &mut egui::Ui, content: &str) {
        ui.heading("Find in Document");
        
        ui.horizontal(|ui| {
            ui.label("Find:");
            let response = ui.text_edit_singleline(&mut self.find_text);
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.find_text.is_empty() {
                // Find would be implemented
            }
        });
        
        ui.checkbox(&mut self.case_sensitive_find, "Case sensitive");
        ui.checkbox(&mut self.whole_word_find, "Whole word");
        
        if ui.button("Find Next").clicked() && !self.find_text.is_empty() {
            // Find next occurrence
        }
        
        if !self.find_text.is_empty() {
            let count = if self.case_sensitive_find {
                content.matches(&self.find_text).count()
            } else {
                content.to_lowercase().matches(&self.find_text.to_lowercase()).count()
            };
            
            if count > 0 {
                ui.colored_label(egui::Color32::GREEN, format!("Found {} occurrences", count));
            } else {
                ui.colored_label(egui::Color32::RED, "No matches found");
            }
        }
    }
    
    fn show_replace_view(&mut self, ui: &mut egui::Ui, content: &str, on_replace: &mut Option<(String, String)>) {
        ui.heading("Find and Replace");
        
        ui.horizontal(|ui| {
            ui.label("Find:");
            ui.text_edit_singleline(&mut self.find_text);
        });
        
        ui.horizontal(|ui| {
            ui.label("Replace:");
            ui.text_edit_singleline(&mut self.replace_text);
        });
        
        ui.checkbox(&mut self.case_sensitive_find, "Case sensitive");
        ui.checkbox(&mut self.whole_word_find, "Whole word");
        
        ui.horizontal(|ui| {
            if ui.button("Replace").clicked() && !self.find_text.is_empty() {
                *on_replace = Some((self.find_text.clone(), self.replace_text.clone()));
            }
            
            if ui.button("Replace All").clicked() && !self.find_text.is_empty() {
                // Implement replace all
                ui.label("Replace All coming soon...");
            }
        });
        
        if !self.find_text.is_empty() {
            let count = if self.case_sensitive_find {
                content.matches(&self.find_text).count()
            } else {
                content.to_lowercase().matches(&self.find_text.to_lowercase()).count()
            };
            ui.label(format!("Found {} occurrences", count));
        }
    }
    
    pub fn visible(&self) -> bool {
        self.visible
    }
    
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
    
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}