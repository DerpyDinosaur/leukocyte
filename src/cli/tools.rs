use owo_colors::{Style, XtermColors};

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

/// Apply colours using owo_colors to strings
///
/// Example:
/// ```ignore
/// println!("{}", String::from("name")).style(paintx(MUTED)));
/// ```
pub fn paintx(index: u8) -> Style {
    Style::new().color(XtermColors::from(index))
}
