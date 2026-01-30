use crate::language::{Language, LanguageManager};
use dashmap::DashMap;
use regex::Regex;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Dictionary {
    words: HashSet<String>,
    ignored_words: HashSet<String>,
    word_pattern: Regex,
    min_word_length: usize,
    language: Language,
    is_loaded: bool,
    word_count_cache: usize,
    ignored_count_cache: usize,
    file_path: Option<PathBuf>,
}

impl Dictionary {
    pub fn new(language: Language) -> Self {
        let word_pattern = Self::get_word_pattern_for_language(&language);
        
        Self {
            words: HashSet::new(),
            ignored_words: HashSet::new(),
            word_pattern,
            min_word_length: 2,
            language,
            is_loaded: false,
            word_count_cache: 0,
            ignored_count_cache: 0,
            file_path: None,
        }
    }
    
    fn get_word_pattern_for_language(language: &Language) -> Regex {
        match language {
            Language::Chinese | Language::Japanese => {
                Regex::new(r"[\p{Han}\p{Hiragana}\p{Katakana}a-zA-Z0-9'-]+").unwrap()
            }
            Language::Korean => {
                Regex::new(r"[\p{Hangul}a-zA-Z0-9'-]+").unwrap()
            }
            Language::Russian => {
                Regex::new(r"[\p{Cyrillic}a-zA-Z0-9'-]+").unwrap()
            }
            _ => {
                Regex::new(r"[\p{L}0-9'-]+").unwrap()
            }
        }
    }
    
    pub fn load(&mut self) -> anyhow::Result<()> {
        if self.is_loaded {
            return Ok(());
        }
        
        let language_manager = LanguageManager::new();
        
        // Try to load main dictionary from CSV first, then fallback to TXT
        if let Some(csv_path) = self.find_dictionary_file(&language_manager, "csv") {
            println!("Loading CSV dictionary for {} from: {:?}", self.language.name(), csv_path);
            self.load_csv_file(&csv_path)?;
            self.file_path = Some(csv_path);
        } else if let Some(txt_path) = self.find_dictionary_file(&language_manager, "txt") {
            println!("Loading TXT dictionary for {} from: {:?}", self.language.name(), txt_path);
            self.load_txt_file(&txt_path)?;
            self.file_path = Some(txt_path);
        } else {
            println!("No dictionary file found for {}. Creating empty dictionary.", self.language.name());
        }
        
        // Load user-added words
        self.load_user_words();
        
        // Load ignored words
        self.load_ignored_words();
        
        self.is_loaded = true;
        self.word_count_cache = self.words.len();
        self.ignored_count_cache = self.ignored_words.len();
        
        println!("Loaded {} words ({} ignored) for {}", 
            self.word_count_cache, self.ignored_count_cache, self.language.name());
        
        Ok(())
    }
    
    fn find_dictionary_file(&self, language_manager: &LanguageManager, extension: &str) -> Option<PathBuf> {
        // First try the dictionary path from language manager
        if let Some(path) = language_manager.get_dictionary_path(&self.language) {
            let mut csv_path = path.clone();
            csv_path.set_extension(extension);
            if csv_path.exists() {
                return Some(csv_path);
            }
        }
        
        // Try common patterns
        let patterns = vec![
            format!("dictionary_{}.{}", self.language.code(), extension),
            format!("dictionary({}).{}", self.language.code(), extension),
            format!("{}.{}", self.language.code(), extension),
        ];
        
        let locations = vec![
            PathBuf::from("src/dictionary"),
            PathBuf::from("dictionary"),
            LanguageManager::system_dict_dir(),
            PathBuf::from("."),
        ];
        
        for location in locations {
            for pattern in &patterns {
                let path = location.join(pattern);
                if path.exists() {
                    return Some(path);
                }
            }
        }
        
        None
    }
    
    fn load_csv_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let file = File::open(path)?;
        let mut reader = csv::Reader::from_reader(file);
        
        let mut new_words = HashSet::new();
        
        for result in reader.records() {
            let record = result?;
            // CSV format: word,frequency,part_of_speech (optional)
            if let Some(word) = record.get(0) {
                let word = word.trim();
                if !word.is_empty() && word.len() >= self.min_word_length {
                    let normalized = self.normalize_word(word);
                    new_words.insert(normalized);
                }
            }
        }
        
