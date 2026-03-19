/// Main render dispatch — responsive layout + menu screens.
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{AppState, MenuItem, Screen, SetupState};
use crate::widgets;
use crate::widgets::starfield::StarfieldWidget;

/// Amber colour used throughout the UI.
const AMBER: Color = Color::Rgb(232, 192, 80);

/// ASCII art title — thick block letters with shadow.
/// Each pair is (main line, shadow line offset +1 col +1 row).
/// All main lines are 49 display-columns wide.
const TITLE_ART: [&str; 5] = [
    " ███████  █████  ██████   █████   ██████  ██████",
    " ██      ██   ██ ██   ██ ██   ██ ██      ██     ",
    " ███████ ███████ ██████  ███████ ██      ██     ",
    "      ██ ██   ██ ██   ██ ██   ██ ██      ██     ",
    " ███████ ██   ██ ██████  ██   ██  ██████  ██████",
];
/// Shadow layer — same text rendered 1 col right, 1 row down, in dark color.
const TITLE_SHADOW_OFFSET: (u16, u16) = (1, 1);

/// Rules text for the How to Play screen.
const RULES_TEXT: &str = "\
WELCOME TO THE TABLE
────────────────────
In the shadow of Kessel's spice mines, smugglers and
scoundrels gather to play the galaxy's most notorious
card game. Fortunes are won and lost in minutes. Lando
Calrissian once bet — and lost — the Millennium Falcon
at a table just like this one.

Your goal is simple: outlast every other player at the
table. When your chips are gone, so are you. The last
one standing takes the pot.

THE DECK
────────
The Sabacc deck is split into two families — Sand and
Blood — each marked with its own colour and symbol.

Each family contains:
  · Number cards valued 1 through 6 (3 copies each)
  · 2 Sylop cards — wildcards that copy the value of
    your other card (powerful, but rare)
  · 2 Impostor cards — their value is unknown until
    the reveal, when you roll dice to determine it

That makes 44 cards in total: 22 Sand, 22 Blood. Two
separate draw piles and two discard piles sit on the
table at all times.

YOUR HAND
─────────
You always hold exactly two cards: one Sand, one Blood.
Think of them as two halves of a wager — you want them
to match as closely as possible.

  Example: Sand 3 + Blood 3 = Sabacc! (difference 0)
  Example: Sand 5 + Blood 2 = difference of 3 (bad)

HAND RANKINGS
─────────────
From best to worst — this is what separates the legends
from the busted:

  1. PURE SABACC
     Both cards are Sylops (Sand Sylop + Blood Sylop).
     The rarest and most unbeatable hand in the game.
     If you see it, savour it — most pilots never will.

  2. PRIME SABACC
     Only possible through the PrimeSabacc shift token.
     You roll two dice and pick a value — if your cards
     match it, you hold a hand that beats everything
     except Pure Sabacc.

  3. SYLOP SABACC
     One Sylop + one number card. The Sylop copies the
     number, giving you a perfect pair (difference 0).
     Example: Sand Sylop + Blood 4 = effectively 4 + 4.

  4. SABACC
     Two number cards with the same value (difference 0).
     Ties between Sabaccs are broken by lowest value:
     a pair of 1s beats a pair of 6s. The humble hand
     of a patient smuggler.

  5. NON-SABACC
     Any hand where the two values differ. The smaller
     the difference, the better. Sand 4 + Blood 5 (diff
     1) beats Sand 1 + Blood 6 (diff 5) every time.

HOW A ROUND PLAYS OUT
─────────────────────
Each round consists of 3 turns. Every player acts once
per turn, in clockwise order from the dealer.

On your turn, you have two choices:

  DRAW — Pick one card from the four sources on the
  table: Sand Deck (face-down), Sand Discard (face-up),
  Blood Deck (face-down), or Blood Discard (face-up).

  After drawing, you must discard one card of the same
  family — either the card you just drew, or the one
  you already held. You always keep exactly 1 Sand and
  1 Blood. Drawing costs 1 chip.

  STAND — Do nothing. It's free. Sometimes the smartest
  play is to hold your nerve and keep what you have.
  But beware: some Shift Tokens punish those who Stand.

Before choosing Draw or Stand, you may optionally play
one Shift Token (see below).

