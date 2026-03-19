/// Table center — 4 cards horizontal: Discard Sand | Deck Sand | Deck Blood | Discard Blood.
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use super::card::{CardWidget, BLOOD_COLOR, SAND_COLOR};
use crate::animation::Animation;
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

    let picking = app.tui.source_picking;
    let sel = app.tui.selected_source;

    // Discard Sand (source index 0)
    if let Some(top) = game.sand_deck.peek_discard() {
        let mut cw = CardWidget::from_card(top, false);
        cw.selected = picking && sel == 0;
        cw.render(card_cols[0], buf);
    } else {
        render_empty_slot(card_cols[0], buf, SAND_COLOR, picking && sel == 0);
    }

    // Deck Sand (source index 1)
    let mut deck_sand = CardWidget::face_down();
    deck_sand.selected = picking && sel == 1;
    deck_sand.render(card_cols[2], buf);

    // Deck Blood (source index 2)
    let mut deck_blood = CardWidget::face_down();
    deck_blood.selected = picking && sel == 2;
    deck_blood.render(card_cols[4], buf);

    // Discard Blood (source index 3)
    if let Some(top) = game.blood_deck.peek_discard() {
        let mut cw = CardWidget::from_card(top, false);
        cw.selected = picking && sel == 3;
        cw.render(card_cols[6], buf);
    } else {
        render_empty_slot(card_cols[6], buf, BLOOD_COLOR, picking && sel == 3);
    }

    // Labels under cards
    let label_y = inner.y + v_offset + 5;
    if label_y < inner.bottom() {
        let highlight_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let dis_sand_style = if picking && sel == 0 {
            highlight_style
        } else {
            Style::default().fg(SAND_COLOR)
        };
        buf.set_string(card_cols[0].x, label_y, "Dis Sand", dis_sand_style);

        let deck_sand_style = if picking && sel == 1 {
            highlight_style
        } else {
            Style::default().fg(Color::DarkGray)
        };
        buf.set_string(
            card_cols[2].x,
            label_y,
            format!("Deck({})", game.sand_deck.draw_pile.len()),
            deck_sand_style,
        );

        let deck_blood_style = if picking && sel == 2 {
            highlight_style
        } else {
            Style::default().fg(Color::DarkGray)
        };
        buf.set_string(
            card_cols[4].x,
            label_y,
            format!("Deck({})", game.blood_deck.draw_pile.len()),
            deck_blood_style,
        );

        let dis_blood_style = if picking && sel == 3 {
            highlight_style
        } else {
            Style::default().fg(BLOOD_COLOR)
        };
        buf.set_string(card_cols[6].x, label_y, "Dis Blood", dis_blood_style);
    }

    // PhaseAnnounce overlay on top of table content
    render_phase_announce(inner, buf, app);
}

/// Renders a PhaseAnnounce text centered on top of the table area.
fn render_phase_announce(area: Rect, buf: &mut Buffer, app: &AppState) {
    if let Some(ref active) = app.animations.current {
        if let Animation::PhaseAnnounce { ref text, .. } = active.animation {
            let progress = active.progress();
            let fg = if progress <= 0.5 {
                Color::White
            } else {
                Color::DarkGray
            };
            let display = text.to_uppercase();
            let text_width = display.chars().count() as u16;
            let x = area.x + area.width.saturating_sub(text_width) / 2;
            let y = area.y + area.height / 2;
            buf.set_string(
                x,
                y,
                &display,
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            );
        }
    }
}

fn render_empty_slot(area: Rect, buf: &mut Buffer, color: Color, selected: bool) {
    if area.height < 5 || area.width < 8 {
        return;
    }
    let border_style = if selected {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let style = Style::default().fg(color);
    buf.set_string(area.x, area.y, "┌──────┐", border_style);
    buf.set_string(area.x, area.y + 1, "│      │", border_style);
    buf.set_string(area.x, area.y + 2, "│ empty│", style);
    buf.set_string(area.x, area.y + 3, "│      │", border_style);
    buf.set_string(area.x, area.y + 4, "└──────┘", border_style);
}
