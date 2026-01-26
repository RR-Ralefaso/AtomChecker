use crate::checker::{DocumentAnalysis, SpellChecker};
use crate::editor::TextEditor;
use crate::language::{Language, LanguageManager};
use crate::sidebar::Sidebar;
use crate::theme::AtomTheme;
use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AppState {
    pub current_file: Option<PathBuf>,
    pub document_content: String,
    pub is_document_modified: bool,
    pub auto_check: bool,
    pub show_line_numbers: bool,
    pub sidebar_width: f32,
    pub theme: AtomTheme,
    pub recent_files: Vec<PathBuf>,
    pub dictionary_paths: Vec<PathBuf>,
    pub selected_language: Language,
    pub auto_detect_language: bool,
    pub available_languages: Vec<Language>,
    pub show_dictionary_manager: bool,
    pub font_size: f32,
    pub wrap_text: bool,
    pub show_whitespace: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let language_manager = LanguageManager::new();
        
        Self {
            current_file: None,
            document_content: String::new(),
            is_document_modified: false,
            auto_check: true,
            show_line_numbers: true,
            sidebar_width: 300.0,
            theme: AtomTheme::OneDark,
            recent_files: Vec::new(),
            dictionary_paths: Vec::new(),
            selected_language: Language::English,
            auto_detect_language: true,
            available_languages: language_manager.available_languages().to_vec(),
            show_dictionary_manager: false,
            font_size: 14.0,
            wrap_text: true,
            show_whitespace: false,
        }
    }
}

pub struct SpellCheckerApp {
    state: AppState,
    text_editor: TextEditor,
    sidebar: Sidebar,
    spell_checker: Arc<SpellChecker>,
    last_check_time: Instant,
    check_interval: std::time::Duration,
    is_dragging_file: bool,
    drop_highlight: bool,
    stats: CheckStats,
    language_manager: LanguageManager,
    analysis: Option<DocumentAnalysis>,
    pending_add_word: Option<String>,
    pending_ignore_word: Option<String>,
    pending_replace: Option<(String, String)>,
}

#[derive(Default)]
struct CheckStats {
    total_words: usize,
    errors: usize,
    last_check_duration: std::time::Duration,
    detected_language: Option<Language>,
}

