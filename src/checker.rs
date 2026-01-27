use crate::dictionary::{Dictionary, DictionaryManager};
use crate::language::Language;
use crate::util::{sanitize_word, is_valid_word};
use dashmap::DashMap;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::cmp::min;

#[derive(Debug, Clone, Serialize)]
pub struct WordCheck {
    pub word: String,
    pub original: String, // Original casing
    pub start: usize,
    pub end: usize,
    pub is_correct: bool,
    pub suggestions: Vec<String>,
    pub line: usize,
    pub column: usize,
    pub confidence: f32, // How confident we are this is a typo
    pub word_type: WordType,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum WordType {
    Normal,
    CodeIdentifier,
    Acronym,
    ProperNoun,
    TechnicalTerm,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentAnalysis {
    pub total_words: usize,
    pub misspelled_words: usize,
    pub accuracy: f32,
    pub words: Vec<WordCheck>,
    pub suggestions_count: usize,
    pub language: Language,
    pub lines_checked: usize,
    pub check_duration_ms: u128,
    pub likely_code: bool,
    pub file_type: Option<String>,
}

pub struct SpellChecker {
    dictionary_manager: DictionaryManager,
    current_language: Language,
    suggestions_enabled: bool,
    case_sensitive: bool,
    max_suggestions: usize,
    cache: Arc<DashMap<String, bool>>,
    ignore_list: HashSet<String>,
    user_dictionary: HashSet<String>,
    proper_nouns: HashSet<String>,
    acronyms: HashSet<String>,
    code_patterns: HashSet<String>,
    confidence_threshold: f32,
}

impl SpellChecker {
    pub fn new(language: Language) -> anyhow::Result<Self> {
        let dictionary_manager = DictionaryManager::new();
        
        // Try to load dictionary
        let dict_result = dictionary_manager.get_dictionary(&language);
        if let Err(e) = dict_result {
            eprintln!("Warning: Could not load dictionary for {}: {}", language.name(), e);
            // Continue with empty dictionary
        }
        
        Ok(Self {
            dictionary_manager,
            current_language: language,
            suggestions_enabled: true,
            case_sensitive: false,
            max_suggestions: 5,
            cache: Arc::new(DashMap::new()),
            ignore_list: HashSet::new(),
            user_dictionary: HashSet::new(),
            proper_nouns: HashSet::new(),
            acronyms: HashSet::new(),
            code_patterns: HashSet::new(),
            confidence_threshold: 0.7,
        })
    }
    
    pub fn set_language(&mut self, language: Language) -> anyhow::Result<()> {
        if language != self.current_language {
            self.dictionary_manager.get_dictionary(&language)?;
            self.current_language = language;
            self.cache.clear(); // Clear cache when language changes
            self.load_user_data(); // Reload user data for new language
        }
        Ok(())
    }
    
    fn load_user_data(&mut self) {
        // Load user dictionary, proper nouns, etc. from files
        // This is a simplified version
        self.user_dictionary.clear();
        self.proper_nouns.clear();
        self.acronyms.clear();
        
        // In a real implementation, you'd load these from files
        self.acronyms.extend(vec![
            "API", "HTTP", "HTTPS", "URL", "URI", "HTML", "CSS", "JS", "TS",
            "JSON", "XML", "SQL", "NoSQL", "CPU", "GPU", "RAM", "ROM", "USB",
            "SSD", "HDD", "LAN", "WAN", "VPN", "DNS", "IP", "TCP", "UDP"
        ].iter().map(|s| s.to_lowercase()));
    }
    
    pub fn check_document_with_context(&self, text: &str, filename: Option<&str>) -> DocumentAnalysis {
        let start_time = std::time::Instant::now();
        
        let dictionary = match self.get_current_dictionary() {
            Ok(dict) => dict,
            Err(_) => {
                return DocumentAnalysis {
                    total_words: 0,
                    misspelled_words: 0,
                    accuracy: 100.0,
                    words: Vec::new(),
                    suggestions_count: 0,
                    language: self.current_language,
                    lines_checked: 0,
                    check_duration_ms: 0,
                    likely_code: false,
                    file_type: filename.map(|f| f.to_string()),
                };
            }
        };
        
        let is_cjk = matches!(self.current_language, Language::Chinese | Language::Japanese | Language::Korean);
        let is_code = filename.map(|f| crate::util::is_code_file(f)).unwrap_or(false) ||
                     crate::util::is_likely_code(text);
        
        let word_pattern = if is_cjk {
            crate::util::CJK_WORD_REGEX.clone()
        } else if is_code {
            crate::util::CODE_WORD_REGEX.clone()
        } else {
            crate::util::WORD_REGEX.clone()
        };
        
        let lines: Vec<&str> = text.lines().collect();
        let mut words = Vec::new();
        let mut suggestions_count = 0;
        let mut total_words = 0;
        let mut misspelled_words = 0;
        
        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;
            
            for mat in word_pattern.find_iter(line) {
                let original_word = mat.as_str();
                let start = mat.start();
                let end = mat.end();
                
                // Skip very short words
                if original_word.len() < 2 {
                    continue;
                }
                
                // Determine word type
                let word_type = self.determine_word_type(original_word, is_code);
                
                // Skip based on word type
                if self.should_skip_word(original_word, &word_type) {
                    words.push(WordCheck {
                        word: original_word.to_string(),
                        original: original_word.to_string(),
                        start,
                        end,
                        is_correct: true,
                        suggestions: Vec::new(),
                        line: line_num,
                        column: start + 1,
                        confidence: 1.0,
                        word_type,
                    });
                    continue;
                }
                
                let word_lower = if is_cjk { original_word.to_string() } else { original_word.to_lowercase() };
                
                // Check in various dictionaries and lists
                let is_correct = self.check_word_correctness(&word_lower, original_word, &word_type, &dictionary);
                let confidence = self.calculate_confidence(original_word, &word_type, is_correct);
                
                total_words += 1;
                if !is_correct && confidence >= self.confidence_threshold {
                    misspelled_words += 1;
                }
                
                let suggestions = if !is_correct && self.suggestions_enabled && confidence >= self.confidence_threshold {
                    let sugg = self.get_suggestions(original_word, &dictionary, &word_type);
                    suggestions_count += sugg.len();
                    sugg
                } else {
                    Vec::new()
                };
                
                words.push(WordCheck {
                    word: word_lower.clone(),
                    original: original_word.to_string(),
                    start,
                    end,
                    is_correct: is_correct || confidence < self.confidence_threshold,
                    suggestions,
                    line: line_num,
                    column: start + 1,
                    confidence,
                    word_type,
                });
            }
        }
        
        let accuracy = if total_words > 0 {
            ((total_words - misspelled_words) as f32 / total_words as f32 * 100.0).round()
        } else {
            100.0
        };
        
        let check_duration = start_time.elapsed();
        
        DocumentAnalysis {
            total_words,
            misspelled_words,
            accuracy,
            words,
            suggestions_count,
            language: self.current_language,
            lines_checked: lines.len(),
            check_duration_ms: check_duration.as_millis(),
            likely_code: is_code,
            file_type: filename.map(|f| f.to_string()),
        }
    }
    
    fn determine_word_type(&self, word: &str, is_code: bool) -> WordType {
        // Check for acronyms (all caps or with numbers)
        if word.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_') && word.len() <= 6 {
            return WordType::Acronym;
        }
        
        // Check for proper nouns (starts with capital, not at sentence start)
        if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && word.len() > 2 {
            // Check if it's not a common word that happens to be capitalized
            let common_caps = ["I", "A", "The", "And", "But", "Or", "For", "Nor", "Yet", "So"];
            if !common_caps.contains(&word) {
                return WordType::ProperNoun;
            }
        }
        
        // Check for code identifiers
        if is_code && (word.contains('_') || 
                      (word.chars().any(|c| c.is_uppercase()) && word.chars().any(|c| c.is_lowercase())) ||
                      word.starts_with("get_") || word.starts_with("set_") ||
                      word.ends_with("_t") || word.ends_with("_ptr")) {
            return WordType::CodeIdentifier;
        }
        
        // Check for technical terms
        if word.contains('-') && word.len() > 5 {
            return WordType::TechnicalTerm;
        }
        
        WordType::Normal
    }
    
