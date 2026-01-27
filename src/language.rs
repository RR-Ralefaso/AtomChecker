use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Afrikaans,
    French,
    Spanish,
    German,
    Chinese,
    Italian,
    Portuguese,
    Russian,
    Japanese,
    Korean,
    AutoDetect,
}

impl Language {
    pub fn all() -> Vec<Language> {
        vec![
            Language::English,
            Language::Afrikaans,
            Language::French,
            Language::Spanish,
            Language::German,
            Language::Chinese,
            Language::Italian,
            Language::Portuguese,
            Language::Russian,
            Language::Japanese,
            Language::Korean,
            Language::AutoDetect,
        ]
    }
    
    pub fn code(&self) -> &str {
        match self {
            Language::English => "eng",
            Language::Afrikaans => "afr",
            Language::French => "fra",
            Language::Spanish => "spa",
            Language::German => "deu",
            Language::Chinese => "zho",
            Language::Italian => "ita",
            Language::Portuguese => "por",
            Language::Russian => "rus",
            Language::Japanese => "jpn",
            Language::Korean => "kor",
            Language::AutoDetect => "auto",
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Afrikaans => "Afrikaans",
            Language::French => "French",
            Language::Spanish => "Spanish",
            Language::German => "German",
            Language::Chinese => "Chinese",
            Language::Italian => "Italian",
            Language::Portuguese => "Portuguese",
            Language::Russian => "Russian",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::AutoDetect => "Auto-detect",
        }
    }
    
    pub fn flag_emoji(&self) -> &str {
        match self {
            Language::English => "🇬🇧",
            Language::Afrikaans => "🇿🇦",
            Language::French => "🇫🇷",
            Language::Spanish => "🇪🇸",
            Language::German => "🇩🇪",
            Language::Chinese => "🇨🇳",
            Language::Italian => "🇮🇹",
            Language::Portuguese => "🇵🇹",
            Language::Russian => "🇷🇺",
            Language::Japanese => "🇯🇵",
            Language::Korean => "🇰🇷",
            Language::AutoDetect => "🌐",
        }
    }
    