        self.words.extend(new_words);
        self.word_count_cache = self.words.len();
        
        Ok(())
    }
    
    fn load_txt_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut new_words = HashSet::new();
        
        for line in reader.lines() {
            let line = line?;
            let word = line.trim();
            
            if !word.is_empty() && word.len() >= self.min_word_length {
                let normalized = self.normalize_word(word);
                new_words.insert(normalized);
            }
        }
        
        self.words.extend(new_words);
        self.word_count_cache = self.words.len();
        
        Ok(())
    }
    
    fn normalize_word(&self, word: &str) -> String {
        match self.language {
            Language::Chinese | Language::Japanese | Language::Korean => {
                word.to_string()
            }
            _ => {
                word.to_lowercase()
            }
        }
    }
    
    fn load_user_words(&mut self) {
        let mut path = LanguageManager::user_dict_dir();
        path.push(format!("user_{}.csv", self.language.code()));
        
        // First try CSV, then fallback to TXT
        if let Ok(file) = File::open(&path) {
            if let Ok(mut reader) = csv::Reader::from_reader(file) {
                for result in reader.records() {
                    if let Ok(record) = result {
                        if let Some(word) = record.get(0) {
                            let word = word.trim().to_string();
                            if !word.is_empty() {
                                self.words.insert(self.normalize_word(&word));
                            }
                        }
                    }
                }
            }
        } else {
            // Fallback to TXT
            let mut txt_path = path.clone();
            txt_path.set_extension("txt");
            if let Ok(file) = File::open(&txt_path) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(word) = line {
                        let word = word.trim().to_string();
                        if !word.is_empty() {
                            self.words.insert(self.normalize_word(&word));
                        }
                    }
                }
            }
        }
    }
    
    fn save_user_words(&self) -> anyhow::Result<()> {
        let mut path = LanguageManager::user_dict_dir();
        path.push(format!("user_{}.csv", self.language.code()));
        
        let file = File::create(&path)?;
        let mut writer = csv::Writer::from_writer(file);
        
        let mut sorted_words: Vec<&String> = self.words.iter().collect();
        sorted_words.sort();
        
        for word in sorted_words {
            writer.write_record(&[word])?;
        }
        
        writer.flush()?;
        
        Ok(())
    }
    
    fn load_ignored_words(&mut self) {
        let mut path = LanguageManager::user_dict_dir();
        path.push(format!("ignored_{}.csv", self.language.code()));
        
        // First try CSV, then fallback to TXT
        if let Ok(file) = File::open(&path) {
            if let Ok(mut reader) = csv::Reader::from_reader(file) {
                for result in reader.records() {
                    if let Ok(record) = result {
                        if let Some(word) = record.get(0) {
                            let word = word.trim().to_string();
                            if !word.is_empty() {
                                self.ignored_words.insert(self.normalize_word(&word));
                            }
                        }
                    }
                }
            }
        } else {
            // Fallback to TXT
            let mut txt_path = path.clone();
            txt_path.set_extension("txt");
            if let Ok(file) = File::open(&txt_path) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(word) = line {
                        let word = word.trim().to_string();
                        if !word.is_empty() {
                            self.ignored_words.insert(self.normalize_word(&word));
                        }
                    }
                }
            }
        }
        
        self.ignored_count_cache = self.ignored_words.len();
    }
    
    fn save_ignored_words(&self) -> anyhow::Result<()> {
        let mut path = LanguageManager::user_dict_dir();
        path.push(format!("ignored_{}.csv", self.language.code()));
        
        let file = File::create(&path)?;
        let mut writer = csv::Writer::from_writer(file);
        
        let mut sorted_words: Vec<&String> = self.ignored_words.iter().collect();
        sorted_words.sort();
        
        for word in sorted_words {
            writer.write_record(&[word])?;
        }
        
        writer.flush()?;
        
        Ok(())
    }
    
    pub fn contains(&self, word: &str, case_sensitive: bool, is_code_context: bool) -> bool {
        let word = word.trim();
        
        if word.is_empty() || word.len() < self.min_word_length {
            return true;
        }
        
        // Check if word is ignored
        let normalized = self.normalize_word(word);
        if self.ignored_words.contains(&normalized) {
            return true;
        }
        
        // Skip words that look like code identifiers in code context
        if is_code_context && self.is_likely_code_identifier(word) {
            return true;
        }
        
        // Skip words with numbers (except in CJK)
        if !matches!(self.language, Language::Chinese | Language::Japanese | Language::Korean) {
            if word.chars().any(|c| c.is_ascii_digit()) && word.len() > 3 {
                // Allow numbers in longer words (like "word123")
                let letter_count = word.chars().filter(|c| c.is_alphabetic()).count();
                if letter_count < 3 {
                    return true;
                }
            }
        }
        
        // Check in dictionary
        match self.language {
            Language::Chinese | Language::Japanese | Language::Korean => {
                self.words.contains(&normalized)
            }
            _ => {
                if case_sensitive {
                    self.words.contains(word)
                } else {
                    self.words.contains(&normalized)
                }
            }
        }
    }
    
    pub fn is_likely_code_identifier(&self, word: &str) -> bool {
        if word.len() < 2 || word.len() > 30 {
            return false;
        }
        
        let has_underscore = word.contains('_');
        let has_mixed_case = word.chars().any(|c| c.is_uppercase()) && 
                             word.chars().any(|c| c.is_lowercase());
        let _starts_with_letter = word.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false);
        
        (has_underscore && !word.starts_with('_') && !word.ends_with('_')) ||
        (has_mixed_case && !word.chars().all(|c| c.is_uppercase())) ||
        word.starts_with("get_") || word.starts_with("set_") ||
        word.starts_with("is_") || word.starts_with("has_") ||
        word.ends_with("_t") || word.ends_with("_ptr") ||
        word.ends_with("Handler") || word.ends_with("Service") ||
        word.ends_with("Manager") || word.ends_with("Factory")
    }
    
    pub fn word_count(&self) -> usize {
        self.word_count_cache
    }
    
    pub fn ignored_word_count(&self) -> usize {
        self.ignored_count_cache
    }
    
    pub fn get_words(&self) -> &HashSet<String> {
        &self.words
    }
    
    pub fn get_word_pattern(&self) -> &Regex {
        &self.word_pattern
    }
    
    pub fn language(&self) -> &Language {
        &self.language
    }
    
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }
    
    pub fn add_word(&mut self, word: &str) -> anyhow::Result<()> {
        let normalized = self.normalize_word(word.trim());
        
        if !normalized.is_empty() && normalized.len() >= self.min_word_length {
            self.words.insert(normalized.clone());
            self.word_count_cache = self.words.len();
            
            self.ignored_words.remove(&normalized);
            self.ignored_count_cache = self.ignored_words.len();
            
            self.save_user_words()?;
        }
        
        Ok(())
    }
    
    pub fn ignore_word(&mut self, word: &str) -> anyhow::Result<()> {
        let normalized = self.normalize_word(word.trim());
        
        if !normalized.is_empty() {
            self.ignored_words.insert(normalized);
            self.ignored_count_cache = self.ignored_words.len();
            
            self.save_ignored_words()?;
        }
        
        Ok(())
    }
    
    pub fn clear_ignored_words(&mut self) -> anyhow::Result<()> {
        self.ignored_words.clear();
        self.ignored_count_cache = 0;
        self.save_ignored_words()?;
        Ok(())
    }
    
    pub fn remove_word(&mut self, word: &str) -> bool {
        let removed = self.words.remove(word);
        if removed {
            self.word_count_cache = self.words.len();
        }
        removed
    }
    
    pub fn save_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("csv");
        
        match extension {
            "csv" => {
                let file = File::create(path)?;
                let mut writer = csv::Writer::from_writer(file);
                let mut sorted_words: Vec<&String> = self.words.iter().collect();
                sorted_words.sort();
                
                for word in sorted_words {
                    writer.write_record(&[word])?;
                }
                
                writer.flush()?;
            }
            "txt" => {
                let mut file = File::create(path)?;
                let mut sorted_words: Vec<&String> = self.words.iter().collect();
                sorted_words.sort();
                
                for word in sorted_words {
                    writeln!(file, "{}", word)?;
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported file extension: {}", extension));
            }
        }
        
        Ok(())
    }
    
    pub fn import_from_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("csv");
        
        match extension {
            "csv" => self.load_csv_file(path),
            "txt" => self.load_txt_file(path),
            _ => Err(anyhow::anyhow!("Unsupported file extension: {}", extension)),
        }
    }
    
    pub fn export_to_file(&self, path: &Path) -> anyhow::Result<()> {
        self.save_to_file(path)
    }
}

