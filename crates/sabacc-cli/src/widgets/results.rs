/// Round results and game over overlay rendering.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Widget};

use crate::app::Overlay;

const WINNER_COLOR: Color = Color::Rgb(232, 192, 80);
const ELIMINATED_COLOR: Color = Color::Rgb(180, 60, 60);
const LOSER_COLOR: Color = Color::Gray;

/// Render the Round Results overlay.
pub fn render_round_results(area: Rect, buf: &mut Buffer, overlay: &Overlay) {
    let (results, scroll_offset, revealed_count) = match overlay {
        Overlay::RoundResults {
            results,
            scroll_offset,
            revealed_count,
            ..
        } => (results, *scroll_offset, *revealed_count),
        _ => return,
    };

    let content_height = results.len() as u16 * 3 + 2; // 3 lines per player + padding
    let popup_w = (area.width.saturating_sub(4)).min(92);
    let popup_h = (area.height.saturating_sub(2)).min(content_height + 4); // +4 for borders+title+footer
    let popup = centered_popup(area, popup_w, popup_h);
    Clear.render(popup, buf);

    let block = Block::default()
        .title(" ROUND RESULTS ")
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(WINNER_COLOR));
    let inner = block.inner(popup);
    block.render(popup, buf);

    if inner.width < 4 || inner.height < 2 {
        return;
    }

    // Split into left and right columns
    let mid = inner.width / 2;
    let left = Rect::new(inner.x, inner.y, mid.saturating_sub(1), inner.height);
    let right = Rect::new(inner.x + mid + 1, inner.y, inner.width.saturating_sub(mid + 1), inner.height);

    // Draw vertical separator
    for y in inner.y..inner.y + inner.height {
        buf.set_string(inner.x + mid, y, "│", Style::default().fg(Color::DarkGray));
    }

    // Column headers
    buf.set_string(
        left.x,
        left.y,
        "HANDS REVEALED",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    buf.set_string(
        right.x,
        right.y,
        "CHIP MOVEMENTS",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    // Render each player (up to revealed_count)
    let visible_start = scroll_offset;
    let available_lines = left.height.saturating_sub(1) as usize; // -1 for header
    let lines_per_player = 3usize;

    for (i, result) in results.iter().enumerate() {
        if i >= revealed_count {
            break;
        }

        let line_offset = i * lines_per_player;
        if line_offset < visible_start {
            continue;
        }
        let y_pos = (line_offset - visible_start) as u16 + 1; // +1 for header
        if y_pos + 2 > left.height {
            break;
        }

        let base_color = if result.is_winner {
            WINNER_COLOR
        } else if result.is_eliminated {
            ELIMINATED_COLOR
        } else {
            LOSER_COLOR
        };

        let marker = if result.is_winner {
            "★"
        } else if result.is_eliminated {
            "✗"
        } else {
            "·"
        };

        // LEFT: Hand info
        let name_line = format!("{marker} {} — {}", result.player_name, result.rank_str);
        buf.set_string(
            left.x,
            left.y + y_pos,
            truncate(&name_line, left.width as usize),
            Style::default()
                .fg(base_color)
                .add_modifier(if result.is_winner {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        );

        // Card inline display
        let (sand_str, sand_color) =
            super::card::CardWidget::from_card(&result.hand.0, false).inline_string();
        let (blood_str, blood_color) =
            super::card::CardWidget::from_card(&result.hand.1, false).inline_string();

        if y_pos + 1 < left.height {
            let card_y = left.y + y_pos + 1;
            buf.set_string(left.x + 2, card_y, &sand_str, Style::default().fg(sand_color));
            let blood_x = left.x + 2 + sand_str.len() as u16 + 1;
            if blood_x < left.x + left.width {
                buf.set_string(blood_x, card_y, &blood_str, Style::default().fg(blood_color));
            }
        }

        // RIGHT: Chip movements
        let chips_before_dots = chip_dots(result.chips_before);
        let chips_after_dots = chip_dots(result.chips_after);
        let delta = if result.is_winner {
            format!("+{}", result.invested)
        } else if result.penalty > 0 {
            format!("-{}", result.invested + result.penalty)
        } else {
            "0".into()
        };

        let delta_color = if result.is_winner {
            Color::Green
        } else if result.penalty > 0 {
            Color::Red
        } else {
            Color::DarkGray
        };

        // Line 1: dots → dots  delta
        let chip_line = format!("{chips_before_dots} → {chips_after_dots}");
        buf.set_string(
            right.x,
            right.y + y_pos,
            truncate(&chip_line, right.width.saturating_sub(5) as usize),
            Style::default().fg(base_color),
        );
        let delta_x = right.x + right.width.saturating_sub(delta.len() as u16 + 1);
        buf.set_string(
            delta_x,
            right.y + y_pos,
            &delta,
            Style::default().fg(delta_color).add_modifier(Modifier::BOLD),
        );

        // Line 2: reason + ELIMINATED if applicable
        if y_pos + 1 < right.height {
            let reason_y = right.y + y_pos + 1;
            let reason = if result.is_eliminated {
                format!("{}  ELIMINATED", result.penalty_reason)
            } else {
                result.penalty_reason.clone()
            };
            let reason_color = if result.is_eliminated {
                ELIMINATED_COLOR
            } else {
                Color::DarkGray
            };
            buf.set_string(
                right.x,
                reason_y,
                truncate(&reason, right.width as usize),
                Style::default().fg(reason_color),
            );
        }
    }

    // Footer: scroll indicators + hints
    let footer_y = popup.y + popup.height - 1;
    let hint = if revealed_count < results.len() {
        "[Space] Skip  [↑↓] Scroll"
    } else {
        "[Enter] Continue  [↑↓] Scroll"
    };
    // Center the hint in the border line
    let hint_x = popup.x + (popup.width.saturating_sub(hint.len() as u16 + 2)) / 2;
    buf.set_string(
        hint_x,
        footer_y,
        format!(" {hint} "),
        Style::default().fg(Color::DarkGray),
    );

    // Scroll indicators
    if scroll_offset > 0 {
        buf.set_string(
            popup.x + popup.width - 3,
            popup.y + 1,
            "▲",
            Style::default().fg(Color::Yellow),
        );
    }
    let max_visible = available_lines / lines_per_player;
    if revealed_count > max_visible + scroll_offset / lines_per_player {
        buf.set_string(
            popup.x + popup.width - 3,
            popup.y + popup.height - 2,
            "▼",
            Style::default().fg(Color::Yellow),
        );
    }
}

/// Render the Game Over overlay.
pub fn render_game_over(area: Rect, buf: &mut Buffer, overlay: &Overlay) {
    let (standings, stats) = match overlay {
        Overlay::GameOverScreen { standings, stats } => (standings, stats),
        _ => return,
    };

    let content_h = standings.len() as u16 + 8; // title + stats + footer
    let popup_w = (area.width.saturating_sub(4)).min(72);
    let popup_h = (area.height.saturating_sub(2)).min(content_h + 4);
    let popup = centered_popup(area, popup_w, popup_h);
    Clear.render(popup, buf);

    let block = Block::default()
        .title(" ★ ★ ★  G A M E   O V E R  ★ ★ ★ ")
        .title_style(
            Style::default()
                .fg(WINNER_COLOR)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(WINNER_COLOR));
    let inner = block.inner(popup);
    block.render(popup, buf);

    if inner.width < 4 || inner.height < 2 {
        return;
    }

    let mut y = inner.y;

    // Section: FINAL STANDINGS
    buf.set_string(
        inner.x,
        y,
        "FINAL STANDINGS",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    y += 1;

    // Separator
    let sep: String = "─".repeat(inner.width as usize);
    buf.set_string(inner.x, y, &sep, Style::default().fg(Color::DarkGray));
    y += 1;

    // Standings table
    for entry in standings {
        if y >= inner.y + inner.height.saturating_sub(4) {
            break;
        }

        let (marker, color) = if entry.rank == 1 {
            ("★", WINNER_COLOR)
        } else {
            (" ", Color::Gray)
        };

        let rank_str = match entry.rank {
            1 => "1st".to_string(),
            2 => "2nd".to_string(),
            3 => "3rd".to_string(),
            n => format!("{n}th"),
        };

        let name_style = if entry.is_human && entry.rank == 1 {
            Style::default()
                .fg(WINNER_COLOR)
                .add_modifier(Modifier::BOLD)
        } else if entry.rank == 1 {
            Style::default().fg(WINNER_COLOR)
        } else {
            Style::default().fg(color)
        };

        let line_text = format!(
            "{marker} {rank_str}  {:<16} {}",
            entry.player_name, entry.status
        );
        buf.set_string(
            inner.x,
            y,
            truncate(&line_text, inner.width as usize),
            name_style,
        );
        y += 1;
    }

    // Stats section
    y += 1;
    if y < inner.y + inner.height.saturating_sub(2) {
        buf.set_string(
            inner.x,
            y,
            &sep,
            Style::default().fg(Color::DarkGray),
        );
        y += 1;
    }

    if y < inner.y + inner.height.saturating_sub(1) {
        let stats_line = format!(
            "Rounds: {}  |  Pot: {} credits  |  Winner: {}",
            stats.rounds_played, stats.credits_in_pot, stats.winner_name
        );
        buf.set_string(
            inner.x,
            y,
            truncate(&stats_line, inner.width as usize),
            Style::default().fg(Color::DarkGray),
        );
    }

    // Footer
    let footer_y = popup.y + popup.height - 1;
    let hint = " [Enter] New game   [q] Quit ";
    let hint_x = popup.x + (popup.width.saturating_sub(hint.len() as u16)) / 2;
    buf.set_string(hint_x, footer_y, hint, Style::default().fg(Color::DarkGray));
}

/// Create a centered rectangle within `area`.
fn centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

/// Render chip count as ● (filled) dots, max 8.
fn chip_dots(count: u8) -> String {
    let filled = count.min(8) as usize;
    let empty = 8usize.saturating_sub(filled);
    format!("{}{}", "●".repeat(filled), "○".repeat(empty))
}

/// Truncate a string to `max_chars` character width.
fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}
