# TO-DAY
It's like "TODO", but "TODAY". Get it?

Wayland quick-note input that appends jotted-down notes into `~/Catch-all/YYYY_MM_DD.md` in a `**HH:MM** {content}` format:

![TO-DAY interface](docs/Interface_v1.0.0.png)

### Why
I don't keep a log of what I work on, have done, or plan to do — and that's a problem. Capture fails because organizing loose, rolling items feels hopeless. With an LLM-assisted inbox that sorts them every morning, it doesn't.

---

# Install

### Dependencies

Requires GTK4 and gtk4-layer-shell system libraries. On Fedora:
```sh
sudo dnf install gtk4-devel gtk4-layer-shell-devel
```
(have instructions for your repo? happy to add - make an issue with them!)

### Build and install

```sh
git clone <repo-url>
cd TO-DAY
cargo install --path . --root ~/.local
```

This puts `today` in `~/.local/bin/` — make sure it's on your `$PATH`:
```sh
# in ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"
```

### Hook it up to your compositor

Pick a key combo and bind it to `to-day`. Examples:

**Hyprland** (`~/.config/hypr/hyprland.conf`):
```
bind = $mainMod, N, exec, to-day
```

**Sway** (`~/.config/sway/config`):
```
bindsym $mod+n exec to-day
```

---

# Version history

#### #future
- [ ] Configurable paths and formats
- [ ] Better scroll of today's notes

#### #v0_1_0
- [x] Works as a concept
