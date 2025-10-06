// Modified from https://github.com/a-kenji/tui-term

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Clear, Widget},
};

pub use vt100::{Cell, Screen};

/// A widget representing a pseudo-terminal screen.
///
/// The `VTermWidget` displays the contents of a pseudo-terminal screen,
/// which is typically populated with text and control sequences from a terminal emulator.
/// It provides a visual representation of the terminal output within a TUI application.
///
/// The contents of the pseudo-terminal screen are represented by a `vt100::Screen` object.
/// The `vt100` library provides functionality for parsing and processing terminal control sequences
/// and handling terminal state, allowing the `PseudoTerminal` widget to accurately render the
/// terminal output.
///
/// # Examples
///
/// ```rust
/// use ratatui::{
///     style::{Color, Modifier, Style},
///     widgets::{Block, Borders},
/// };
/// use tui_term::widget::PseudoTerminal;
/// use vt100::Parser;
///
/// let mut parser = vt100::Parser::new(24, 80, 0);
/// let pseudo_term = PseudoTerminal::new(parser.screen())
///     .style(
///         Style::default()
///             .fg(Color::White)
///             .bg(Color::Black)
///             .add_modifier(Modifier::BOLD),
///     );
/// ```
pub struct VTermWidget<'a> {
    screen: &'a Screen,
}

impl<'a> VTermWidget<'a> {
    /// Creates a new instance of `PseudoTerminal`.
    ///
    /// # Arguments
    ///
    /// * `screen`: The reference to the `Screen`.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_term::widget::PseudoTerminal;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let pseudo_term = PseudoTerminal::new(parser.screen());
    /// ```
    #[inline]
    #[must_use]
    pub fn new(screen: &'a Screen) -> Self {
        VTermWidget { screen }
    }

    #[inline]
    #[must_use]
    pub const fn screen(&self) -> &Screen {
        self.screen
    }
}

impl Widget for VTermWidget<'_> {
    #[inline]
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let cols = area.width;
        let rows = area.height;
        let col_start = area.x;
        let row_start = area.y;
        let area_cols = area.width + area.x;
        let area_rows = area.height + area.y;
        let screen = self.screen();

        // The [`Screen`] is made out of rows of cells
        for row in 0..rows {
            for col in 0..cols {
                let buf_col = col + col_start;
                let buf_row = row + row_start;

                if buf_row > area_rows || buf_col > area_cols {
                    // Skip writing outside the area
                    continue;
                }

                if let Some(screen_cell) = screen.cell(row, col) {
                    let cell = &mut buf[(buf_col, buf_row)];
                    apply_cell_styles(screen_cell, cell);
                }
            }
        }
    }
}

#[inline]
fn apply_cell_styles(screen_cell: &Cell, buf_cell: &mut ratatui::buffer::Cell) {
    let fg = color_map(screen_cell.fgcolor());
    let bg = color_map(screen_cell.bgcolor());
    if screen_cell.has_contents() {
        buf_cell.set_symbol(&screen_cell.contents());
    }

    let mut style = Style::reset();
    if screen_cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if screen_cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if screen_cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if screen_cell.inverse() {
        style = style.add_modifier(Modifier::REVERSED);
    }
    buf_cell.set_style(style);
    buf_cell.set_fg(fg);
    buf_cell.set_bg(bg);
}

#[inline]
fn color_map(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
