/// Header bar showing game title, round, turn, and phase.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use sabacc_core::game::GamePhase;

use crate::app::AppState;

/// Renders the header bar.
pub fn render(area: Rect, buf: &mut Buffer, app: &AppState) {
    let title_style = Style::default()
        .fg(Color::Rgb(232, 192, 80))
        .add_modifier(Modifier::BOLD);

    let (round_str, turn_str, phase_str) = match &app.game {
        Some(g) => (
            format!("Round {}", g.round),
            format!("Turn {}/3", g.turn),
            phase_label(&g.phase),
        ),
        None => ("—".into(), "—".into(), "Setup".into()),
    };

    let info_style = Style::default().fg(Color::White);
    let phase_style = Style::default().fg(Color::Cyan);

    let line = Line::from(vec![
        Span::styled(" KESSEL SABACC ", title_style),
        Span::raw("  "),
        Span::styled(&round_str, info_style),
        Span::raw(" · "),
        Span::styled(&turn_str, info_style),
        Span::raw("  │  "),
        Span::styled(&phase_str, phase_style),
    ]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let para = Paragraph::new(line).block(block);
    para.render(area, buf);
}

fn phase_label(phase: &GamePhase) -> String {
    match phase {
        GamePhase::Setup => "Setup".into(),
        GamePhase::TurnAction => "Action".into(),
        GamePhase::ChoosingDiscard { .. } => "Discard Choice".into(),
        GamePhase::ImpostorReveal { .. } => "Impostor Reveal".into(),
        GamePhase::Reveal { .. } => "Reveal".into(),
        GamePhase::PrimeSabaccChoice { .. } => "Prime Sabacc".into(),
        GamePhase::RoundEnd => "Round End".into(),
        GamePhase::GameOver { .. } => "Game Over".into(),
    }
}