IMPOSTORS
─────────
Impostors are wild cards with a twist. You won't know
their value until the reveal at the end of the round.

When hands are revealed, any player holding an Impostor
rolls two Sabacc dice (each showing 1-6) and picks one
of the two values. If you're holding two Impostors, you
choose for each one separately.

  Example: You hold Sand Impostor + Blood 2. At reveal
  you roll a 3 and a 5. You pick 3 — your hand becomes
  effectively Sand 3 + Blood 2, difference 1.

Impostors can save a bad hand... or make it worse. Such
is the gambler's life on the Outer Rim.

SCORING & PENALTIES
───────────────────
After 3 turns, all players reveal their hands. The best
hand wins the round.

  · The WINNER recovers all chips invested this round.
    Their chips come back to their reserve.

  · Losers with a SABACC hand (difference 0, but not the
    best) lose only 1 chip as a penalty.

  · Losers with a NON-SABACC hand lose chips equal to
    the difference between their two cards.
    Example: Sand 6 + Blood 1 = difference 5 = lose 5!

Penalty chips are destroyed — removed from the game
entirely, not given to the winner. The pot shrinks as
players bleed out.

If multiple players tie for best hand, they ALL recover
their invested chips. Penalties still apply to the rest.

ELIMINATION & VICTORY
─────────────────────
A player who runs out of chips is eliminated from the
game. No chips, no seat at the table.

The last player with chips remaining wins the game and
claims the entire credit pot. May the odds be ever in
your favour... or at least better than Lando's.

SHIFT TOKENS
────────────
Shift Tokens are the wild cards of the metagame. Each
player receives a random set at the start. A token can
only be used ONCE per game (not per round). You play a
token at the start of your turn, before choosing Draw
or Stand.

Some tokens help you:
  · FreeDraw — Draw without paying a chip this turn.
  · Refund — Recover 2 of your invested chips.
  · ExtraRefund — Recover 3 invested chips.
  · Immunity — Block all token effects from opponents
    until the end of the round.

Some tokens hurt others:
  · GeneralTariff — Every opponent pays 1 chip.
  · TargetTariff — One chosen opponent pays 2 chips.
  · Embargo — The next player must Stand (no drawing).
  · Embezzlement — Steal 1 chip from each opponent.
  · GeneralAudit — Players who chose Stand pay 2 chips.
  · TargetAudit — One chosen Standing player pays 3.
  · Exhaustion — Force a target to discard their hand
    and draw a completely new one.

Some tokens change the rules:
  · Markdown — Sylops count as 0 instead of copying.
  · MajorFraud — All Impostors are fixed at value 6.
  · CookTheBooks — Inverts Sabacc rankings! The worst
    pair (6-6) becomes the best. Chaos ensues.
  · DirectTransaction — Swap your hand with a target.
  · PrimeSabacc — Roll dice and pick a value. If your
    hand matches, you hold the second-best hand in the
    game.

Use them wisely. A well-timed token can turn a losing
round into a devastating victory.";

/// Top-level render function called from the main loop.
pub fn render(frame: &mut Frame, app: &AppState) {
    match app.screen {
        Screen::MainMenu => render_menu(frame, app),
        Screen::Setup => render_setup(frame, app),
        Screen::HowToPlay => render_how_to_play(frame, app),
        Screen::Playing => render_playing(frame, app),
    }

    // Help overlay on top of everything
    if app.tui.show_help {
        render_help(frame);
    }
}

// ── Shared menu chrome ──────────────────────────────────────────────

/// Renders the amber rounded border + starfield background.
/// Returns the inner area (inside the border).
fn render_chrome(frame: &mut Frame, app: &AppState) -> Rect {
    let area = frame.area();
    let border = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(AMBER));
    let inner = border.inner(area);
    frame.render_widget(border, area);

    // Starfield background
    frame.render_widget(StarfieldWidget::new(&app.starfield), inner);

    inner
}

