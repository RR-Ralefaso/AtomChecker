// Core modules
pub mod checker;
pub mod dictionary;
pub mod editor;
pub mod gui;
pub mod language;
pub mod sidebar;
pub mod theme;
pub mod util;

// Re-export common types for easier access
pub use checker::{DocumentAnalysis, SpellChecker, WordCheck, WordType};
pub use dictionary::DictionaryManager;
pub use gui::SpellCheckerApp;
pub use language::{Language, LanguageManager};
pub use theme::AtomTheme;
pub use sidebar::Sidebar;

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum SpellCheckerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid dictionary path: {0}")]
    InvalidDictionaryPath(String),
    
    #[error("Dictionary not found for language: {0}")]
    DictionaryNotFound(String),
    
    #[error("Empty dictionary")]
    EmptyDictionary,
    
    #[error("Invalid document encoding")]
    InvalidEncoding,
    
    #[error("Language error: {0}")]
    Language(String),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Serde JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Dictionary error: {0}")]
    Dictionary(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, SpellCheckerError>;

// Constants
pub const APP_NAME: &str = "AtomSpell";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_AUTHOR: &str = "RR-RALEFASO";
pub const APP_REPOSITORY: &str = "https://github.com/RR-Ralefaso/SpellChecker";
pub const SPONSOR_URL: &str = "https://github.com/sponsors/RR-Ralefaso";

// Helper functions
pub fn open_sponsor_page() -> Result<()> {
    open::that(SPONSOR_URL).map_err(|e| SpellCheckerError::Unknown(e.into()))
}

pub fn open_repository() -> Result<()> {
    open::that(APP_REPOSITORY).map_err(|e| SpellCheckerError::Unknown(e.into()))
}

// Global configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub enable_auto_save: bool,
    pub auto_save_interval: u64,
    pub max_recent_files: usize,
    pub enable_animations: bool,
    pub enable_advanced_typo_detection: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_auto_save: true,
            auto_save_interval: 30,
            max_recent_files: 10,
            enable_animations: true,
            enable_advanced_typo_detection: true,
        }
    }
}