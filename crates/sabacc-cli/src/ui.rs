/// Main render dispatch — responsive layout + setup screen.
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{AppState, Screen, SetupState};
use crate::widgets;

/// Top-level render function called from the main loop.
pub fn render(frame: &mut Frame, app: &AppState) {
    match app.screen {
        Screen::Setup => render_setup(frame, app),
        Screen::Playing => render_playing(frame, app),
    }

    // Help overlay on top of everything
    if app.tui.show_help {
        render_help(frame);
    }
}

// ── Setup screen ─────────────────────────────────────────────────────

fn render_setup(frame: &mut Frame, app: &AppState) {
    let area = frame.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(12),   // form
            Constraint::Length(2), // hints
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " KESSEL SABACC ",
            Style::default()
                .fg(Color::Rgb(232, 192, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — Configuration", Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(title, layout[0]);

    // Form
    render_setup_form(frame, layout[1], &app.setup);

    // Hints
    let hints = Paragraph::new(Line::from(vec![Span::styled(
        " Tab/↑↓: naviguer  ◀▶: modifier  Enter: confirmer  Esc: quitter",
        Style::default().fg(Color::DarkGray),
    )]));
    frame.render_widget(hints, layout[2]);
}

fn render_setup_form(frame: &mut Frame, area: Rect, setup: &SetupState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut fields: Vec<(&str, String)> = vec![
        ("Nom du joueur", format!("{}_", setup.player_name)),
        ("Nombre de bots", format!("◀ {} ▶", setup.num_bots)),
        (
            "Buy-in",
            format!(
                "◀ {} crédits ({} jetons) ▶",
                setup.buy_in(),
                setup.starting_chips()
            ),
        ),
        (
            "ShiftTokens",
            if setup.tokens_enabled {
                "◀ ON ▶".into()
            } else {
                "◀ OFF ▶".into()
            },
        ),
    ];
    if setup.tokens_enabled {
        fields.push((
            "Tokens par joueur",
            format!("◀ {} ▶", setup.tokens_per_player),
        ));
    } else {
        fields.push(("Tokens par joueur", "—".into()));
    }
    fields.push(("", "[ LANCER LA PARTIE ]".into()));

    for (i, (label, value)) in fields.iter().enumerate() {
        if i as u16 * 2 >= inner.height {
            break;
        }
        let selected = i == setup.selected_field;

        let y = inner.y + (i as u16) * 2;

        if !label.is_empty() {
            let label_style = Style::default().fg(Color::DarkGray);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(*label, label_style))),
                Rect::new(inner.x + 2, y, inner.width.saturating_sub(2), 1),
            );
        }

        let value_style = if selected {
            if i == SetupState::NUM_FIELDS - 1 {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(232, 192, 80))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::White)
        };

        let prefix = if selected { "▶ " } else { "  " };
        let value_y = if label.is_empty() { y } else { y + 1 };

        if value_y < inner.bottom() {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(value.as_str(), value_style),
                ])),
                Rect::new(inner.x + 2, value_y, inner.width.saturating_sub(2), 1),
            );
        }
    }
}

// ── Playing screen ───────────────────────────────────────────────────

fn render_playing(frame: &mut Frame, app: &AppState) {
    let area = frame.area();

    // Minimum size guard
    if area.width < 120 || area.height < 30 {
        let msg = format!(
            "Terminal trop petit ({}×{})\nMinimum requis : 120×30",
            area.width, area.height
        );
        let para = Paragraph::new(msg)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Red));
        let y = area.height / 2;
        frame.render_widget(para, Rect::new(area.x, y.saturating_sub(1), area.width, 3));
        return;
    }

    // Vertical: header + main
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(20)])
        .split(area);

    // Header
    widgets::header::render(main_layout[0], frame.buffer_mut(), app);

    // Horizontal: 3 columns
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // players
            Constraint::Min(60),    // center (game)
            Constraint::Length(27), // log
        ])
        .split(main_layout[1]);

    // Center vertical: tapis (expands) + actions + hand (fixed bottom)
    // Heights include 2 for borders (top+bottom)
    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(9),     // tapis — takes all remaining space
            Constraint::Length(5),  // actions (2 border + 3 content)
            Constraint::Length(10), // hand + tokens (fixed at bottom)
        ])
        .split(cols[1]);

    // Render widgets
    widgets::players::render(cols[0], frame.buffer_mut(), app);
    widgets::table::render(center[0], frame.buffer_mut(), app);
    widgets::actions::render_bar(center[1], frame.buffer_mut(), app);
    widgets::hand::render(center[2], frame.buffer_mut(), app);
    widgets::log::render(cols[2], frame.buffer_mut(), app);

    // Overlays render on top of the full terminal area
    if app.tui.overlay.is_some() {
        widgets::actions::render_overlay(area, frame.buffer_mut(), app);
    }
}

// ── Help overlay ─────────────────────────────────────────────────────

fn render_help(frame: &mut Frame) {
    let area = frame.area();
    let w = 50u16.min(area.width);
    let h = 16u16.min(area.height);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup = Rect::new(x, y, w, h);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Aide — Kessel Sabacc ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab/←→    Naviguer entre les actions"),
        Line::from("  Enter     Confirmer la sélection"),
        Line::from("  1-4       Sélection directe (source)"),
        Line::from("  s         Jouer un ShiftToken"),
        Line::from("  Esc       Annuler / Fermer"),
        Line::from("  Space     Skip les animations"),
        Line::from("  PgUp/PgDn Scroller le log"),
        Line::from("  ?         Cette aide"),
        Line::from("  q         Quitter"),
        Line::from(""),
        Line::from(Span::styled(
            "But: garder 2 cartes proches en valeur !",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Appuyez sur une touche pour fermer.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(help_text).block(block);
    frame.render_widget(para, popup);
}
