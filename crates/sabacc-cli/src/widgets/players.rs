/// Players panel — shows all players with chips and hand status (3-line format).
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::animation::Animation;
use crate::app::AppState;

use super::card::SAND_COLOR;

/// Renders the players panel in 3-line-per-player format.
pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let block = Block::default()
        .title(" PLAYERS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let game = match &app.game {
        Some(g) => g,
        None => return,
    };

    // Pot line at the top of the players panel
    let pot_text = format!("Pot: {}cr", game.credits_in_pot);
    buf.set_string(
        inner.x + 1,
        inner.y,
        &pot_text,
        Style::default()
            .fg(SAND_COLOR)
            .add_modifier(Modifier::BOLD),
    );

    // Adjust inner area: skip the pot line + 1 separator
    let inner = Rect::new(inner.x, inner.y + 2, inner.width, inner.height.saturating_sub(2));

    let available_height = inner.height as usize;
    let player_count = game.players.len();

    // Calculate how many lines each player needs
    // Full: 3 lines + 1 separator = 4 lines (last player: 3 lines)
    let full_lines_needed = player_count * 4 - 1;
    let use_compact_eliminated = full_lines_needed > available_height;

    // If a bot is highlighted by animation, suppress `is_current` on others
    let any_highlighted = app
        .animations
        .current
        .as_ref()
        .is_some_and(|a| matches!(a.animation, Animation::PlayerHighlight { .. }));

    let mut y = inner.y;

    for (i, player) in game.players.iter().enumerate() {
        if y >= inner.bottom() {
            // Truncation indicator on last visible line
            let remaining = player_count - i;
            if remaining > 0 {
                buf.set_string(
                    inner.x,
                    inner.bottom().saturating_sub(1),
                    format!("+{remaining}..."),
                    Style::default().fg(Color::DarkGray),
                );
            }
            break;
        }

        let is_current = i == game.current_player_idx && !any_highlighted;
        let is_highlighted = is_player_highlighted(app, player.id);

        // Compact eliminated players if needed
        if use_compact_eliminated && player.is_eliminated {
            let name_style = Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::CROSSED_OUT);
            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(&player.name, name_style),
            ]);
            buf.set_line(inner.x, y, &line, inner.width);
            y += 1;
            continue;
        }

        // Line 1: indicator + name
        let is_active = (is_current || is_highlighted) && !player.is_eliminated;
        let name_style = if player.is_eliminated {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::CROSSED_OUT)
        } else if is_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let indicator = if is_active { "▶ " } else { "  " };

        let name_line = Line::from(vec![
            Span::raw(indicator),
            Span::styled(&player.name, name_style),
        ]);
        buf.set_line(inner.x, y, &name_line, inner.width);
        y += 1;

        if y >= inner.bottom() {
            break;
        }

        // Line 2: chips ●○
        let filled = "●".repeat(player.chips as usize);
        let invested = "○".repeat(player.pot as usize);
        let chips_color = if player.chips == 0 && !player.is_eliminated {
            Color::Red
        } else {
            Color::Rgb(200, 200, 100)
        };
        let chips_line = Line::from(vec![
            Span::raw("  "),
            Span::styled(&filled, Style::default().fg(chips_color)),
            Span::styled(&invested, Style::default().fg(Color::DarkGray)),
        ]);
        buf.set_line(inner.x, y, &chips_line, inner.width);
        y += 1;

        if y >= inner.bottom() {
            break;
        }

        // Line 3: detail text
        let detail = format!("  {} res. + {} pot", player.chips, player.pot);
        buf.set_string(inner.x, y, &detail, Style::default().fg(Color::DarkGray));
        y += 1;

        // Separator line (except after last player)
        if i < player_count - 1 && y < inner.bottom() {
            y += 1;
        }
    }
}

fn is_player_highlighted(app: &AppState, player_id: sabacc_core::PlayerId) -> bool {
    if let Some(ref active) = app.animations.current {
        if let Animation::PlayerHighlight { player_id: pid, .. } = &active.animation {
            return *pid == player_id;
        }
    }
    false
}
