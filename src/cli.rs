#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use colored::*;
#[cfg(feature = "cli")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "cli")]
use spellchecker::{checker::SpellChecker, language::Language, util::*};
#[cfg(feature = "cli")]
use std::path::PathBuf;

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "spellchecker-cli")]
#[command(about = "Command-line spell checker", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Check spelling in a file
    Check {
        /// Input file to check
        file: PathBuf,
        
        /// Language to use (eng, afr, fra, etc.)
        #[arg(short, long, default_value = "eng")]
        language: String,
        
        /// Output suggestions
        #[arg(short, long)]
        suggest: bool,
        
        /// Show statistics
        #[arg(long)]
        stats: bool,
        
        /// Case sensitive checking
        #[arg(short = 'c', long)]
        case_sensitive: bool,
        
        /// Output JSON format
        #[arg(long)]
        json: bool,
    },
    
    /// Analyze word frequency
    Frequency {
        /// Input file to analyze
        file: PathBuf,
        
        /// Number of top words to show
        #[arg(short, long, default_value_t = 10)]
        top: usize,
        
        /// Language for word extraction
        #[arg(short, long, default_value = "eng")]
        language: String,
        
        /// Output JSON format
        #[arg(long)]
        json: bool,
    },
    
    /// Create a dictionary from a text file
    CreateDict {
        /// Input text file
        input: PathBuf,
        
        /// Output dictionary file
        output: PathBuf,
        
        /// Language code
        #[arg(short, long, default_value = "eng")]
        lang: String,
        
        /// Minimum word length
        #[arg(short = 'm', long, default_value_t = 3)]
        min_length: usize,
        
        /// Output format (csv or txt)
        #[arg(short = 'f', long, default_value = "csv")]
        format: String,
    },
    
    /// Check spelling from stdin
    Stdin {
        /// Language to use
        #[arg(short, long, default_value = "eng")]
        language: String,
        
        /// Output suggestions
        #[arg(short, long)]
        suggest: bool,
        
        /// Output JSON format
        #[arg(long)]
        json: bool,
    },
    
    /// Interactive mode
    Interactive {
        /// Language to use
        #[arg(short, long, default_value = "eng")]
        language: String,
    },
}