impl SpellCheckerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let state = AppState::default();
        let language_manager = LanguageManager::new();
        
        let spell_checker = Arc::new(
            SpellChecker::new(state.selected_language)
                .expect("Failed to create spell checker"),
        );
        
        let mut text_editor = TextEditor::new();
        text_editor.set_font_size(state.font_size);
        
        Self {
            state: state.clone(),
            text_editor,
            sidebar: Sidebar::new(),
            spell_checker,
            last_check_time: Instant::now(),
            check_interval: std::time::Duration::from_millis(1000),
            is_dragging_file: false,
            drop_highlight: false,
            stats: CheckStats::default(),
            language_manager,
            analysis: None,
            pending_add_word: None,
            pending_ignore_word: None,
            pending_replace: None,
        }
    }
    
    fn check_spelling(&mut self) {
        if self.state.auto_check && !self.state.document_content.is_empty() {
            let start_time = Instant::now();
            
            // Detect language if auto-detect is enabled
            let language_to_use = if self.state.auto_detect_language {
                let detected = self.language_manager.detect_language(&self.state.document_content);
                self.stats.detected_language = Some(detected);
                detected
            } else {
                self.state.selected_language
            };
            
            // Update spell checker language if changed
            if language_to_use != self.spell_checker.current_language() {
                if let Err(e) = Arc::get_mut(&mut self.spell_checker).unwrap().set_language(language_to_use) {
                    eprintln!("Failed to change language: {}", e);
                }
            }
            
            self.analysis = Some(self.spell_checker.check_document(&self.state.document_content));
            let analysis = self.analysis.as_ref().unwrap();
            
            self.stats.total_words = analysis.total_words;
            self.stats.errors = analysis.misspelled_words;
            self.stats.last_check_duration = start_time.elapsed();
            
            self.text_editor.set_analysis(analysis.clone());
            self.last_check_time = Instant::now();
        }
    }
    
    fn open_file(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(&path)?;
        self.state.current_file = Some(path.clone());
        self.state.document_content = content;
        self.state.is_document_modified = false;
        
        // Add to recent files
        if !self.state.recent_files.contains(&path) {
            self.state.recent_files.insert(0, path);
            if self.state.recent_files.len() > 10 {
                self.state.recent_files.pop();
            }
        }
        
        // Auto-detect language from file content
        if self.state.auto_detect_language {
            let detected = self.language_manager.detect_language(&self.state.document_content);
            self.state.selected_language = detected;
            if let Err(e) = Arc::get_mut(&mut self.spell_checker).unwrap().set_language(detected) {
                eprintln!("Failed to set language: {}", e);
            }
        }
        
        // Trigger spell check
        self.check_spelling();
        
        Ok(())
    }
    
    fn save_file(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.state.current_file {
            std::fs::write(path, &self.state.document_content)?;
            self.state.is_document_modified = false;
        }
        Ok(())
    }
    
    fn save_as(&mut self) -> anyhow::Result<()> {
        if let Some(path) = FileDialog::new()
            .add_filter("Text files", &["txt", "md", "rs", "py", "js", "html", "css"])
            .set_file_name(
                self.state
                    .current_file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("document.txt"),
            )
            .save_file()
        {
            std::fs::write(&path, &self.state.document_content)?;
            self.state.current_file = Some(path);
            self.state.is_document_modified = false;
        }
        Ok(())
    }
    
    fn handle_file_drop(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            self.is_dragging_file = true;
        } else {
            self.is_dragging_file = false;
        }
        
        if ctx.input(|i| i.raw.dropped_files.len() > 0) {
            if let Some(file) = ctx.input(|i| i.raw.dropped_files[0].path.clone()) {
                if let Err(e) = self.open_file(file) {
                    eprintln!("Failed to open dropped file: {}", e);
                }
            }
            self.drop_highlight = false;
        }
        
        if ctx.input(|i| i.pointer.any_down()) && self.is_dragging_file {
            self.drop_highlight = true;
        } else {
            self.drop_highlight = false;
        }
    }
    
    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("📂 Open File...").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Text files", &["txt", "md", "rs", "py", "js", "html", "css"])
                        .pick_file()
                    {
                        if let Err(e) = self.open_file(path) {
                            eprintln!("Failed to open file: {}", e);
                        }
                    }
                    ui.close_menu();
                }
                
                if ui.button("📁 Open Folder...").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        println!("Selected folder: {:?}", path);
                    }
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("💾 Save").clicked() {
                    if let Err(e) = self.save_file() {
                        eprintln!("Failed to save file: {}", e);
                    }
                    ui.close_menu();
                }
                
                if ui.button("💾 Save As...").clicked() {
                    if let Err(e) = self.save_as() {
                        eprintln!("Failed to save file: {}", e);
                    }
                    ui.close_menu();
                }
                
                ui.separator();
                
                if !self.state.recent_files.is_empty() {
                    ui.menu_button("Recent Files", |ui| {
                        for path in &self.state.recent_files {
                            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                if ui.button(format!("📄 {}", filename)).clicked() {
                                    if let Err(e) = self.open_file(path.clone()) {
                                        eprintln!("Failed to open file: {}", e);
                                    }
                                    ui.close_menu();
                                }
                            }
                        }
                    });
                }
                
                ui.separator();
                
                if ui.button("🚪 Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            
            ui.menu_button("Edit", |ui| {
                if ui.button("✏️ Check Spelling Now").clicked() {
                    self.check_spelling();
                    ui.close_menu();
                }
                
                ui.checkbox(&mut self.state.auto_check, "🔄 Auto