#[derive(Clone)]
pub struct DictionaryManager {
    dictionaries: Arc<DashMap<Language, Dictionary>>,
    language_manager: LanguageManager,
}

impl Default for DictionaryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryManager {
    pub fn new() -> Self {
        let manager = LanguageManager::new();
        let dictionaries = Arc::new(DashMap::new());
        
        Self {
            dictionaries,
            language_manager: manager,
        }
    }
    
    pub fn get_dictionary(&self, language: &Language) -> anyhow::Result<Dictionary> {
        if let Some(dict) = self.dictionaries.get(language) {
            return Ok(dict.clone());
        }
        
        let mut dict = Dictionary::new(*language);
        dict.load()?;
        self.dictionaries.insert(*language, dict.clone());
        
        Ok(dict)
    }
    
    pub fn reload_dictionary(&mut self, language: &Language) -> anyhow::Result<()> {
        let mut dict = Dictionary::new(*language);
        dict.load()?;
        self.dictionaries.insert(*language, dict);
        Ok(())
    }
    
    pub fn add_custom_dictionary(&mut self, path: PathBuf, language: Language) -> anyhow::Result<()> {
        let mut dict = Dictionary::new(language);
        dict.import_from_file(&path)?;
        self.dictionaries.insert(language, dict);
        Ok(())
    }
    
