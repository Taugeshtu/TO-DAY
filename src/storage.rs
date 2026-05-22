use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use directories::ProjectDirs;
use jiff::Zoned;

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub paths: PathConfig,
}

#[derive(Deserialize, Default, Clone)]
pub struct PathConfig {
    pub log_dir: Option<String>,
}

pub fn load_config() -> Config {
    ProjectDirs::from("", "", "to-day")
        .and_then(|proj_dirs| {
            let path = proj_dirs.config_dir().join("config.toml");
            fs::read_to_string(path).ok()
        })
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn default_log_dir(config: &Config) -> PathBuf {
    if let Some(ref dir) = config.paths.log_dir {
        PathBuf::from(dir)
    } else {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join("Catch-all")
    }
}

pub fn log_path(dir: &PathBuf) -> PathBuf {
    let date = Zoned::now().strftime("%Y-%m-%d").to_string();
    dir.join(format!("{}.md", date))
}

/// Extracts up to `n` significant words from `text` by filtering stop words.
/// Returns a kebab-case slug suitable for a filename.
pub fn slug_from_content(text: &str, n: usize) -> String {
    const STOP: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "from", "up", "as", "is", "was", "are", "were",
        "be", "been", "being", "have", "has", "had", "do", "does", "did",
        "will", "would", "could", "should", "may", "might", "shall", "can",
        "not", "no", "nor", "so", "yet", "this", "that", "these", "those",
        "i", "me", "my", "we", "our", "you", "your", "it", "its",
        "he", "she", "they", "them", "their", "what", "which", "who",
        "just", "also", "about", "into", "then", "than", "when", "there",
        "its", "am", "if", "how", "out", "get", "got",
    ];

    let words: Vec<String> = text
        .split_whitespace()
        .filter_map(|w| {
            let clean: String = w.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if clean.len() >= 2 && !STOP.contains(&clean.as_str()) {
                Some(clean)
            } else {
                None
            }
        })
        .take(n)
        .collect();

    if words.is_empty() {
        "note".to_string()
    } else {
        words.join("-")
    }
}

/// Path for a standalone note file — named from content, with HH-MM appended on collision.
pub fn note_path(dir: &PathBuf, text: &str) -> PathBuf {
    let slug = slug_from_content(text, 4);
    let candidate = dir.join(format!("{}.md", slug));
    if candidate.exists() {
        let time = Zoned::now().strftime("%H-%M").to_string();
        dir.join(format!("{}_{}.md", slug, time))
    } else {
        candidate
    }
}

pub fn read_tail(path: &PathBuf, n_lines: usize) -> String {
    match fs::read_to_string(path) {
        Ok(content) if !content.trim().is_empty() => {
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(n_lines);
            lines[start..].join("\n")
        }
        _ => String::from("(no entries yet today)"),
    }
}

pub fn append_entry(path: &PathBuf, text: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let time = Zoned::now().strftime("%H:%M").to_string();
        let _ = writeln!(file, "\n**{}**  {}", time, text.trim());
    }
}

pub fn write_note(path: &PathBuf, text: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).write(true).open(path) {
        let _ = write!(file, "{}", text.trim());
    }
}
