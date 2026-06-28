use gtk4::{self as gtk, glib, Orientation};
use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use crate::AppState;
use crate::storage::{
    log_path, note_path, read_daily_tail_async, read_folder_notes_tail_async, append_entry_async, write_note_async
};

pub fn activate(application: &gtk::Application, state: AppState) {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "scrolledwindow, textview, textview text { background: transparent; }
         .no-scrollbar scrollbar { display: none; }"
    );
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window = gtk::ApplicationWindow::new(application);

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_margin(Edge::Top, 80);
    window.set_default_size(640, 380);

    let vbox = gtk::Box::new(Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);

    let log_dir = state.log_dir.clone();
    let target_dir = state.target_dir.clone();

    // --- Tail: last N lines of today's log ---
    let daily_log_path = log_path(&log_dir);

    let tail_scroll = gtk::ScrolledWindow::new();
    tail_scroll.set_min_content_height(160);
    tail_scroll.set_max_content_height(160);
    tail_scroll.set_size_request(-1, 160);
    tail_scroll.set_propagate_natural_height(false);
    tail_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    tail_scroll.add_css_class("no-scrollbar");

    let tail_view = gtk::TextView::new();
    tail_view.set_editable(false);
    tail_view.set_cursor_visible(false);
    tail_view.set_wrap_mode(gtk::WrapMode::Word);
    tail_view.set_left_margin(8);
    tail_view.set_top_margin(8);
    tail_view.set_bottom_margin(8);
    tail_view.set_opacity(0.65);
    tail_view.buffer().set_text("(loading tail...)");
    
    // Spawn tail preview loading asynchronously
    {
        let tail_view_clone = tail_view.clone();
        let target_dir_clone = target_dir.clone();
        let daily_log_path_clone = daily_log_path.clone();
        glib::spawn_future_local(async move {
            let preview_text = if let Some(dir) = target_dir_clone {
                read_folder_notes_tail_async(dir, 20).await
            } else {
                read_daily_tail_async(daily_log_path_clone, 8).await
            };
            tail_view_clone.buffer().set_text(&preview_text);

            // Wait 50ms for GTK to layout the text and update vertical adjustment limits
            glib::timeout_future(std::time::Duration::from_millis(50)).await;

            let buffer = tail_view_clone.buffer();
            let end_iter = buffer.end_iter();
            let mark = buffer.create_mark(None, &end_iter, false);
            tail_view_clone.scroll_to_mark(&mark, 0.0, true, 0.0, 1.0);
        });
    }

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
    if let Some(ref target) = target_dir {
        // Two-line hint: primary destination (target folder) on top, default log below
        let hint_box = gtk::Box::new(Orientation::Vertical, 2);
        hint_box.set_halign(gtk::Align::End);

        let line1 = gtk::Label::new(Some(&format!(
            "Ctrl+Enter  →  {}",
            target.display()
        )));
        line1.set_halign(gtk::Align::End);
        line1.set_opacity(0.75);

        let line2 = gtk::Label::new(Some(&format!(
            "Alt+Enter   →  {}  ·  Esc  dismiss",
            log_dir.display()
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
        let daily_log_path = daily_log_path.clone();
        let target_dir = target_dir.clone();
        let window_clone = window.clone();
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
                    let text_str = text.to_string();
                    if !text_str.trim().is_empty() {
                        window_clone.set_visible(false);
                        let app_clone = app.clone();
                        let target_dir_clone = target_dir.clone();
                        let daily_log_path_clone = daily_log_path.clone();
                        glib::spawn_future_local(async move {
                            if let Some(ref dir) = target_dir_clone {
                                let note_p = note_path(dir, &text_str);
                                let _ = write_note_async(note_p, text_str).await;
                            } else {
                                let _ = append_entry_async(daily_log_path_clone, text_str).await;
                            }
                            app_clone.quit();
                        });
                    } else {
                        app.quit();
                    }
                    glib::Propagation::Stop
                }
                Key::Return if mods.contains(ModifierType::ALT_MASK) => {
                    let buf = tv.buffer();
                    let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                    let text_str = text.to_string();
                    if !text_str.trim().is_empty() {
                        window_clone.set_visible(false);
                        let app_clone = app.clone();
                        let daily_log_path_clone = daily_log_path.clone();
                        glib::spawn_future_local(async move {
                            let _ = append_entry_async(daily_log_path_clone, text_str).await;
                            app_clone.quit();
                        });
                    } else {
                        app.quit();
                    }
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
