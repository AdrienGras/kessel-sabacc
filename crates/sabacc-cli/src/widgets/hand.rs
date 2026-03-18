/// Player's hand — 3 columns: chips left, cards center, tokens right.
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use super::card::{CardWidget, SAND_COLOR};
use crate::app::AppState;

/// Renders the human player's hand with border, split into 3 columns.
pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let game = match &app.game {
        Some(g) => g,
        None => return,
    };

    let player = match game.players.first() {
        Some(p) => p,
        None => return,
    };

    let block = Block::default()
        .title(" YOUR HAND ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title_style(Style::default().fg(SAND_COLOR).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 3 || inner.width < 40 {
        return;
    }

    // 3 equal columns: chips | cards | tokens
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(inner);

    render_chips(cols[0], buf, player);
    render_cards(cols[1], buf, app);
    render_tokens(cols[2], buf, player);
}

fn render_chips(area: Rect, buf: &mut Buffer, player: &sabacc_core::player::Player) {
    // Vertically center: 2 lines of content (chips + detail)
    let v_offset = area.height.saturating_sub(2) / 2;
    let mut y = area.y + v_offset;

    // Chips visual ●○
    let filled = "●".repeat(player.chips as usize);
    let invested = "○".repeat(player.pot as usize);
    let chips_color = if player.chips == 0 {
        Color::Red
    } else {
        Color::Rgb(200, 200, 100)
    };

    let chips_line = Line::from(vec![
        Span::styled(&filled, Style::default().fg(chips_color)),
        Span::styled(&invested, Style::default().fg(Color::DarkGray)),
    ]);
    buf.set_line(area.x + 1, y, &chips_line, area.width.saturating_sub(1));
    y += 1;

    if y < area.bottom() {
        let detail = format!("{} res. + {} pot", player.chips, player.pot);
        buf.set_string(area.x + 1, y, &detail, Style::default().fg(Color::DarkGray));
    }
}

fn render_cards(area: Rect, buf: &mut Buffer, app: &AppState) {
    let player = match app.game.as_ref().and_then(|g| g.players.first()) {
        Some(p) => p,
        None => return,
    };

    if let Some(hand) = &player.hand {
        // Center cards both vertically and horizontally in the column
        let cards_width: u16 = 18; // 8 + 2 + 8
        let v_offset = area.height.saturating_sub(5) / 2;
        let h_offset = area.width.saturating_sub(cards_width) / 2;
        let card_y = area.y + v_offset;

        let card_area = Rect::new(
            area.x + h_offset,
            card_y,
            cards_width.min(area.width),
            5.min(area.height),
        );
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(2),
                Constraint::Length(8),
            ])
            .split(card_area);

        CardWidget::from_card(&hand.sand, false).render(cols[0], buf);
        CardWidget::from_card(&hand.blood, false).render(cols[2], buf);
    }
}

fn render_tokens(area: Rect, buf: &mut Buffer, player: &sabacc_core::player::Player) {
    let mut y = area.y;

    // Title
    buf.set_string(
        area.x,
        y,
        "SHIFT TOKENS",
        Style::default().fg(SAND_COLOR).add_modifier(Modifier::BOLD),
    );
    y += 1;

    if y >= area.bottom() {
        return;
    }

    if player.shift_tokens.is_empty() {
        buf.set_string(area.x, y, "(none)", Style::default().fg(Color::DarkGray));
        return;
    }

    let available_lines = area.bottom().saturating_sub(y) as usize;
    let token_count = player.shift_tokens.len();

    // If we can fit 2 lines per token (name + desc), do it
    // Otherwise fall back to 1 line per token (name only)
    let use_compact = token_count * 2 > available_lines;

    if use_compact {
        // Compact: 1 line per token, name + short desc on same line
        for token in &player.shift_tokens {
            if y >= area.bottom() {
                break;
            }
            let name = token_name(token);
            let desc = token_description(token);

            let max_w = area.width as usize;
            let text = if name.len() + 3 + desc.len() <= max_w {
                format!("{name} — {desc}")
            } else {
                name
            };

            buf.set_string(area.x, y, &text, Style::default().fg(Color::Cyan));
            y += 1;
        }
    } else {
        // Full: 2 lines per token
        for token in &player.shift_tokens {
            if y >= area.bottom() {
                break;
            }
            let name = token_name(token);
            buf.set_string(area.x, y, &name, Style::default().fg(Color::Cyan));
            y += 1;

            if y >= area.bottom() {
                break;
            }
            let desc = token_description(token);
            buf.set_string(area.x + 2, y, &desc, Style::default().fg(Color::DarkGray));
            y += 1;
        }
    }
}

fn token_name(token: &sabacc_core::shift_token::ShiftToken) -> String {
    use sabacc_core::shift_token::ShiftToken;
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

fn token_description(token: &sabacc_core::shift_token::ShiftToken) -> String {
    use sabacc_core::shift_token::ShiftToken;
    match token {
        ShiftToken::FreeDraw => "Free draw".into(),
        ShiftToken::Refund => "Recover 2 chips".into(),
        ShiftToken::ExtraRefund => "Recover 3 chips".into(),
        ShiftToken::GeneralTariff => "All pay 1 chip".into(),
        ShiftToken::TargetTariff(_) => "Target pays 2 chips".into(),
        ShiftToken::Embargo => "Next must Stand".into(),
        ShiftToken::Markdown => "Sylop = 0".into(),
        ShiftToken::Immunity => "Token immunity".into(),
        ShiftToken::GeneralAudit => "Standing pay 2 chips".into(),
        ShiftToken::TargetAudit(_) => "Target standing pays 3".into(),
        ShiftToken::MajorFraud => "Impostor locked at 6".into(),
        ShiftToken::Embezzlement => "1 chip per opponent".into(),
        ShiftToken::CookTheBooks => "Reverse ranking".into(),
        ShiftToken::Exhaustion(_) => "Target redraws hand".into(),
        ShiftToken::DirectTransaction(_) => "Swap hand w/ target".into(),
        ShiftToken::PrimeSabacc => "Dice → best Sabacc".into(),
    }
}
