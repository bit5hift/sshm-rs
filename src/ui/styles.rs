use ratatui::style::{Color, Modifier, Style};
use std::sync::RwLock;

use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Global theme store — uses RwLock to allow runtime theme swapping.
// ---------------------------------------------------------------------------

static THEME: RwLock<Option<Theme>> = RwLock::new(None);

/// Install or replace the active theme. Safe to call multiple times.
pub fn init_theme(theme: Theme) {
    if let Ok(mut guard) = THEME.write() {
        *guard = Some(theme);
    }
}

/// Return a clone of the currently active theme.
fn theme() -> Theme {
    THEME
        .read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_default()
}

/// Return the currently active theme (public accessor for event handlers).
pub fn current_theme() -> Theme {
    theme()
}

// ---------------------------------------------------------------------------
// Colour accessors — replace the old `pub const` values.
// ---------------------------------------------------------------------------

pub fn bg() -> Color {
    let t = theme();
    Color::Rgb(t.bg[0], t.bg[1], t.bg[2])
}

pub fn fg() -> Color {
    let t = theme();
    Color::Rgb(t.fg[0], t.fg[1], t.fg[2])
}

pub fn primary() -> Color {
    let t = theme();
    Color::Rgb(t.primary[0], t.primary[1], t.primary[2])
}

pub fn green() -> Color {
    let t = theme();
    Color::Rgb(t.green[0], t.green[1], t.green[2])
}

pub fn red() -> Color {
    let t = theme();
    Color::Rgb(t.red[0], t.red[1], t.red[2])
}

pub fn yellow() -> Color {
    let t = theme();
    Color::Rgb(t.yellow[0], t.yellow[1], t.yellow[2])
}

pub fn muted() -> Color {
    let t = theme();
    Color::Rgb(t.muted[0], t.muted[1], t.muted[2])
}

pub fn cyan() -> Color {
    let t = theme();
    Color::Rgb(t.cyan[0], t.cyan[1], t.cyan[2])
}

pub fn purple() -> Color {
    let t = theme();
    Color::Rgb(t.purple[0], t.purple[1], t.purple[2])
}

#[allow(dead_code)]
pub fn orange() -> Color {
    let t = theme();
    Color::Rgb(t.orange[0], t.orange[1], t.orange[2])
}

pub fn selection_bg() -> Color {
    let t = theme();
    Color::Rgb(t.selection_bg[0], t.selection_bg[1], t.selection_bg[2])
}

pub fn hover_bg() -> Color {
    let t = theme();
    Color::Rgb(t.hover_bg[0], t.hover_bg[1], t.hover_bg[2])
}

// ---------------------------------------------------------------------------
// Composite style helpers (unchanged API, now use functions above).
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn header_style() -> Style {
    Style::default().fg(primary()).add_modifier(Modifier::BOLD)
}

#[allow(dead_code)]
pub fn table_header_style() -> Style {
    Style::default()
        .fg(primary())
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED)
}

#[allow(dead_code)]
pub fn table_row_style() -> Style {
    Style::default().fg(fg()).bg(bg())
}

#[allow(dead_code)]
pub fn table_selected_style() -> Style {
    Style::default()
        .fg(fg())
        .bg(selection_bg())
        .add_modifier(Modifier::BOLD)
}

#[allow(dead_code)]
pub fn multi_selected_style() -> Style {
    Style::default()
        .fg(cyan())
        .bg(Color::Rgb(0x1e, 0x2a, 0x3a))
}

#[allow(dead_code)]
pub fn search_focused_style() -> Style {
    Style::default().fg(primary())
}

#[allow(dead_code)]
pub fn search_unfocused_style() -> Style {
    Style::default().fg(muted())
}

pub fn help_text_style() -> Style {
    Style::default().fg(muted())
}

pub fn status_online_style() -> Style {
    Style::default().fg(green())
}

pub fn status_offline_style() -> Style {
    Style::default().fg(red())
}

pub fn status_unknown_style() -> Style {
    Style::default().fg(muted())
}

pub fn status_connecting_style() -> Style {
    Style::default().fg(yellow())
}

pub fn border_focused_style() -> Style {
    Style::default().fg(primary())
}

pub fn border_unfocused_style() -> Style {
    Style::default().fg(muted())
}

pub fn delete_title_style() -> Style {
    Style::default().fg(red()).add_modifier(Modifier::BOLD)
}

pub fn delete_warning_style() -> Style {
    Style::default().fg(red())
}

pub fn help_key_style() -> Style {
    Style::default().fg(primary()).add_modifier(Modifier::BOLD)
}

pub fn help_desc_style() -> Style {
    Style::default().fg(fg())
}

pub fn help_section_style() -> Style {
    Style::default()
        .fg(primary())
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED)
}

#[allow(dead_code)]
pub fn hover_row_style() -> Style {
    Style::default().fg(fg()).bg(hover_bg())
}

pub fn tag_style(tag: &str) -> Style {
    // Hash-based color palette: each tag gets a consistent color
    // based on its name, regardless of content.
    const PALETTE: &[(u8, u8, u8)] = &[
        (0x8b, 0x22, 0x52), // rose
        (0xb5, 0x65, 0x1d), // orange
        (0x2d, 0x6a, 0x4f), // green
        (0x1a, 0x7f, 0x7f), // teal
        (0x2d, 0x4a, 0x8b), // blue
        (0x6b, 0x3f, 0xa0), // purple
        (0x8b, 0x5c, 0x2a), // brown
        (0x5a, 0x2d, 0x82), // indigo
        (0x7a, 0x30, 0x30), // crimson
        (0x30, 0x6b, 0x70), // cyan
    ];

    let hash = tag.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let idx = (hash as usize) % PALETTE.len();
    let (r, g, b) = PALETTE[idx];

    Style::default().fg(Color::Rgb(0xff, 0xff, 0xff)).bg(Color::Rgb(r, g, b))
}
