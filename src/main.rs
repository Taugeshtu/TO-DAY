use gtk4::{self as gtk, glib, Orientation};
use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use jiff::Zoned;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn default_log_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Catch-all")
}

fn log_path(dir: &PathBuf) -> PathBuf {
    let date = Zoned::now().strftime("%Y-%m-%d").to_string();
    dir.join(format!("{}.md", date))
}

/// Extracts up to `n` significant words from `text` by filtering stop words.
/// Returns a kebab-case slug suitable for a filename.
fn slug_from_content(text: &str, n: usize) -> String {
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
fn note_path(dir: &PathBuf, text: &str) -> PathBuf {
    let slug = slug_from_content(text, 4);
    let candidate = dir.join(format!("{}.md", slug));
    if candidate.exists() {
        let time = Zoned::now().strftime("%H-%M").to_string();
        dir.join(format!("{}_{}.md", slug, time))
    } else {
        candidate
    }
}

fn read_tail(path: &PathBuf, n_lines: usize) -> String {
    match fs::read_to_string(path) {
        Ok(content) if !content.trim().is_empty() => {
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(n_lines);
            lines[start..].join("\n")
        }
        _ => String::from("(no entries yet today)"),
    }
}

fn append_entry(path: &PathBuf, text: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let time = Zoned::now().strftime("%H:%M").to_string();
        let _ = writeln!(file, "\n**{}**  {}", time, text.trim());
    }
}

fn write_note(path: &PathBuf, text: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).write(true).open(path) {
        let _ = write!(file, "{}", text.trim());
    }
}

fn activate(application: &gtk::Application, target_dir: Option<PathBuf>) {
    let window = gtk::ApplicationWindow::new(application);

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_margin(Edge::Top, 80);
    window.set_default_size(640, 300);

    let vbox = gtk::Box::new(Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);

    let default_dir = default_log_dir();
    let primary_dir = target_dir.clone().unwrap_or_else(|| default_dir.clone());

    // --- Tail: last N lines of today's log (primary destination) ---
    let path = log_path(&primary_dir);
    let tail_text = read_tail(&path, 8);

    let tail_scroll = gtk::ScrolledWindow::new();
    tail_scroll.set_max_content_height(110);
    tail_scroll.set_propagate_natural_height(true);
    tail_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let tail_view = gtk::TextView::new();
    tail_view.set_editable(false);
    tail_view.set_cursor_visible(false);
    tail_view.set_wrap_mode(gtk::WrapMode::Word);
    tail_view.set_left_margin(4);
    tail_view.set_top_margin(4);
    tail_view.set_bottom_margin(4);
    tail_view.buffer().set_text(&tail_text);
    tail_scroll.set_child(Some(&tail_view));
    vbox.append(&tail_scroll);

    vbox.append(&gtk::Separator::new(Orientation::Horizontal));

    // --- Input area ---
    let input_scroll = gtk::ScrolledWindow::new();
    input_scroll.set_vexpand(true);
    input_scroll.set_min_content_height(80);
    input_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let text_view = gtk::TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    text_view.set_accepts_tab(false);
    text_view.set_left_margin(4);
    text_view.set_top_margin(4);
    text_view.set_bottom_margin(4);
    input_scroll.set_child(Some(&text_view));
    vbox.append(&input_scroll);

    // --- Hint area ---
    if target_dir.is_some() {
        // Two-line hint: primary destination on top, default below
        let hint_box = gtk::Box::new(Orientation::Vertical, 2);
        hint_box.set_halign(gtk::Align::End);

        let line1 = gtk::Label::new(Some(&format!(
            "Ctrl+Enter  →  {}",
            primary_dir.display()
        )));
        line1.set_halign(gtk::Align::End);
        line1.set_opacity(0.75);

        let line2 = gtk::Label::new(Some(&format!(
            "Alt+Enter   →  {}  ·  Esc  dismiss",
            default_dir.display()
        )));
        line2.set_halign(gtk::Align::End);
        line2.set_opacity(0.4);

        hint_box.append(&line1);
        hint_box.append(&line2);
        vbox.append(&hint_box);
    } else {
        let hint = gtk::Label::new(Some("Ctrl+Enter  save  ·  Esc  dismiss"));
        hint.set_halign(gtk::Align::End);
        hint.set_opacity(0.5);
        vbox.append(&hint);
    }

    // --- Key handling ---
    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);
    {
        let app = application.clone();
        let tv = text_view.clone();
        let default_path = log_path(&default_dir);
        let target = target_dir.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, mods| {
            use gtk4::gdk::{Key, ModifierType};
            match key {
                Key::Escape => {
                    app.quit();
                    glib::Propagation::Stop
                }
                Key::Return if mods.contains(ModifierType::CONTROL_MASK) => {
                    let buf = tv.buffer();
                    let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                    if !text.trim().is_empty() {
                        if let Some(ref dir) = target {
                            // Standalone file named from content
                            write_note(&note_path(dir, &text), &text);
                        } else {
                            append_entry(&default_path, &text);
                        }
                    }
                    app.quit();
                    glib::Propagation::Stop
                }
                Key::Return if mods.contains(ModifierType::ALT_MASK) => {
                    let buf = tv.buffer();
                    let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                    if !text.trim().is_empty() {
                        append_entry(&default_path, &text);
                    }
                    app.quit();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
    }
    window.add_controller(key_ctrl);

    window.set_child(Some(&vbox));
    window.present();
    text_view.grab_focus();
}

fn main() {
    // Grab our positional arg before GTK consumes argv.
    let target_dir: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from);

    let app = gtk::Application::new(Some("games.tau.today"), Default::default());
    app.connect_activate(move |a| activate(a, target_dir.clone()));
    // Pass only the program name so GTK doesn't choke on our path arg.
    app.run_with_args(&["to-day"]);
}
