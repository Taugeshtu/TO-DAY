> It's like "TODO", but "TODAY". Get it?

## why
I wasn't keeping a log of what I work on, have done, or plan to do — and that's a problem! Capture fails because organizing loose, rolling items feels hopeless. With a consistent inbox that gets sorted every morning, it doesn't. (good place to inject LLMs btw)

## how
Wayland quick-note input that appends to a daily note file (`{log_dir}/YYYY_MM_DD.md`), or dumps them as `first-four-significant-words.md` into supplied `target_dir`.

![TO-DAY interface](docs/Interface_v1.0.0.png)

## Usage: daily note file
Lives in `{log_dir}/YYYY_MM_DD.md`.

`log_dir` is `~/Catch-all` by default. It can be configured by permanently patching the binary:

```bash
to-day --set-log-dir /path/to/your/notes
```

You append to a daily note by:
- **Ctrl+Enter**: when `target_dir` is not supplied (`to-day` is called without an argument)
- **Alt+Enter**: always goes into daily note

Input is appended in format:

> `**HH:MM** {content}`

## Usage: file into folder

```
to-day {target_dir}
```

- **Ctrl+Enter** — save to `target_dir` as `first-four-significant-words.md`
- **Alt+Enter** — goes into daily note

_(if `target_dir` is a file, to-day will make a sibling)_

---

## Install

Dependencies: 
- [rust installed in your system](https://rust-lang.org/tools/install/)
- `GTK4` and `gtk4-layer-shell` system libraries. On Fedora:
```sh
sudo dnf install gtk4-devel gtk4-layer-shell-devel
```
_(have instructions for your repo? happy to add - make an issue with them!)_

Build & install with cargo:
```bash
cargo install --git https://github.com/Taugeshtu/TouchEdgeGlide --root ~/.local
```

_Alternatively:_
```sh
# navigate to where you want it to live, for example, ~/Applications/Gits
git clone https://github.com/Taugeshtu/TO-DAY
cd TO-DAY
cargo install --path . --root ~/.local
```

This puts `to-day` in `~/.local/bin/`.

#### hook it up to your compositor

Pick a key combo and bind it to `to-day`. Examples:

**Hyprland** (`~/.config/hypr/hyprland.conf`):
```
bind = $mainMod, N, exec, to-day
bind = $mainMod, M, exec, to-day /home/projects/work/thang
```

**Sway** (`~/.config/sway/config`):
```
bindsym $mod+n exec to-day
bindsym $mod+m exec to-day /home/projects/work/thang
```

---

## Version history

#### #future
- [ ] Configurable format
- [ ] Surface an error when file creation fails

#### #v0_5_0
- [x] Better scroll of today's notes
- [x] Folder preview should shows list of markdown files

#### #v0_4_0
- [x] Configurable default path via self-patching binary

#### #v0_3_0
- [x] Configurable default path via `~/.config/to-day/config.toml`

#### #v0_2_0
- [x] Optional `TARGET_FOLDER` argument
- [x] Ctrl+Enter → target (or default); Alt+Enter → always default

#### #v0_1_0
- [x] Works as a concept