/// Parse a description line, rendering `{text}` segments as strikethrough.
fn parse_desc_line(line: &str) -> Line<'_> {
    let gray = Style::default().fg(Color::Gray);
    let strike = Style::default()
        .fg(Color::Gray)
        .add_modifier(Modifier::CROSSED_OUT);

    let mut spans: Vec<Span> = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('{') {
        if start > 0 {
            spans.push(Span::styled(&rest[..start], gray));
        }
        if let Some(end) = rest[start..].find('}') {
            spans.push(Span::styled(&rest[start + 1..start + end], strike));
            rest = &rest[start + end + 1..];
        } else {
            break;
        }
    }
    if !rest.is_empty() {
        spans.push(Span::styled(rest, gray));
    }
    Line::from(spans)
}

// ── Main Menu ───────────────────────────────────────────────────────

fn render_menu(frame: &mut Frame, app: &AppState) {
    let inner = render_chrome(frame, app);

    // Reserve 1 line at the very bottom for hints (sticky footer)
    let hints_y = inner.bottom().saturating_sub(1);

    // Content area = inner minus the footer line
    let content = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));

    let title_height = TITLE_ART.len() as u16 + TITLE_SHADOW_OFFSET.1; // +1 for shadow

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),              // flex top
            Constraint::Length(title_height), // title ASCII art
            Constraint::Length(1),           // spacer between title and subtitle
            Constraint::Length(1),           // subtitle
            Constraint::Length(1),           // separator
            Constraint::Length(2),           // description
            Constraint::Length(1),           // spacer
            Constraint::Length(5),           // menu items (5 items, compact)
            Constraint::Min(1),              // flex bottom
        ])
        .split(content);

    // Title ASCII art — shadow first, then main text on top
    let title_char_width = TITLE_ART[0].chars().count() as u16;
    let title_x = inner.x + inner.width.saturating_sub(title_char_width) / 2;
    let shadow_style = Style::default().fg(Color::Rgb(80, 65, 25));
    let main_style = Style::default().fg(AMBER).add_modifier(Modifier::BOLD);

    // Shadow pass (offset +1 col, +1 row)
    let (sx, sy) = TITLE_SHADOW_OFFSET;
    for (i, line) in TITLE_ART.iter().enumerate() {
        let y = layout[1].y + i as u16 + sy;
        let x = title_x + sx;
        if y < layout[1].bottom() + 1 {
            // Render shadow char by char (only block chars cast shadow)
            let shadow_line: String = line.chars().map(|c| if c == '█' { '█' } else { ' ' }).collect();
            let sw = shadow_line.chars().count() as u16;
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(shadow_line, shadow_style))),
                Rect::new(x, y, sw, 1),
            );
        }
    }

    // Main text pass
    for (i, line) in TITLE_ART.iter().enumerate() {
        let y = layout[1].y + i as u16;
        if y < layout[1].bottom() {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    *line,
                    main_style,
                ))),
                Rect::new(title_x, y, title_char_width, 1),
            );
        }
    }

    // Subtitle
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "K  E  S  S  E  L",
        Style::default().fg(Color::Gray),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, layout[3]);

    // Separator — thin amber line
    let sep_width = 30u16.min(inner.width);
    let sep_x = inner.x + (inner.width.saturating_sub(sep_width)) / 2;
    let sep_line = "─".repeat(sep_width as usize);
    let sep = Paragraph::new(Line::from(Span::styled(
        sep_line,
        Style::default().fg(AMBER),
    )));
    frame.render_widget(
        sep,
        Rect::new(sep_x, layout[4].y, sep_width, 1),
    );

    // Description — changes with selected item
    let selected_item = MenuItem::ALL[app.menu.selected];
    let desc_text: Vec<Line> = selected_item
        .description()
        .lines()
        .map(|l| parse_desc_line(l))
        .collect();
    let desc = Paragraph::new(desc_text).alignment(Alignment::Center);
    frame.render_widget(desc, layout[5]);

    // Menu items — compact, no spacing between items
    let menu_lines: Vec<Line> = MenuItem::ALL
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let selected = i == app.menu.selected;
            let (prefix, style) = if *item == MenuItem::Quit {
                if selected {
                    ("▶ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                } else {
                    ("  ", Style::default().fg(Color::DarkGray))
                }
            } else if selected {
                ("▶ ", Style::default().fg(AMBER).add_modifier(Modifier::BOLD))
            } else {
                ("  ", Style::default().fg(Color::DarkGray))
            };
            Line::from(vec![
                Span::raw(prefix),
                Span::styled(item.label(), style),
            ])
        })
        .collect();
    let menu = Paragraph::new(menu_lines).alignment(Alignment::Center);
    frame.render_widget(menu, layout[7]);

    // Hints — sticky footer, always at the last line of inner
    let hints = Paragraph::new(Line::from(Span::styled(
        "↑↓ Navigate  ·  Enter Select  ·  q Quit",
        Style::default().fg(Color::DarkGray),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(hints, Rect::new(inner.x, hints_y, inner.width, 1));
}

// ── How to Play ─────────────────────────────────────────────────────

fn render_how_to_play(frame: &mut Frame, app: &AppState) {
    let inner = render_chrome(frame, app);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // title
            Constraint::Min(1),   // content
            Constraint::Length(1), // hints
        ])
        .split(inner);

    // Title
    let title = Paragraph::new(Line::from(Span::styled(
        "HOW TO PLAY",
        Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(title, layout[0]);

    // Content — centered with padding
    let content_width = 60u16.min(layout[1].width.saturating_sub(4));
    let content_x = layout[1].x + (layout[1].width.saturating_sub(content_width)) / 2;
    let content_area = Rect::new(content_x, layout[1].y, content_width, layout[1].height);

    let rules_lines: Vec<Line> = RULES_TEXT
        .lines()
        .map(|l| {
            // Section headers (all-caps lines or lines with ─) in amber
            if l.chars().all(|c| c == '─' || c.is_whitespace()) && l.contains('─') {
                Line::from(Span::styled(l, Style::default().fg(AMBER)))
            } else if !l.is_empty()
                && l.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && !l.contains("  ")
                && l.len() < 30
            {
                Line::from(Span::styled(
                    l,
                    Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(l, Style::default().fg(Color::White)))
            }
        })
        .collect();

    let scroll = app.how_to_play.scroll_offset as u16;
    let para = Paragraph::new(rules_lines)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(para, content_area);

    // Hints
    let hints = Paragraph::new(Line::from(Span::styled(
        "↑↓/PgUp/PgDn Scroll  ·  Esc Back",
        Style::default().fg(Color::DarkGray),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(hints, layout[2]);
}

// ── Setup screen ─────────────────────────────────────────────────────

fn render_setup(frame: &mut Frame, app: &AppState) {
    let inner = render_chrome(frame, app);

    // Center the form box (max 50 wide, 16 tall)
    let form_w = 50u16.min(inner.width.saturating_sub(4));
    let form_h = 16u16.min(inner.height.saturating_sub(2));
    let form_x = inner.x + (inner.width.saturating_sub(form_w)) / 2;
    let form_y = inner.y + (inner.height.saturating_sub(form_h)) / 2;
    let form_area = Rect::new(form_x, form_y, form_w, form_h);

    // Form with border, title, and footer hints
    render_setup_form(frame, form_area, &app.setup);
}

fn render_setup_form(frame: &mut Frame, area: Rect, setup: &SetupState) {
    let title_line = Line::from(Span::styled(
        " Custom Game ",
        Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
    ));
    let footer_line = Line::from(Span::styled(
        " Tab/↑↓ Navigate · ◀▶ Modify · Enter Start · Esc Back ",
        Style::default().fg(Color::DarkGray),
    ));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title_line)
        .title_bottom(footer_line);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut fields: Vec<(&str, String)> = vec![
        ("Player name", format!("{}_", setup.player_name)),
        ("Number of bots", format!("◀ {} ▶", setup.num_bots)),
        (
            "Buy-in",
            format!(
                "◀ {} credits ({} chips) ▶",
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
            "Tokens per player",
            format!("◀ {} ▶", setup.tokens_per_player),
        ));
    } else {
        fields.push(("Tokens per player", "—".into()));
    }

    // Render form fields
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
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let prefix = if selected { "▶ " } else { "  " };
        let value_y = y + 1;

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

    // START button — separated by a blank line
    let start_field_idx = SetupState::NUM_FIELDS - 1;
    let start_selected = setup.selected_field == start_field_idx;
    let start_y = inner.y + (fields.len() as u16) * 2 + 1; // +1 for blank line separator

    if start_y < inner.bottom() {
        let btn_text = "[ START GAME ]";
        let btn_style = if start_selected {
            Style::default()
                .fg(Color::Black)
                .bg(AMBER)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if start_selected { "▶ " } else { "  " };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(btn_text, btn_style),
            ])),
            Rect::new(inner.x + 2, start_y, inner.width.saturating_sub(2), 1),
        );
    }
}

// ── Playing screen ───────────────────────────────────────────────────

fn render_playing(frame: &mut Frame, app: &AppState) {
    let area = frame.area();

    // Minimum size guard
    if area.width < 120 || area.height < 30 {
        let msg = format!(
            "Terminal too small ({}×{})\nMinimum required: 120×30",
            area.width, area.height
        );
        let para = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        let y = area.height / 2;
        frame.render_widget(para, Rect::new(area.x, y.saturating_sub(1), area.width, 3));
        return;
    }

    // Build dynamic border title: "Round N · Turn N · Phase"
    let is_human = app.is_human_turn();
    let title_line = match &app.game {
        Some(g) => {
            let (phase_text, phase_color) =
                widgets::header::phase_label_styled(&g.phase, is_human);
            let dot_style = Style::default().fg(Color::DarkGray);
            let phase_style = if matches!(g.phase, sabacc_core::game::GamePhase::GameOver { .. }) {
                Style::default()
                    .fg(phase_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(phase_color)
            };
            Line::from(vec![
                Span::styled(
                    format!(" Round {} ", g.round),
                    Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
                ),
                Span::styled("· ", dot_style),
                Span::styled(format!("Turn {}/3 ", g.turn), Style::default().fg(Color::White)),
                Span::styled("· ", dot_style),
                Span::styled(format!("{} ", phase_text), phase_style),
            ])
        }
        None => Line::from(Span::styled(
            " Kessel Sabacc ",
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        )),
    };

    // Build dynamic footer hints
    let footer_text = match &app.game {
        Some(g) => match &g.phase {
            sabacc_core::game::GamePhase::TurnAction if is_human => {
                "↑↓ Navigate · Enter Select · s Token · ? Help"
            }
            sabacc_core::game::GamePhase::TurnAction => "? Help · q Quit",
            sabacc_core::game::GamePhase::ChoosingDiscard { .. } => {
                "1/2 Choose · Enter Confirm"
            }
            sabacc_core::game::GamePhase::Reveal { .. }
            | sabacc_core::game::GamePhase::RoundEnd => "Enter Continue · ↑↓ Scroll",
            sabacc_core::game::GamePhase::GameOver { .. } => "Enter New game · q Quit",
            sabacc_core::game::GamePhase::ImpostorReveal { .. }
            | sabacc_core::game::GamePhase::PrimeSabaccChoice { .. } => {
                "↑↓ Navigate · Enter Confirm"
            }
            _ => "? Help · q Quit",
        },
        None => "? Help · q Quit",
    };
    let footer_line = Line::from(Span::styled(
        format!(" {} ", footer_text),
        Style::default().fg(Color::DarkGray),
    ));

    // Amber rounded border
    let border = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(AMBER))
        .title(title_line)
        .title_bottom(footer_line);
    let inner = border.inner(area);
    frame.render_widget(border, area);

    // Horizontal: 3 columns
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // players
            Constraint::Min(60),    // center (game)
            Constraint::Length(27), // log
        ])
        .split(inner);

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
    let h = 19u16.min(area.height);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup = Rect::new(x, y, w, h);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Help — Kessel Sabacc ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab/←→    Navigate between actions"),
        Line::from("  ↑↓        Navigate overlay lists"),
        Line::from("  Enter     Confirm selection"),
        Line::from("  d         Enter draw mode (pick source)"),
        Line::from("  1-4       Direct source selection"),
        Line::from("  s         Play a ShiftToken"),
        Line::from("  Esc       Cancel / Back"),
        Line::from("  Space     Skip animations"),
        Line::from("  PgUp/PgDn Scroll log"),
        Line::from("  ?         This help"),
        Line::from("  q         Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Goal: keep 2 cards close in value!",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Press any key to close.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(help_text).block(block);
    frame.render_widget(para, popup);
}
