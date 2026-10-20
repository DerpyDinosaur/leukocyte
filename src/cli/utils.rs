use console::Style;

/// Cell Layout:
/// 1        4
/// 2        5
/// 3        6
/// 7        8
///
/// Reference:
/// https://en.wikipedia.org/wiki/Braille_Patterns
pub mod dot8 {
    pub const D1: u8 = 0x01;
    pub const D2: u8 = 0x02;
    pub const D3: u8 = 0x04;
    pub const D4: u8 = 0x08;
    pub const D5: u8 = 0x10;
    pub const D6: u8 = 0x20;
    pub const D7: u8 = 0x40;
    pub const D8: u8 = 0x80;
}

/// Apply dim style to string
///
/// Example:
/// ```
/// let progress_bar = multi_progress.add(ProgressBar::new_spinner());
/// progress_bar.set_message(muted().apply_to("hello world").to_string());
/// ```
pub fn muted() -> Style {
    Style::new().color256(244)
}
