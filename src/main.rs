use std::path::PathBuf;
use gio::prelude::*;

mod storage;
mod ui;

#[derive(Clone)]
pub struct AppState {
    pub target_dir: Option<PathBuf>,
    pub log_dir: PathBuf,
}

fn make_absolute(path: &str) -> String {
    let p = if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(&path[2..])
        } else {
            PathBuf::from(path)
        }
    } else if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    };

    if p.is_absolute() {
        p.to_string_lossy().into_owned()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(p).to_string_lossy().into_owned(),
            Err(_) => p.to_string_lossy().into_owned(),
        }
    }
}

fn print_help() {
    println!(
        "TO-DAY: A Wayland quick-note input utility.

Usage:
  to-day [TARGET_DIR]
      Launches the quick-note input window.
      If TARGET_DIR is supplied (or a file, which writes to its parent directory),
      Ctrl+Enter saves to TARGET_DIR as a slug-named file: `first-four-significant-words.md`.
      Alt+Enter always appends to the daily note.
      If TARGET_DIR is not supplied, Ctrl+Enter appends to the daily note.

Configuration:
  The daily note directory (where daily YYYY-MM-DD.md logs are stored)
  can be configured by permanently patching the executable path:

     to-day --set-log-dir <PATH>
         Patches the running binary to permanently use <PATH> as the log directory.

  Fallback:
     Defaults to `~/Catch-all`.

Keys:
  Ctrl+Enter - Save note to target destination (or append to daily log)
  Alt+Enter  - Force append to daily log
  Esc        - Dismiss window"
    );
}

fn main() {
    if std::env::var("GSK_RENDERER").is_err() {
        unsafe {
            std::env::set_var("GSK_RENDERER", "cairo");
        }
    }
    let args: Vec<String> = std::env::args().collect();

    // Check if `--help` or `-h` is passed
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        std::process::exit(0);
    }
    
    // Check if `--set-log-dir` is passed
    let mut set_log_dir_val = None;
    let mut idx = 1;
    while idx < args.len() {
        if args[idx] == "--set-log-dir" {
            if idx + 1 < args.len() {
                set_log_dir_val = Some(args[idx + 1].clone());
                idx += 2;
            } else {
                eprintln!("Error: --set-log-dir requires a path value");
                std::process::exit(1);
            }
        } else if args[idx].starts_with("--set-log-dir=") {
            set_log_dir_val = Some(args[idx].split_at(14).1.to_string());
            idx += 1;
        } else {
            idx += 1;
        }
    }

    if let Some(val) = set_log_dir_val {
        let abs_val = make_absolute(&val);
        println!("Patching embedded log directory to: {}", abs_val);
        match storage::patch_embedded_config(&abs_val) {
            Ok(_) => {
                println!("Successfully patched binary!");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error patching binary: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Grab target directory if passed (skip flags)
    let mut target_dir: Option<PathBuf> = None;
    for arg in args.iter().skip(1) {
        if !arg.starts_with('-') {
            let path = PathBuf::from(arg);
            if path.is_file() {
                target_dir = path.parent().map(|p| {
                    if p.as_os_str().is_empty() {
                        PathBuf::from(".")
                    } else {
                        p.to_path_buf()
                    }
                }).or_else(|| Some(PathBuf::from(".")));
            } else {
                target_dir = Some(path);
            }
            break;
        }
    }

    let embedded_config = storage::read_embedded_config();
    let log_dir = storage::default_log_dir(&embedded_config);
    let state = AppState {
        target_dir,
        log_dir,
    };

    let app = gtk4::Application::new(Some("games.tau.today"), Default::default());
    app.connect_activate(move |a| ui::activate(a, state.clone()));
    // Pass only the program name so GTK doesn't choke on our path arg.
    app.run_with_args(&["to-day"]);
}
