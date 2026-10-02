use ratatui::style::Color;

pub struct Theme;

impl Theme {
    pub const PRIMARY: Color = Color::Rgb(255, 182, 193); // Light pink
    pub const SECONDARY: Color = Color::Rgb(152, 224, 237); // Light cyan
    pub const ACCENT: Color = Color::Rgb(218, 179, 255); // Lavender
    pub const SUCCESS: Color = Color::Rgb(144, 238, 144); // Light green
    pub const WARNING: Color = Color::Rgb(255, 215, 0); // Soft gold
    pub const MUTED: Color = Color::Rgb(128, 128, 128); // Muted gray
    pub const TEXT: Color = Color::Rgb(248, 248, 242); // Crisp white
}
