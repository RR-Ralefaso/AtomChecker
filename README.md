# AtomSpell - Atom IDE Inspired Spell Checker

A modern, multilingual spell checker with a beautiful Atom IDE-inspired interface.

## Features

- **Multi-language Support**: Check spelling in 10+ languages
- **Real-time Checking**: Auto-check as you type
- **Smart Suggestions**: Intelligent word suggestions
- **Atom IDE Themes**: Multiple beautiful themes
- **Document Statistics**: Word frequency, accuracy, reading time
- **Dictionary Management**: Add custom words and dictionaries
- **Find & Replace**: Powerful text search and replace
- **Drag & Drop**: Open files by dragging them onto the app

## Supported Languages

- English 🇬🇧
- Afrikaans 🇿🇦
- French 🇫🇷
- Spanish 🇪🇸
- German 🇩🇪
- Chinese 🇨🇳
- Italian 🇮🇹
- Portuguese 🇵🇹
- Russian 🇷🇺
- Japanese 🇯🇵
- Korean 🇰🇷
- Auto-detect 🌐

## Testers

-**[Minenhle Majozi](mailto:Minenhlemajozi11@gmail.com)**

-**[Belinda Mambo](mailto:nyashabelinda85@gmail.com)**

## Installation

### From Source

1. Clone the repository:

```bash
git clone https://github.com/RR-Ralefaso/SpellChecker.git
cd SpellChecker

building windows
=================
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
//should find it in .target/release/x86_64-pc-windows-gnu

making linux portable version
=============================
mkdir -p ~/Desktop/SpellCheckerPortable
cp ./target/release/spellchecker ~/Desktop/SpellCheckerPortable/
cp -r assets/ ~/Desktop/SpellCheckerPortable/
cp -r src/dictionary/ ~/Desktop/SpellCheckerPortable/spellchecker.exe
