/// Log panel — scrollable action history with word-wrap and PageUp/PageDown.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::app::AppState;

/// Renders the log panel with scroll support and word-wrapped entries.
pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let block = Block::default()
        .title(" LOG ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height == 0 || inner.width < 4 {
        return;
    }

    let max_w = inner.width as usize;

    // Pre-compute all wrapped lines with their styles
    let mut all_lines: Vec<(String, Style)> = Vec::new();
    for entry in &app.log {
        let style = if entry.is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let wrapped = wrap_text(&entry.text, max_w);
        for (i, line) in wrapped.iter().enumerate() {
            if i == 0 {
                // First line: prefix with ›
                all_lines.push((format!("› {line}"), style));
            } else {
                // Continuation: indent
                all_lines.push((format!("  {line}"), style));
            }
        }
    }

    let total = all_lines.len();
    let offset = app.tui.log_scroll_offset;
    let show_indicator = !app.tui.log_auto_scroll && offset > 0;
    let visible = if show_indicator {
        (inner.height as usize).saturating_sub(1)
    } else {
        inner.height as usize
    };

    let start = total.saturating_sub(visible + offset);
    let end = (start + visible).min(total);

    for (i, (text, style)) in all_lines[start..end].iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        // Clamp to inner width
        let display: String = text.chars().take(max_w).collect();
        buf.set_string(inner.x, inner.y + i as u16, &display, *style);
    }

    // Scroll indicator
    if show_indicator && inner.height > 1 {
        buf.set_string(
            inner.x,
            inner.bottom().saturating_sub(1),
            "▼ new",
            Style::default().fg(Color::Yellow),
        );
    }
}

/// Wraps text into lines that fit within `max_width`, accounting for
/// the 2-char prefix ("› " or "  ") that will be added later.
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let usable = max_width.saturating_sub(2); // reserve 2 for prefix
    if usable == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        let char_count = remaining.chars().count();
        if char_count <= usable {
            lines.push(remaining.to_string());
            break;
        }

        // Find a break point: last space within usable width
        let boundary: String = remaining.chars().take(usable).collect();
        let break_at = boundary
            .rfind(' ')
            .map(|pos| pos + 1) // include the space
            .unwrap_or(usable); // no space found, hard break

        let (chunk, rest) = split_at_char(remaining, break_at);
        lines.push(chunk.trim_end().to_string());
        remaining = rest.trim_start();
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Splits a string at a character index (not byte index).
fn split_at_char(s: &str, char_idx: usize) -> (&str, &str) {
    let byte_idx = s
        .char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    s.split_at(byte_idx)
}
