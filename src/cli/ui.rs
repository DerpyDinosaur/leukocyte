/// Colour palette for leuko
///
/// Examples:
/// ```ignore
/// // Indicatif
/// let style = ProgressStyle::with_template(
///     &format!("{{value.{FG}/{BG}}}")
/// );
/// // owo_colors
/// let foreground = XtermColors::from(GREEN);
/// let background = XtermColors::from(RED);
/// println!("{}", "Hello".color(foreground).on_color(background));
/// ```
/// References:
///
/// https://www.ditig.com/256-colors-cheat-sheet
///
/// Hex -> Xterm: https://hm66hd.csb.app/
pub mod palette {
    /// Lonetree green
    ///
    /// rgb(57,228,126)
    ///
    /// #39e47e
    pub const GREEN: u8 = 78;
    pub const YELLOW: u8 = 221;
    pub const RED: u8 = 167;
    pub const WHITE: u8 = 250;

    /// Soften text
    pub const MUTED: u8 = 240;
}

// ----------------------------------------------------
// Header Spinner
// ----------------------------------------------------
// Current active spinner
pub const SPINNER_CELL_SMALL: [&str; 6] = ["⢾⣉⡷", "⠰⣹⠆", "⠰⣿⠆", "⠀⣿⠀", "⠰⣿⠆", "⠰⣏⠆"];

pub const SPINNER_CELL: [&str; 8] = [
    "⠶⣏⣹⠶",
    "⠰⣏⣹⠆",
    "⠰⣏⣿⠆",
    "⠀⢾⡷⠀",
    "⠀⢸⡇⠀",
    "⠀⢾⡷⠀",
    "⠰⣿⣹⠆",
    "⠰⣏⣹⠆",
];

pub const SPINNER_WAVE: [&str; 16] = [
    "⠣⣄⡔⠋",
    "⢦⣠⠞⠙",
    "⣄⡤⠋⠓",
    "⣄⡴⠋⠳",
    "⣠⠜⠙⢢",
    "⣠⠚⠙⢤",
    "⡴⠋⠳⣄",
    "⡔⠋⠣⣄",
    "⠞⠙⢦⣠",
    "⠋⠓⣄⡤",
    "⠋⠳⣄⡴",
    "⠙⢢⣠⠜",
    "⠙⢦⣠⠞",
    "⠙⢤⣠⠚",
    "⠳⣄⡴⠋",
    "⠣⣄⡔⠋",
];

pub const SPINNER_HELIX: [&str; 5] = ["⠶⣉⣉⠶", "⣉⣉⠶⠶", "⣉⠶⠶⣉", "⠶⠶⣉⣉", "⠶⣉⣉⠶"];

// ----------------------------------------------------
// Loading Bars
// ----------------------------------------------------

pub const PROGRESS_DNA: [&str; 12] = [
    "⠋⠉⠙⠚",
    "⠉⠙⠚⠒",
    "⠙⠚⠒⠂",
    "⠚⠒⠂⠂",
    "⠒⠂⠂⠒",
    "⠂⠂⠒⠲",
    "⠂⠒⠲⠴",
    "⠒⠲⠴⠤",
    "⠲⠴⠤⠄",
    "⠴⠤⠄⠋",
    "⠤⠄⠋⠉",
    "⠄⠋⠉⠙",
];
