pub mod actions;
pub mod card;
pub mod hand;
pub mod header;
pub mod log;
pub mod players;
pub mod results;
pub mod starfield;
pub mod table;

use ratatui::layout::Rect;

/// Creates a centered rectangle within `area`.
pub fn centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
