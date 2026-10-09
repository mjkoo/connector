use std::io;
use std::path::Path;

pub const CONFIG_DIR: &str = "Config";

const RENAMED: [(&str, &str); 1] = [("slot.state", "connector.state")];

const MOVED: [(&str, &str); 5] = [
    ("slot.state", "connector.state"),
    ("selected_core.ini", "selected_core.ini"),
    ("video_mode.ini", "video_mode.ini"),
    ("cart_shell.ini", "cart_shell.ini"),
    ("theme.txt", "theme.txt"),
];

pub fn move_config(root: &Path) -> io::Result<()> {
    let config = root.join(CONFIG_DIR);
    std::fs::create_dir_all(&config)?;
    let renamed = RENAMED.map(|(from, to)| (config.join(from), config.join(to)));
    let moved = MOVED.map(|(from, to)| (root.join("System").join(from), config.join(to)));
    for (old, new) in renamed.iter().chain(&moved) {
        if old.is_file() && !new.exists() {
            if let Err(e) = std::fs::rename(old, new) {
                eprintln!(
                    "connector: config: {} -> {}: {e}",
                    old.display(),
                    new.display()
                );
            }
        }
    }
    Ok(())
}
