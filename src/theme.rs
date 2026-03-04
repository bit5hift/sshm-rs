use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub bg: [u8; 3],
    pub fg: [u8; 3],
    pub primary: [u8; 3],
    pub green: [u8; 3],
    pub red: [u8; 3],
    pub yellow: [u8; 3],
    pub muted: [u8; 3],
    pub cyan: [u8; 3],
    pub purple: [u8; 3],
    pub orange: [u8; 3],
    pub selection_bg: [u8; 3],
}

impl Theme {
    #[allow(dead_code)]
    pub fn color(&self, rgb: [u8; 3]) -> Color {
        Color::Rgb(rgb[0], rgb[1], rgb[2])
    }

    /// Tokyo Night — the original hardcoded palette.
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            bg: [0x1a, 0x1b, 0x26],
            fg: [0xc0, 0xca, 0xf5],
            primary: [0x7a, 0xa2, 0xf7],
            green: [0x9e, 0xce, 0x6a],
            red: [0xf7, 0x76, 0x8e],
            yellow: [0xe0, 0xaf, 0x68],
            muted: [0x73, 0x7a, 0xa2],
            cyan: [0x7d, 0xcf, 0xff],
            purple: [0xbb, 0x9a, 0xf7],
            orange: [0xff, 0x9e, 0x64],
            selection_bg: [0x36, 0x4a, 0x82],
        }
    }

    /// High Contrast — pure black background, bright ANSI-style colors.
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            bg: [0x00, 0x00, 0x00],
            fg: [0xff, 0xff, 0xff],
            primary: [0x00, 0xaf, 0xff],
            green: [0x00, 0xff, 0x00],
            red: [0xff, 0x00, 0x00],
            yellow: [0xff, 0xff, 0x00],
            muted: [0x80, 0x80, 0x80],
            cyan: [0x00, 0xff, 0xff],
            purple: [0xff, 0x00, 0xff],
            orange: [0xff, 0x87, 0x00],
            selection_bg: [0x00, 0x5f, 0xaf],
        }
    }

    /// Light — dark text on a light background.
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            bg: [0xf8, 0xf8, 0xf2],
            fg: [0x28, 0x28, 0x28],
            primary: [0x00, 0x5f, 0xd7],
            green: [0x00, 0x87, 0x00],
            red: [0xc0, 0x00, 0x00],
            yellow: [0x87, 0x5f, 0x00],
            muted: [0x60, 0x60, 0x60],
            cyan: [0x00, 0x5f, 0x87],
            purple: [0x5f, 0x00, 0xaf],
            orange: [0xd7, 0x5f, 0x00],
            selection_bg: [0xaf, 0xd7, 0xff],
        }
    }

    /// Load a theme from `~/.config/sshm-rs/theme.json` (or platform equivalent).
    /// Falls back to Tokyo Night if the file does not exist or cannot be parsed.
    pub fn load() -> Self {
        let config_dir = crate::config::sshm_config_dir().ok();
        if let Some(dir) = config_dir {
            let path = dir.join("theme.json");
            if path.exists() {
                if let Ok(data) = std::fs::read_to_string(&path) {
                    if let Ok(theme) = serde_json::from_str::<Theme>(&data) {
                        return theme;
                    }
                }
            }
        }
        Self::tokyo_night()
    }

    /// All built-in themes, in cycle order.
    pub fn builtin_themes() -> Vec<Self> {
        vec![Self::tokyo_night(), Self::high_contrast(), Self::light()]
    }
}
