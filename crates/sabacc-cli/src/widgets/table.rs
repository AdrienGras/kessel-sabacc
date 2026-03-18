/// Table center — 4 cards horizontal: Discard Sand | Deck Sand | Deck Blood | Discard Blood.
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use super::card::{CardWidget, BLOOD_COLOR, SAND_COLOR};
use crate::app::AppState;

/// Renders the table with 4 cards in a horizontal line, centered, with border.
pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let game = match &app.game {
        Some(g) => g,
        None => return,
    };

    let block = Block::default()
        .title(" GAME TABLE ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title_style(Style::default().fg(SAND_COLOR).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 6 || inner.width < 40 {
        return;
    }

    // 4 cards: 8+2+8+4+8+2+8 = 40 chars total, 6 lines (5 card + 1 label)
    let cards_total_width: u16 = 40;
    let content_height: u16 = 6;
    // Center horizontally and vertically
    let h_offset = inner.width.saturating_sub(cards_total_width) / 2;
    let v_offset = inner.height.saturating_sub(content_height) / 2;

    let cards_area = Rect::new(
        inner.x + h_offset,
        inner.y + v_offset,
        cards_total_width.min(inner.width),
        5,
    );

    let card_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(8), // Discard Sand
            Constraint::Length(2), // spacer
            Constraint::Length(8), // Deck Sand
            Constraint::Length(4), // spacer (family separation)
            Constraint::Length(8), // Deck Blood
            Constraint::Length(2), // spacer
            Constraint::Length(8), // Discard Blood
        ])
        .split(cards_area);

    // Discard Sand
    if let Some(top) = game.sand_deck.peek_discard() {
        CardWidget::from_card(top, false).render(card_cols[0], buf);
    } else {
        render_empty_slot(card_cols[0], buf, SAND_COLOR);
    }

    // Deck Sand
    CardWidget::face_down().render(card_cols[2], buf);

    // Deck Blood
    CardWidget::face_down().render(card_cols[4], buf);

    // Discard Blood
    if let Some(top) = game.blood_deck.peek_discard() {
        CardWidget::from_card(top, false).render(card_cols[6], buf);
    } else {
        render_empty_slot(card_cols[6], buf, BLOOD_COLOR);
    }

    // Labels under cards
    let label_y = inner.y + v_offset + 5;
    if label_y < inner.bottom() {
        buf.set_string(
            card_cols[0].x,
            label_y,
            "Dis Sand",
            Style::default().fg(SAND_COLOR),
        );
        buf.set_string(
            card_cols[2].x,
            label_y,
            format!("Deck({})", game.sand_deck.draw_pile.len()),
            Style::default().fg(Color::DarkGray),
        );
        buf.set_string(
            card_cols[4].x,
            label_y,
            format!("Deck({})", game.blood_deck.draw_pile.len()),
            Style::default().fg(Color::DarkGray),
        );
        buf.set_string(
            card_cols[6].x,
            label_y,
            "Dis Blood",
            Style::default().fg(BLOOD_COLOR),
        );
    }
}

fn render_empty_slot(area: Rect, buf: &mut Buffer, color: Color) {
    if area.height < 5 || area.width < 8 {
        return;
    }
    let style = Style::default().fg(color);
    let dim = Style::default().fg(Color::DarkGray);
    buf.set_string(area.x, area.y, "┌──────┐", dim);
    buf.set_string(area.x, area.y + 1, "│      │", dim);
    buf.set_string(area.x, area.y + 2, "│ empty│", style);
    buf.set_string(area.x, area.y + 3, "│      │", dim);
    buf.set_string(area.x, area.y + 4, "└──────┘", dim);
}
