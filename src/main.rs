use gtk4::{self as gtk, glib, Orientation};
use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use jiff::Zoned;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn log_path() -> PathBuf {
    let date = Zoned::now().strftime("%Y-%m-%d").to_string();
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Catch-all").join(format!("{}.md", date))
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

fn activate(application: &gtk::Application) {
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

    // --- Tail: last N lines of today's log ---
    let path = log_path();
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

    // --- Hint ---
    let hint = gtk::Label::new(Some("Ctrl+Enter  save  ·  Esc  dismiss"));
    hint.set_halign(gtk::Align::End);
    hint.set_opacity(0.5);
    vbox.append(&hint);

    // --- Key handling ---
    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);
    {
        let app = application.clone();
        let tv = text_view.clone();
        let p = path.clone();
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
                        append_entry(&p, &text);
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
    let app = gtk::Application::new(Some("games.tau.today"), Default::default());
    app.connect_activate(|a| activate(a));
    app.run();
}