    fn should_skip_word(&self, word: &str, word_type: &WordType) -> bool {
        match word_type {
            WordType::Acronym => {
                // Skip common acronyms
                self.acronyms.contains(&word.to_lowercase())
            }
            WordType::CodeIdentifier => {
                // Skip common code patterns
                word.len() <= 3 || // Very short identifiers
                word.chars().all(|c| c.is_numeric()) || // Numbers
                word.starts_with("0x") || // Hex numbers
                word.contains("__") // Python dunders
            }
            WordType::ProperNoun => {
                // Skip if it's in our proper nouns list
                self.proper_nouns.contains(&word.to_lowercase())
            }
            _ => false,
        }
    }
    
    fn check_word_correctness(&self, word_lower: &str, original_word: &str, word_type: &WordType, dictionary: &Dictionary) -> bool {
        // Check ignore list
        if self.ignore_list.contains(word_lower) {
            return true;
        }
        
        // Check user dictionary
        if self.user_dictionary.contains(word_lower) {
            return true;
        }
        
        // Check cache
        let cache_key = format!("{}_{}", self.current_language.code(), word_lower);
        if let Some(cached) = self.cache.get(&cache_key) {
            return *cached;
        }
        
        // Check main dictionary
        let in_dictionary = dictionary.contains(original_word, self.case_sensitive);
        
        // For proper nouns and acronyms, be more lenient
        let is_correct = match word_type {
            WordType::ProperNoun | WordType::Acronym => {
                // Check if it looks reasonable
                in_dictionary || self.looks_reasonable(original_word)
            }
            WordType::CodeIdentifier => {
                // For code, we're more lenient
                in_dictionary || original_word.len() <= 15
            }
            _ => in_dictionary,
        };
        
        self.cache.insert(cache_key, is_correct);
        is_correct
    }
    