    pub fn dictionary_filename(&self) -> Option<String> {
        match self {
            Language::AutoDetect => None,
            _ => Some(format!("dictionary({}).txt", self.code())),
        }
    }
    
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "eng" | "en" | "english" => Language::English,
            "afr" | "af" | "afrikaans" => Language::Afrikaans,
            "fra" | "fr" | "french" => Language::French,
            "spa" | "es" | "spanish" => Language::Spanish,
            "deu" | "de" | "german" => Language::German,
            "zho" | "zh" | "chinese" => Language::Chinese,
            "ita" | "it" | "italian" => Language::Italian,
            "por" | "pt" | "portuguese" => Language::Portuguese,
            "rus" | "ru" | "russian" => Language::Russian,
            "jpn" | "ja" | "japanese" => Language::Japanese,
            "kor" | "ko" | "korean" => Language::Korean,
            "auto" | "autodetect" => Language::AutoDetect,
            _ => Language::English,
        }
    }
    
    pub fn detect_from_text(text: &str) -> Vec<(Language, f32)> {
        static COMMON_WORDS: Lazy<HashMap<Language, Vec<&'static str>>> = Lazy::new(|| {
            let mut map = HashMap::new();
            
            map.insert(Language::English, vec![
                "the", "and", "that", "have", "for", "with", "this", "from", "they", "would",
                "will", "what", "there", "their", "about", "which", "when", "who", "them",
                "some", "time", "could", "people", "other", "than", "then", "now", "look",
                "only", "come", "its", "over", "think", "also", "back", "after", "use",
                "two", "how", "our", "work", "first", "well", "way", "even", "new", "want"
            ]);
            
            map.insert(Language::Afrikaans, vec![
                "die", "en", "het", "vir", "om", "wat", "in", "is", "jy", "ek",
                "nie", "sy", "ons", "hulle", "daar", "maar", "my", "haar", "so", "by",
                "kan", "van", "dit", "te", "met", "hy", "was", "op", "een", "nie",
                "toe", "gaan", "moet", "nog", "al", "uit", "sê", "moet", "baie", "hier",
                "wees", "gewees", "het", "word", "waar", "kom", "laat", "dink", "sien", "nous"
            ]);
            
            map.insert(Language::French, vec![
                "le", "la", "et", "que", "dans", "un", "est", "pour", "des", "les",
                "une", "pas", "son", "avec", "il", "elle", "dans", "qui", "mais", "nous",
                "vous", "ce", "se", "aux", "du", "de", "par", "sur", "est", "sont",
                "cette", "été", "plus", "pouvoir", "comme", "tout", "faire", "me", "même",
                "sans", "autre", "aussi", "bien", "si", "y", "ou", "où", "lui", "donc"
            ]);
            
            map.insert(Language::Spanish, vec![
                "el", "la", "de", "que", "y", "a", "en", "un", "ser", "se",
                "no", "haber", "por", "con", "su", "para", "como", "estar", "tener", "le",
                "lo", "todo", "pero", "más", "hacer", "o", "poder", "decir", "este", "ir",
                "otro", "ese", "la", "si", "me", "ya", "ver", "porque", "dar", "cuando",
                "él", "muy", "sin", "vez", "mucho", "saber", "qué", "sobre", "mi", "alguno"
            ]);
            
            map.insert(Language::German, vec![
                "der", "die", "und", "in", "den", "von", "zu", "das", "mit", "sich",
                "des", "auf", "für", "ist", "im", "dem", "nicht", "ein", "Die", "eine",
                "als", "auch", "es", "an", "werden", "aus", "er", "hat", "dass", "sie",
                "nach", "wird", "bei", "einer", "Der", "um", "am", "sind", "noch", "wie"
            ]);
            
            map
        });
        
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        
        if words.len() < 3 {
            return vec![(Language::English, 100.0)];
        }
        
        let mut scores = HashMap::new();
        
        for (language, common_words) in COMMON_WORDS.iter() {
            let mut matches = 0;
            let mut total_checked = 0;
            
            for word in words.iter().take(50) {
                total_checked += 1;
                if common_words.contains(word) {
                    matches += 1;
                }
            }
            
            if total_checked > 0 {
                let score = (matches as f32 / total_checked as f32) * 100.0;
                if score > 10.0 {
                    scores.insert(*language, score);
                }
            }
        }
        
        let cjk_chars: Vec<char> = text.chars()
            .filter(|c| {
                ('\u{4E00}' <= *c && *c <= '\u{9FFF}') ||
                ('\u{3040}' <= *c && *c <= '\u{309F}') ||
                ('\u{30A0}' <= *c && *c <= '\u{30FF}') ||
                ('\u{AC00}' <= *c && *c <= '\u{D7AF}')
            })
            .collect();
        
        let cjk_ratio = cjk_chars.len() as f32 / text.chars().count().max(1) as f32;
        
        if cjk_ratio > 0.3 {
            if text.contains('\u{4E00}') {
                scores.insert(Language::Chinese, 100.0);
            } else if text.contains('\u{3040}') || text.contains('\u{30A0}') {
                scores.insert(Language::Japanese, 100.0);
            } else if text.contains('\u{AC00}') {
                scores.insert(Language::Korean, 100.0);
            }
        }
        
        if scores.is_empty() {
            scores.insert(Language::English, 80.0);
        }
        
        let mut sorted_scores: Vec<(Language, f32)> = scores.into_iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        sorted_scores.truncate(3);
        sorted_scores
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageManager {
    available_languages: Vec<Language>,
    current_language: Language,
    #[serde(skip)]
    dictionary_paths: HashMap<Language, PathBuf>,
}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageManager {
    pub fn new() -> Self {
        let mut manager = Self {
            available_languages: Language::all(),
            current_language: Language::English,
            dictionary_paths: HashMap::new(),
        };
        
        manager.scan_dictionaries();
        manager
    }
    
    fn scan_dictionaries(&mut self) {
        let locations = vec![
            PathBuf::from("src/dictionary"),
            PathBuf::from("dictionary"),
            Self::system_dict_dir(),
            PathBuf::from("."),
        ];
        
        for location in locations {
            if let Ok(entries) = std::fs::read_dir(&location) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                        if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                            if let Some(lang_code) = filename
                                .strip_prefix("dictionary(")
                                .and_then(|s| s.strip_suffix(")"))
                            {
                                let language = Language::from_code(lang_code);
                                self.dictionary_paths.insert(language, path.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn dictionary_dir() -> PathBuf {
        let project_path = PathBuf::from("src/dictionary");
        if project_path.exists() {
            return project_path;
        }
        
        let current_path = PathBuf::from("dictionary");
        if current_path.exists() {
            return current_path;
        }
        
        PathBuf::from(".")
    }
    
    pub fn system_dict_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "ralefaso", "AtomSpell")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }
    
    pub fn user_dict_dir() -> PathBuf {
        let mut path = Self::system_dict_dir();
        path.push("user_dictionaries");
        std::fs::create_dir_all(&path).ok();
        path
    }
    
    pub fn get_dictionary_path(&self, language: &Language) -> Option<PathBuf> {
        match language {
            Language::AutoDetect => None,
            lang => {
                if let Some(path) = self.dictionary_paths.get(lang) {
                    if path.exists() {
                        return Some(path.clone());
                    }
                }
                
                if let Some(filename) = lang.dictionary_filename() {
                    let locations = vec![
                        Self::dictionary_dir().join(&filename),
                        PathBuf::from("src/dictionary").join(&filename),
                        PathBuf::from("dictionary").join(&filename),
                        Self::user_dict_dir().join(&filename),
                    ];
                    
                    for path in locations {
                        if path.exists() {
                            return Some(path);
                        }
                    }
                }
                
                None
            }
        }
    }
    
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }
    
    pub fn current_language(&self) -> Language {
        self.current_language
    }
    
    pub fn available_languages(&self) -> &[Language] {
        &self.available_languages
    }
    
    pub fn add_custom_dictionary(&mut self, path: PathBuf, language: Language) {
        self.dictionary_paths.insert(language, path);
    }
    
    pub fn detect_language(&self, text: &str) -> Language {
        if text.trim().is_empty() {
            return Language::English;
        }
        
        let scores = Language::detect_from_text(text);
        
        if let Some((detected_lang, score)) = scores.first() {
            if *score > 25.0 {
                return *detected_lang;
            }
        }
        
        Language::English
    }
}