#[cfg(feature = "cli")]
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Check { file, language, suggest, stats, case_sensitive, json } => {
            let content = std::fs::read_to_string(&file)?;
            let language = Language::from_code(&language);
            
            if !json {
                println!("{}", format!("Checking '{}' in {}...", file.display(), language.name()).bold());
                println!("{}", "-".repeat(50));
            }
            
            let mut checker = SpellChecker::new(language)?;
            checker.enable_suggestions(suggest);
            checker.set_case_sensitive(case_sensitive);
            
            let analysis = checker.check_document(&content, Some(&file.to_string_lossy()));
            
            if json {
                println!("{}", serde_json::to_string_pretty(&analysis)?);
            } else {
                println!("\n{}", "Results:".bold().underline());
                println!("  📊 Total words: {}", analysis.total_words);
                println!("  🔤 Unique words: {}", analysis.unique_words);
                println!("  ❌ Misspelled: {}", analysis.misspelled_words);
                println!("  ✅ Accuracy: {:.1}%", analysis.accuracy);
                println!("  ⚡ Check time: {}ms", analysis.check_duration_ms);
                
                if analysis.misspelled_words > 0 {
                    println!("\n{}", "Errors found:".red().bold().underline());
                    for word in analysis.words.iter().filter(|w| !w.is_correct) {
                        println!("\n  Line {}: '{}'", word.line, word.word.red().bold());
                        if suggest && !word.suggestions.is_empty() {
                            println!("    💡 Suggestions: {}", word.suggestions.join(", ").green());
                        }
                    }
                    println!("\n{}", format!("Total errors: {}", analysis.misspelled_words).red());
                } else if analysis.total_words > 0 {
                    println!("\n{}", "✓ No spelling errors found!".green().bold());
                }
                
                if stats {
                    let reading_time = reading_time(&content);
                    let is_cjk = matches!(language, Language::Chinese | Language::Japanese | Language::Korean);
                    let is_code = is_code_file(&file.to_string_lossy());
                    let freq = word_frequency(&content, is_cjk, is_code);
                    let common = most_common_words(&freq, 5);
                    
                    println!("\n{}", "Statistics:".bold().underline());
                    println!("  ⏱️  Reading time: {} min {} sec", reading_time.0, reading_time.1);
                    println!("  🔤 Unique words: {}", freq.len());
                    println!("  📈 Most common words:");
                    for (word, count) in common {
                        println!("    • {}: {}", word.cyan(), count);
                    }
                    println!("  📚 Dictionary size: {} words", checker.word_count());
                }
            }
        }
        
        Commands::Frequency { file, top, language, json } => {
            let content = std::fs::read_to_string(&file)?;
            let lang = Language::from_code(&language);
            let is_cjk = matches!(lang, Language::Chinese | Language::Japanese | Language::Korean);
            let is_code = is_code_file(&file.to_string_lossy());
            let freq = word_frequency(&content, is_cjk, is_code);
            let common = most_common_words(&freq, top);
            
            if json {
                let result = serde_json::json!({
                    "file": file.to_string_lossy(),
                    "language": lang.name(),
                    "total_unique_words": freq.len(),
                    "total_word_count": freq.values().sum::<usize>(),
                    "top_words": common
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{}", format!("Top {} words in '{}':", top, file.display()).bold());
                println!("{}", "=".repeat(50));
                println!("{:<25} {:>15}", "Word", "Frequency");
                println!("{}", "-".repeat(50));
                
                for (word, count) in common {
                    println!("{:<25} {:>15}", word, count.to_string().yellow());
                }
                
                let total_words: usize = freq.values().sum();
                println!("{}", "=".repeat(50));
                println!("{:<25} {:>15}", "Total unique words:", freq.len().to_string().green());
                println!("{:<25} {:>15}", "Total word count:", total_words.to_string().green());
                
                if total_words > 0 {
                    let reading_time = reading_time(&content);
                    println!("{:<25} {:>15}", "Reading time:", format!("{}m {}s", reading_time.0, reading_time.1).blue());
                }
            }
        }
        
        Commands::CreateDict { input, output, lang, min_length, format } => {
            let content = std::fs::read_to_string(&input)?;
            let language = Language::from_code(&lang);
            let is_cjk = matches!(language, Language::Chinese | Language::Japanese | Language::Korean);
            let is_code = is_code_file(&input.to_string_lossy());
            let words = extract_words(&content, is_cjk, is_code);
            let unique_words: std::collections::HashSet<_> = words
                .into_iter()
                .filter(|w| w.len() >= min_length)
                .collect();
            
            let pb = ProgressBar::new(unique_words.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            
            match format.as_str() {
                "csv" => {
                    let mut writer = csv::Writer::from_path(&output)?;
                    for word in unique_words {
                        writer.write_record(&[&word])?;
                        pb.inc(1);
                    }
                    writer.flush()?;
                }
                "txt" => {
                    let mut content = String::new();
                    for word in unique_words {
                        content.push_str(&word);
                        content.push('\n');
                        pb.inc(1);
                    }
                    std::fs::write(&output, content)?;
                }
                _ => {
                    return Err(anyhow::anyhow!("Unsupported format: {}", format));
                }
            }
            
            pb.finish_with_message("Dictionary created!");
            
            println!("✅ Created dictionary '{}'", output.display());
            println!("   Language: {}", language.name());
            println!("   Words: {}", unique_words.len());
            println!("   Source: {}", input.display());
            println!("   Min word length: {}", min_length);
            println!("   Format: {}", format);
        }
        
        Commands::Stdin { language, suggest, json } => {
            use std::io::{self, Read};
            
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            
            if content.trim().is_empty() {
                eprintln!("No input provided");
                return Ok(());
            }
            
            let language = Language::from_code(&language);
            let mut checker = SpellChecker::new(language)?;
            checker.enable_suggestions(suggest);
            
            let analysis = checker.check_document(&content, None);
            
            if json {
                println!("{}", serde_json::to_string_pretty(&analysis)?);
            } else {
                println!("{}", "Spell Check Results:".bold());
                println!("Language: {}", language.name());
                println!("Words checked: {}", analysis.total_words);
                println!("Unique words: {}", analysis.unique_words);
                println!("Errors found: {}", analysis.misspelled_words);
                println!("Accuracy: {:.1}%", analysis.accuracy);
                
                if analysis.misspelled_words > 0 {
                    println!("\nErrors:");
                    for word in analysis.words.iter().filter(|w| !w.is_correct) {
                        print!("Line {}: '{}'", word.line, word.word.red());
                        if suggest && !word.suggestions.is_empty() {
                            print!(" → {}", word.suggestions.join(", ").green());
                        }
                        println!();
                    }
                }
            }
        }
        
        Commands::Interactive { language } => {
            use std::io::{self, Write};
            
            let language = Language::from_code(&language);
            let mut checker = SpellChecker::new(language)?;
            checker.enable_suggestions(true);
            
            println!("{}", "AtomSpell Interactive Mode".bold().green());
            println!("Language: {}", language.name());
            println!("Type 'quit' or 'exit' to exit");
            println!("Type 'check <text>' to check text");
            println!("Type 'add <word>' to add word to dictionary");
            println!("Type 'ignore <word>' to ignore word");
            println!("{}", "-".repeat(50));
            
            loop {
                print!("> ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                if input == "quit" || input == "exit" {
                    break;
                }
                
                if input.starts_with("check ") {
                    let text = &input[6..];
                    let analysis = checker.check_document(text, None);
                    
                    println!("Words: {}, Errors: {}, Accuracy: {:.1}%", 
                        analysis.total_words, analysis.misspelled_words, analysis.accuracy);
                    
                    if analysis.misspelled_words > 0 {
                        for word in analysis.words.iter().filter(|w| !w.is_correct) {
                            println!("  '{}' at line {}", word.word.red(), word.line);
                            if !word.suggestions.is_empty() {
                                println!("    Suggestions: {}", word.suggestions.join(", ").green());
                            }
                        }
                    }
                } else if input.starts_with("add ") {
                    let word = &input[4..];
                    if let Err(e) = checker.add_word_to_dictionary(word) {
                        println!("Error: {}", e);
                    } else {
                        println!("Added '{}' to dictionary", word.green());
                    }
                } else if input.starts_with("ignore ") {
                    let word = &input[7..];
                    if let Err(e) = checker.ignore_word(word) {
                        println!("Error: {}", e);
                    } else {
                        println!("Ignored '{}' for this session", word.yellow());
                    }
                } else {
                    println!("Unknown command. Type 'help' for commands");
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("CLI feature not enabled. Build with --features cli");
    println!("Example: cargo build --features cli");
    println!("Or: cargo run --bin spellchecker_cli --features cli -- [args]");
}