    fn looks_reasonable(&self, word: &str) -> bool {
        // Check if a word looks like a reasonable proper noun or acronym
        if word.is_empty() {
            return false;
        }
        
        // Check character composition
        let letters = word.chars().filter(|c| c.is_alphabetic()).count();
        let total = word.chars().count();
        
        if total == 0 {
            return false;
        }
        
        let letter_ratio = letters as f32 / total as f32;
        
        // Should be mostly letters
        letter_ratio > 0.7 &&
        // Shouldn't have weird character repetitions
        !has_repeated_characters(word, 4) &&
        // Should have vowel-consonant mix for longer words
        (word.len() <= 4 || has_vowels(word))
    }
    
    fn calculate_confidence(&self, word: &str, word_type: &WordType, is_correct: bool) -> f32 {
        if is_correct {
            return 1.0;
        }
        
        let mut confidence = 0.5; // Base confidence
        
        // Adjust based on word characteristics
        match word_type {
            WordType::Normal => confidence *= 1.2,
            WordType::CodeIdentifier => confidence *= 0.3, // Low confidence for code
            WordType::Acronym => confidence *= 0.4,
            WordType::ProperNoun => confidence *= 0.6,
            WordType::TechnicalTerm => confidence *= 0.8,
        }
        
        // Adjust based on word length
        if word.len() < 3 {
            confidence *= 0.3; // Very short words are hard to judge
        } else if word.len() > 20 {
            confidence *= 0.7; // Very long words might be technical
        }
        
        // Adjust based on character patterns
        if word.contains('_') || word.contains('-') {
            confidence *= 1.1; // Compound words are more likely to be correct
        }
        
        // Check for common typo patterns
        if has_common_typo_patterns(word) {
            confidence *= 1.3;
        }
        
        confidence.min(1.0).max(0.0)
    }
    
    // ... rest of the methods remain similar but updated to use new logic ...
}

fn has_repeated_characters(word: &str, max_repeats: usize) -> bool {
    let chars: Vec<char> = word.chars().collect();
    let mut current_char = ' ';
    let mut current_count = 0;
    
    for &c in &chars {
        if c == current_char {
            current_count += 1;
            if current_count > max_repeats {
                return true;
            }
        } else {
            current_char = c;
            current_count = 1;
        }
    }
    
    false
}

fn has_vowels(word: &str) -> bool {
    let vowels = ['a', 'e', 'i', 'o', 'u', 'y', 'A', 'E', 'I', 'O', 'U', 'Y'];
    word.chars().any(|c| vowels.contains(&c))
}

fn has_common_typo_patterns(word: &str) -> bool {
    let common_patterns = [
        "ie", "ei", // i before e except after c
        "tion", "sion", // Common endings
        "able", "ible", // Common suffixes
        "ment", "ness", // More suffixes
        "ough", // Tricky spelling
    ];
    
    common_patterns.iter().any(|pattern| word.contains(pattern))
}