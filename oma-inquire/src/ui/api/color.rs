/// Represents a color to be used for text styling purposes.
///
/// Currently a clone of [crossterm::style::Color]. Check their documentation
/// for detailed documentation.
///
/// In summary, the 16 defined colors are supported by almost all terminals.
/// The Rgb and AnsiValue variants are supported in more modern ones.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// Black color.
    ///
    /// Ansi code reference: 0
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    Black,

    /// Light red color.
    ///
    /// Ansi code reference: 9
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `DarkRed` color (Ansi code reference 1).
    LightRed,

    /// Dark red color.
    ///
    /// Ansi code reference: 1
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    DarkRed,

    /// Light green color.
    ///
    /// Ansi code reference: 10
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `DarkGreen` color (Ansi code reference 2).
    LightGreen,

    /// Dark green color.
    ///
    /// Ansi code reference: 2
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    DarkGreen,

    /// Light yellow color.
    ///
    /// Ansi code reference: 11
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `DarkYellow` color (Ansi code reference 3).
    LightYellow,

    /// Dark yellow color.
    ///
    /// Ansi code reference: 3
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    DarkYellow,

    /// Light blue color.
    ///
    /// Ansi code reference: 12
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `DarkBlue` color (Ansi code reference 4).
    LightBlue,

    /// Dark blue color.
    ///
    /// Ansi code reference: 4
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    DarkBlue,

    /// Light magenta color.
    ///
    /// Ansi code reference: 13
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `DarkMagenta` color (Ansi code reference 5).
    LightMagenta,

    /// Dark magenta color.
    ///
    /// Ansi code reference: 5
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    DarkMagenta,

    /// Light cyan color.
    ///
    /// Ansi code reference: 14
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `DarkCyan` color (Ansi code reference 6).
    LightCyan,

    /// Dark cyan color.
    ///
    /// Ansi code reference: 6
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    DarkCyan,

    /// White color.
    ///
    /// Ansi code reference: 15
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `Grey` color (Ansi code reference 7).
    White,

    /// Grey color.
    ///
    /// Ansi code reference: 7
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    Grey,

    /// Dark grey color.
    ///
    /// Ansi code reference: 8
    ///
    /// Supported on two terminal back-ends: `crossterm` (the default) and `termion`.
    /// On `console`, it is mapped to the `Black` color (Ansi code reference 0).
    DarkGrey,

    /// An RGB color. See [RGB color model](https://en.wikipedia.org/wiki/RGB_color_model) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes) for more info.
    ///
    /// Supported on the default terminal back-end `crossterm` and on `termion`.
    /// Not supported on `console`.
    Rgb {
        /// red value of RGB.
        r: u8,

        /// green value of RGB.
        g: u8,

        /// blue value of RGB.
        b: u8,
    },

    /// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes) for more info.
    ///
    /// Supported on all terminal back-ends: `crossterm`, `termion` and `console`.
    AnsiValue(u8),
}

impl Color {
    /// Shorthand method for creating a Color from RGB components
    ///
    /// ```
    /// # use inquire::ui::Color;
    ///
    /// assert_eq!(Color::rgb(42, 17, 97), Color::Rgb { r: 42, g: 17, b: 97 });
    /// ```
    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::Rgb { r, g, b }
    }
}