    pub fn add_word_to_dictionary(&mut self, word: &str, language: Language) -> anyhow::Result<()> {
        if let Some(mut dict) = self.dictionaries.get_mut(&language) {
            dict.add_word(word)
        } else {
            let mut dict = Dictionary::new(language);
            dict.load()?;
            dict.add_word(word)?;
            self.dictionaries.insert(language, dict);
            Ok(())
        }
    }
    
    pub fn ignore_word(&mut self, word: &str, language: Language) -> anyhow::Result<()> {
        if let Some(mut dict) = self.dictionaries.get_mut(&language) {
            dict.ignore_word(word)
        } else {
            let mut dict = Dictionary::new(language);
            dict.load()?;
            dict.ignore_word(word)?;
            self.dictionaries.insert(language, dict);
            Ok(())
        }
    }
    
    pub fn clear_ignored_words(&mut self, language: Language) -> anyhow::Result<()> {
        if let Some(mut dict) = self.dictionaries.get_mut(&language) {
            dict.clear_ignored_words()
        } else {
            let mut dict = Dictionary::new(language);
            dict.load()?;
            dict.clear_ignored_words()?;
            self.dictionaries.insert(language, dict);
            Ok(())
        }
    }
    
    pub fn import_dictionary(&mut self, path: PathBuf, language: Language) -> anyhow::Result<()> {
        self.add_custom_dictionary(path, language)
    }
    
    pub fn export_dictionary(&self, language: &Language, path: &Path) -> anyhow::Result<()> {
        let dict = self.get_dictionary(language)?;
        dict.export_to_file(path)
    }
    
    pub fn get_available_languages(&self) -> Vec<Language> {
        self.language_manager.available_languages().to_vec()
    }
    
    pub fn detect_language(&self, text: &str) -> Language {
        self.language_manager.detect_language(text)
    }
    
    pub fn get_current_language(&self) -> Language {
        self.language_manager.current_language()
    }
    
    pub fn set_current_language(&mut self, language: Language) {
        self.language_manager.set_language(language);
    }
    
    pub fn get_cached_dictionary(&self, language: &Language) -> Option<Dictionary> {
        self.dictionaries.get(language).map(|d| d.value().clone())
    }
}