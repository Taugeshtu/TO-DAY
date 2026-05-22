use std::path::PathBuf;
use gio::prelude::*;

mod storage;
mod ui;

#[derive(Clone)]
pub struct AppState {
    pub target_dir: Option<PathBuf>,
    pub config: storage::Config,
}

fn main() {
    // Grab our positional arg before GTK consumes argv.
    let mut target_dir: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from);
    if let Some(ref path) = target_dir {
        if path.is_file() {
            target_dir = path.parent().map(|p| {
                if p.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    p.to_path_buf()
                }
            }).or_else(|| Some(PathBuf::from(".")));
        }
    }
    let config = storage::load_config();
    let state = AppState {
        target_dir,
        config,
    };

    let app = gtk4::Application::new(Some("games.tau.today"), Default::default());
    app.connect_activate(move |a| ui::activate(a, state.clone()));
    // Pass only the program name so GTK doesn't choke on our path arg.
    app.run_with_args(&["to-day"]);
}
