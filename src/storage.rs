use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use jiff::Zoned;
use gio::prelude::*;

const MAGIC: &[u8; 8] = b"TODAYCFG";

pub fn read_embedded_config() -> Option<String> {
    let exe_path = std::env::current_exe().ok()?;
    let mut file = File::open(&exe_path).ok()?;
    let file_len = file.metadata().ok()?.len();
    if file_len < 12 {
        return None;
    }
    
    // Read magic signature at the end
    file.seek(SeekFrom::End(-8)).ok()?;
    let mut magic_buf = [0u8; 8];
    file.read_exact(&mut magic_buf).ok()?;
    if &magic_buf != MAGIC {
        return None;
    }
    
    // Read length
    file.seek(SeekFrom::End(-12)).ok()?;
    let mut len_buf = [0u8; 4];
    file.read_exact(&mut len_buf).ok()?;
    let len = u32::from_le_bytes(len_buf) as u64;
    
    if file_len < 12 + len {
        return None;
    }
    
    // Read config string
    file.seek(SeekFrom::End(-(12 + len as i64))).ok()?;
    let mut config_buf = vec![0u8; len as usize];
    file.read_exact(&mut config_buf).ok()?;
    
    String::from_utf8(config_buf).ok()
}

pub fn patch_embedded_config(new_config: &str) -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let mut file_bytes = std::fs::read(&exe_path)?;
    
    let file_len = file_bytes.len();
    let mut payload_len = 0;
    if file_len >= 12 {
        let magic_start = file_len - 8;
        if &file_bytes[magic_start..] == MAGIC {
            let len_start = file_len - 12;
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&file_bytes[len_start..magic_start]);
            let len = u32::from_le_bytes(len_bytes) as usize;
            if file_len >= 12 + len {
                payload_len = 12 + len;
            }
        }
    }
    
    if payload_len > 0 {
        file_bytes.truncate(file_len - payload_len);
    }
    
    let config_bytes = new_config.as_bytes();
    let config_len = config_bytes.len() as u32;
    file_bytes.extend_from_slice(config_bytes);
    file_bytes.extend_from_slice(&config_len.to_le_bytes());
    file_bytes.extend_from_slice(MAGIC);
    
    let tmp_path = exe_path.with_extension("tmp_patch");
    {
        let mut tmp_file = File::create(&tmp_path)?;
        tmp_file.write_all(&file_bytes)?;
    }
    
    let metadata = std::fs::metadata(&exe_path)?;
    std::fs::set_permissions(&tmp_path, metadata.permissions())?;
    
    std::fs::rename(&tmp_path, &exe_path)?;
    Ok(())
}

pub fn default_log_dir(embedded_config: &Option<String>) -> PathBuf {
    if let Some(dir) = embedded_config {
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

fn gio_file_from_path(path: &Path) -> gio::File {
    gio::File::for_path(path)
}

pub async fn read_tail_async(path: PathBuf, n_lines: usize) -> String {
    let file = gio_file_from_path(&path);
    match file.load_contents_future().await {
        Ok((contents, _)) if !contents.is_empty() => {
            if let Ok(content_str) = std::str::from_utf8(&contents) {
                let trimmed = content_str.trim();
                if !trimmed.is_empty() {
                    let lines: Vec<&str> = trimmed.lines().collect();
                    let start = lines.len().saturating_sub(n_lines);
                    return lines[start..].join("\n");
                }
            }
            String::from("(no entries yet today)")
        }
        _ => String::from("(no entries yet today)"),
    }
}

fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

pub async fn append_entry_async(path: PathBuf, text: String) -> Result<(), gio::glib::Error> {
    ensure_parent_dir(&path);
    let file = gio_file_from_path(&path);
    
    // Read the existing content first
    let mut current_bytes = Vec::new();
    if let Ok((contents, _)) = file.load_contents_future().await {
        current_bytes.extend_from_slice(&contents);
    }
    
    // Format the new entry
    let time = Zoned::now().strftime("%H:%M").to_string();
    let new_entry = format!("\n**{}**  {}\n", time, text.trim());
    current_bytes.extend_from_slice(new_entry.as_bytes());
    
    // Write back
    let _ = file.replace_contents_future(
        current_bytes,
        None,
        false,
        gio::FileCreateFlags::REPLACE_DESTINATION,
    ).await.map_err(|(_, err)| err)?;
    
    Ok(())
}

pub async fn write_note_async(path: PathBuf, text: String) -> Result<(), gio::glib::Error> {
    ensure_parent_dir(&path);
    let file = gio_file_from_path(&path);
    
    let content_bytes = text.trim().as_bytes().to_vec();
    let _ = file.replace_contents_future(
        content_bytes,
        None,
        false,
        gio::FileCreateFlags::REPLACE_DESTINATION,
    ).await.map_err(|(_, err)| err)?;
    
    Ok(())
}
