/// Action bar and overlay rendering.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Widget};

use sabacc_core::game::GamePhase;
use sabacc_core::shift_token::ShiftToken;

use crate::app::{AppState, Overlay};

/// Renders the action bar with border.
pub fn render_bar(area: Rect, buf: &mut Buffer, app: &AppState) {
    let block = Block::default()
        .title(" ACTIONS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title_style(
            Style::default()
                .fg(super::card::SAND_COLOR)
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(area);
    block.render(area, buf);

    let game = match &app.game {
        Some(g) => g,
        None => return,
    };

    match &game.phase {
        GamePhase::TurnAction if app.is_human_turn() => {
            render_action_bar(inner, buf, app);
        }
        GamePhase::TurnAction => {
            let line = Line::from(vec![Span::styled(
                "Waiting for bots...",
                Style::default().fg(Color::DarkGray),
            )]);
            buf.set_line(inner.x, inner.y, &line, inner.width);
        }
        GamePhase::Reveal { .. } | GamePhase::RoundEnd => {
            let line = Line::from(vec![Span::styled(
                "[Enter] Continue",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]);
            buf.set_line(inner.x, inner.y, &line, inner.width);
        }
        GamePhase::ImpostorReveal { pending, .. } => {
            let msg = if pending.contains(&0u8) {
                "Impostor! Choose a die value..."
            } else {
                "Resolving Impostors..."
            };
            let line = Line::from(vec![Span::styled(msg, Style::default().fg(Color::Magenta))]);
            buf.set_line(inner.x, inner.y, &line, inner.width);
        }
        GamePhase::PrimeSabaccChoice { player_id, .. } => {
            let msg = if *player_id == 0 {
                "Prime Sabacc! Choose a value..."
            } else {
                "Resolving Prime Sabacc..."
            };
            let line = Line::from(vec![Span::styled(msg, Style::default().fg(Color::Magenta))]);
            buf.set_line(inner.x, inner.y, &line, inner.width);
        }
        GamePhase::ChoosingDiscard { .. } => {
            let line = Line::from(vec![Span::styled(
                "Choose which card to keep...",
                Style::default().fg(Color::Cyan),
            )]);
            buf.set_line(inner.x, inner.y, &line, inner.width);
        }
        GamePhase::GameOver { .. } => {
            let line = Line::from(vec![
                Span::styled(
                    "[Enter] New game  ",
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled("[q] Quit", Style::default().fg(Color::DarkGray)),
            ]);
            buf.set_line(inner.x, inner.y, &line, inner.width);
        }
        _ => {}
    }
}

fn render_action_bar(area: Rect, buf: &mut Buffer, app: &AppState) {
    let has_tokens = app.game.as_ref().is_some_and(|g| {
        g.config.enable_shift_tokens
            && !g.token_played_this_turn
            && g.players
                .first()
                .is_some_and(|p| !p.shift_tokens.is_empty())
    });

    let actions: Vec<(&str, &str)> = if has_tokens {
        vec![("Draw", "d"), ("Stand", ""), ("Token", "s")]
    } else {
        vec![("Draw", "d"), ("Stand", "")]
    };

    let spans: Vec<Span> = actions
        .iter()
        .enumerate()
        .flat_map(|(i, (name, key))| {
            let selected = i == app.tui.selected_action;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let mut spans = vec![Span::styled(format!(" {name} "), style)];
            if !key.is_empty() {
                spans.push(Span::styled(
                    format!("({key})"),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            spans.push(Span::raw("  "));
            spans
        })
        .collect();

    let line = Line::from(spans);
    buf.set_line(area.x, area.y, &line, area.width);

    // Hint line
    if area.height > 1 {
        let hint = Line::from(Span::styled(
            "Tab: navigate · Enter: confirm · ?: help",
            Style::default().fg(Color::DarkGray),
        ));
        buf.set_line(area.x, area.y + 1, &hint, area.width);
    }
}

/// Renders overlays on top of the full terminal area.
pub fn render_overlay(area: Rect, buf: &mut Buffer, app: &AppState) {
    let overlay = match &app.tui.overlay {
        Some(o) => o,
        None => return,
    };
    match overlay {
        Overlay::QuitConfirm => {
            let popup = centered_popup(area, 30, 5);
            Clear.render(popup, buf);
            let block = Block::default()
                .title(" Quit? ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));
            let inner = block.inner(popup);
            block.render(popup, buf);
            let line = Line::from(vec![
                Span::styled("[y] Yes  ", Style::default().fg(Color::Red)),
                Span::styled("[n] No", Style::default().fg(Color::Green)),
            ]);
            buf.set_line(inner.x, inner.y + 1, &line, inner.width);
        }
        Overlay::SourcePicker => {
            let popup = centered_popup(area, 36, 8);
            Clear.render(popup, buf);
            let block = Block::default()
                .title(" Choose source ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(popup);
            block.render(popup, buf);

            let sources = [
                "1. Deck Sand",
                "2. Deck Blood",
                "3. Discard Sand",
                "4. Discard Blood",
            ];
            let colors = [
                super::card::SAND_COLOR,
                super::card::BLOOD_COLOR,
                super::card::SAND_COLOR,
                super::card::BLOOD_COLOR,
            ];

            for (i, (source, color)) in sources.iter().zip(colors.iter()).enumerate() {
                let selected = i == app.tui.selected_source;
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(*color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(*color)
                };
                if (i as u16) < inner.height {
                    let prefix = if selected { "▶ " } else { "  " };
                    buf.set_string(inner.x, inner.y + i as u16, prefix, style);
                    buf.set_string(inner.x + 2, inner.y + i as u16, source, style);
                }
            }
        }
        Overlay::DiscardChoice { drawn, current } => {
            let popup = centered_popup(area, 46, 11);
            Clear.render(popup, buf);
            let block = Block::default()
                .title(" Keep or Discard? ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(popup);
            block.render(popup, buf);

            let (drawn_str, drawn_color) =
                super::card::CardWidget::from_card(drawn, false).inline_string();
            let (current_str, current_color) =
                super::card::CardWidget::from_card(current, false).inline_string();

            // Show both cards
            let header = Line::from(vec![
                Span::styled("Drawn: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&drawn_str, Style::default().fg(drawn_color)),
                Span::raw("   "),
                Span::styled("In hand: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&current_str, Style::default().fg(current_color)),
            ]);
            buf.set_line(inner.x, inner.y, &header, inner.width);

            // Separator
            if inner.height > 1 {
                buf.set_string(
                    inner.x,
                    inner.y + 1,
                    "──────────────────────────────────────────",
                    Style::default().fg(Color::DarkGray),
                );
            }

            // Options with card previews
            let options = [
                format!("Keep {}  (discard {})", drawn_str, current_str),
                format!("Keep {}  (discard {})", current_str, drawn_str),
            ];
            for (i, opt) in options.iter().enumerate() {
                let selected = i == app.tui.selected_discard;
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if selected { "▶ " } else { "  " };
                if (i as u16 + 3) < inner.height {
                    buf.set_string(inner.x, inner.y + 3 + i as u16, prefix, style);
                    buf.set_string(inner.x + 2, inner.y + 3 + i as u16, opt, style);
                }
            }

            // Hint
            if inner.height > 6 {
                buf.set_string(
                    inner.x,
                    inner.y + inner.height - 1,
                    "Tab: switch  Enter: confirm",
                    Style::default().fg(Color::DarkGray),
                );
            }
        }
        Overlay::TokenPicker => {
            let tokens: Vec<(String, String)> = app
                .game
                .as_ref()
                .and_then(|g| g.players.first())
                .map_or(Vec::new(), |p| {
                    p.shift_tokens
                        .iter()
                        .map(|t| (token_name(t), token_description(t)))
                        .collect()
                });
            let h = (tokens.len() as u16 + 4).min(area.height);
            let popup = centered_popup(area, 56, h);
            Clear.render(popup, buf);
            let block = Block::default()
                .title(" ShiftTokens ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta));
            let inner = block.inner(popup);
            block.render(popup, buf);

            for (i, (name, desc)) in tokens.iter().enumerate() {
                if (i as u16) >= inner.height {
                    break;
                }
                let selected = i == app.tui.selected_token;
                let name_style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan)
                };
                let desc_style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Magenta)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let prefix = if selected { "▶ " } else { "  " };
                buf.set_string(inner.x, inner.y + i as u16, prefix, name_style);
                buf.set_string(inner.x + 2, inner.y + i as u16, name, name_style);
                let name_len = name.len() as u16 + 3;
                if name_len < inner.width {
                    buf.set_string(
                        inner.x + name_len,
                        inner.y + i as u16,
                        format!(" — {desc}"),
                        desc_style,
                    );
                }
            }
        }
        Overlay::TargetPicker { token } => {
            let targets: Vec<(String, u8)> = app.game.as_ref().map_or(Vec::new(), |g| {
                g.players
                    .iter()
                    .filter(|p| p.id != 0 && !p.is_eliminated)
                    .map(|p| (p.name.clone(), p.id))
                    .collect()
            });
            let h = (targets.len() as u16 + 4).min(area.height);
            let popup = centered_popup(area, 36, h);
            Clear.render(popup, buf);
            let block = Block::default()
                .title(format!(" Target — {:?} ", token))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));
            let inner = block.inner(popup);
            block.render(popup, buf);

            for (i, (name, _)) in targets.iter().enumerate() {
                if (i as u16) >= inner.height {
                    break;
                }
                let selected = i == app.tui.selected_target;
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if selected { "▶ " } else { "  " };
                buf.set_string(inner.x, inner.y + i as u16, prefix, style);
                buf.set_string(inner.x + 2, inner.y + i as u16, name, style);
            }
        }
        Overlay::ImpostorChoice { die1, die2, .. } | Overlay::PrimeSabaccChoice { die1, die2 } => {
            let title = match overlay {
                Overlay::ImpostorChoice { .. } => " Impostor — Choose a die ",
                _ => " Prime Sabacc — Choose a die ",
            };
            let popup = centered_popup(area, 34, 7);
            Clear.render(popup, buf);
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta));
            let inner = block.inner(popup);
            block.render(popup, buf);

            let d1 = *die1;
            let d2 = *die2;
            let options = [format!("Die 1: {d1}"), format!("Die 2: {d2}")];
            for (i, opt) in options.iter().enumerate() {
                let selected = i == app.tui.selected_die;
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if selected { "▶ " } else { "  " };
                if (i as u16 + 1) < inner.height {
                    buf.set_string(inner.x, inner.y + 1 + i as u16, prefix, style);
                    buf.set_string(inner.x + 2, inner.y + 1 + i as u16, opt, style);
                }
            }
        }
        Overlay::GameOver {
            winner_name,
            is_human,
        } => {
            let popup = centered_popup(area, 40, 9);
            Clear.render(popup, buf);
            let color = if *is_human { Color::Yellow } else { Color::Red };
            let block = Block::default()
                .title(" Game Over ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color));
            let inner = block.inner(popup);
            block.render(popup, buf);

            let msg = if *is_human {
                "Victory! You win the pot!".to_string()
            } else {
                format!("{winner_name} wins the game.")
            };

            let style = Style::default().fg(color).add_modifier(Modifier::BOLD);
            buf.set_string(inner.x + 1, inner.y + 1, &msg, style);

            let hint = "[Enter] New game  [q] Quit";
            buf.set_string(
                inner.x + 1,
                inner.y + 3,
                hint,
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

/// Creates a centered rectangle within `area`.
fn centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

/// Returns a short human-readable name for a shift token.
fn token_name(token: &ShiftToken) -> String {
    match token {
        ShiftToken::FreeDraw => "FreeDraw".into(),
        ShiftToken::Refund => "Refund".into(),
        ShiftToken::ExtraRefund => "ExtraRefund".into(),
        ShiftToken::GeneralTariff => "GeneralTariff".into(),
        ShiftToken::TargetTariff(_) => "TargetTariff".into(),
        ShiftToken::Embargo => "Embargo".into(),
        ShiftToken::Markdown => "Markdown".into(),
        ShiftToken::Immunity => "Immunity".into(),
        ShiftToken::GeneralAudit => "GeneralAudit".into(),
        ShiftToken::TargetAudit(_) => "TargetAudit".into(),
        ShiftToken::MajorFraud => "MajorFraud".into(),
        ShiftToken::Embezzlement => "Embezzlement".into(),
        ShiftToken::CookTheBooks => "CookTheBooks".into(),
        ShiftToken::Exhaustion(_) => "Exhaustion".into(),
        ShiftToken::DirectTransaction(_) => "DirectTransaction".into(),
        ShiftToken::PrimeSabacc => "PrimeSabacc".into(),
    }
}

/// Returns a short description for a shift token.
fn token_description(token: &ShiftToken) -> String {
    match token {
        ShiftToken::FreeDraw => "Draw without paying 1 chip".into(),
        ShiftToken::Refund => "Recover 2 invested chips".into(),
        ShiftToken::ExtraRefund => "Recover 3 invested chips".into(),
        ShiftToken::GeneralTariff => "All others pay 1 chip".into(),
        ShiftToken::TargetTariff(_) => "Targeted player pays 2 chips".into(),
        ShiftToken::Embargo => "Next player must Stand".into(),
        ShiftToken::Markdown => "Sylop = 0 (no match)".into(),
        ShiftToken::Immunity => "Immune to opponent tokens".into(),
        ShiftToken::GeneralAudit => "Standing players pay 2 chips".into(),
        ShiftToken::TargetAudit(_) => "Targeted standing pays 3 chips".into(),
        ShiftToken::MajorFraud => "Impostor locked at 6".into(),
        ShiftToken::Embezzlement => "Take 1 chip from each opponent".into(),
        ShiftToken::CookTheBooks => "Reverse Sabacc ranking".into(),
        ShiftToken::Exhaustion(_) => "Target redraws a new hand".into(),
        ShiftToken::DirectTransaction(_) => "Swap hand with target".into(),
        ShiftToken::PrimeSabacc => "Dice → value = best Sabacc".into(),
    }
}
