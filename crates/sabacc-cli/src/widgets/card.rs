/// ASCII card rendering widget.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use sabacc_core::card::{CardValue, Family};

/// Colour palette for card families.
pub const SAND_COLOR: Color = Color::Rgb(232, 192, 80);
pub const BLOOD_COLOR: Color = Color::Rgb(232, 72, 72);
pub const SYLOP_COLOR: Color = Color::Rgb(144, 144, 224);
pub const IMPOSTOR_COLOR: Color = Color::Rgb(112, 112, 112);
pub const FACE_DOWN_COLOR: Color = Color::Rgb(60, 60, 60);

/// A single card rendered as an 8×5 ASCII block.
#[derive(Debug, Clone)]
pub struct CardWidget {
    pub family: Option<Family>,
    pub value: Option<CardValue>,
    pub face_down: bool,
    pub selected: bool,
    pub highlighted: bool,
    /// Resolved impostor die value (shown instead of '?').
    pub resolved_impostor: Option<u8>,
}

impl CardWidget {
    pub fn from_card(card: &sabacc_core::card::Card, face_down: bool) -> Self {
        Self {
            family: Some(card.family),
            value: Some(card.value),
            face_down,
            selected: false,
            highlighted: false,
            resolved_impostor: None,
        }
    }

    pub fn face_down() -> Self {
        Self {
            family: None,
            value: None,
            face_down: true,
            selected: false,
            highlighted: false,
            resolved_impostor: None,
        }
    }

    fn accent_color(&self) -> Color {
        if self.face_down {
            return FACE_DOWN_COLOR;
        }
        match (&self.family, &self.value) {
            (_, Some(CardValue::Sylop)) => SYLOP_COLOR,
            (_, Some(CardValue::Impostor)) => IMPOSTOR_COLOR,
            (Some(Family::Sand), _) => SAND_COLOR,
            (Some(Family::Blood), _) => BLOOD_COLOR,
            _ => FACE_DOWN_COLOR,
        }
    }

    fn symbol(&self) -> &str {
        if self.face_down {
            return "▓▓";
        }
        match &self.value {
            Some(CardValue::Sylop) => "◎",
            Some(CardValue::Impostor) if self.resolved_impostor.is_some() => "?→",
            Some(CardValue::Impostor) => "?",
            _ => match &self.family {
                Some(Family::Sand) => "△",
                Some(Family::Blood) => "◇",
                None => "▓▓",
            },
        }
    }

    fn label(&self) -> String {
        if self.face_down {
            return "????".into();
        }
        match &self.value {
            Some(CardValue::Sylop) => "SYLOP".into(),
            Some(CardValue::Impostor) => "IMPOS".into(),
            Some(CardValue::Number(_)) => match &self.family {
                Some(Family::Sand) => "SAND".into(),
                Some(Family::Blood) => "BLOOD".into(),
                None => "?".into(),
            },
            None => "????".into(),
        }
    }

    fn value_str(&self) -> String {
        if self.face_down {
            return "▓▓".into();
        }
        match &self.value {
            Some(CardValue::Number(n)) => format!("{n}"),
            Some(CardValue::Sylop) => String::new(),
            Some(CardValue::Impostor) => {
                match self.resolved_impostor {
                    Some(v) => format!("{v}"),
                    None => String::new(),
                }
            }
            None => "▓▓".into(),
        }
    }

    /// Render as an inline compact string: `[S△ 3]`.
    pub fn inline_string(&self) -> (String, Color) {
        let color = self.accent_color();
        if self.face_down {
            return ("[??]".into(), FACE_DOWN_COLOR);
        }
        let prefix = match &self.family {
            Some(Family::Sand) => "S",
            Some(Family::Blood) => "B",
            None => "?",
        };
        let sym = self.symbol();
        let val = self.value_str();
        if val.is_empty() {
            (format!("[{prefix}{sym}]"), color)
        } else {
            (format!("[{prefix}{sym}{val}]"), color)
        }
    }
}

impl Widget for CardWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 8 || area.height < 5 {
            // Too small — render inline
            let (text, color) = self.inline_string();
            let style = Style::default().fg(color);
            buf.set_string(area.x, area.y, &text, style);
            return;
        }

        let accent = self.accent_color();
        let border_style = if self.selected {
            Style::default().fg(Color::White)
        } else if self.highlighted {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(accent)
        };
        let text_style = Style::default().fg(accent);

        // Line 0: ┌──────┐
        buf.set_string(area.x, area.y, "┌──────┐", border_style);
        // Line 1: │LABEL │
        let label = self.label();
        let padded = format!("│{:<6}│", label);
        buf.set_string(area.x, area.y + 1, &padded, border_style);
        // Colorize the label text
        for (i, ch) in label.chars().enumerate().take(6) {
            buf.set_string(
                area.x + 1 + i as u16,
                area.y + 1,
                ch.to_string(),
                text_style,
            );
        }
        // Line 2: │  △   │
        let sym = self.symbol();
        let sym_line = format!("│  {:<4}│", sym);
        buf.set_string(area.x, area.y + 2, &sym_line, border_style);
        buf.set_string(area.x + 3, area.y + 2, sym, text_style);
        // Line 3: │  3   │
        let val = self.value_str();
        let val_line = format!("│  {:<4}│", val);
        buf.set_string(area.x, area.y + 3, &val_line, border_style);
        if !val.is_empty() {
            buf.set_string(area.x + 3, area.y + 3, &val, text_style);
        }
        // Line 4: └──────┘
        buf.set_string(area.x, area.y + 4, "└──────┘", border_style);
    }
}
