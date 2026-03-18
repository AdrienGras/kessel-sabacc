/// Animated starfield background widget.
///
/// Renders scattered star characters at various brightness levels into a
/// Ratatui buffer.  Stars drift downward at different speeds and wrap around
/// when they reach the bottom.
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

/// A single star in the field.
#[derive(Debug, Clone)]
pub struct Star {
    /// Column position.
    pub x: u16,
    /// Row position.
    pub y: u16,
    /// Ticks between each downward move (1 = fast, 3 = slow).
    pub speed: u8,
    /// 0 = dim, 1 = normal, 2 = bright.
    pub brightness: u8,
}

/// Persistent starfield state shared across screens.
#[derive(Debug, Clone)]
pub struct Starfield {
    pub stars: Vec<Star>,
    pub tick_counter: u16,
    pub width: u16,
    pub height: u16,
}

impl Starfield {
    /// Number of stars to generate.
    const NUM_STARS: usize = 60;

    /// Create a new starfield filling `width`×`height`.
    pub fn new(width: u16, height: u16, rng: &mut impl Rng) -> Self {
        let stars = Self::generate_stars(width, height, rng);
        Self {
            stars,
            tick_counter: 0,
            width,
            height,
        }
    }

    /// Advance stars by one tick (~33 ms).
    pub fn tick(&mut self) {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        for star in &mut self.stars {
            // Move down every `speed` ticks
            if self.tick_counter.is_multiple_of(star.speed as u16) {
                star.y += 1;
                if star.y >= self.height {
                    star.y = 0;
                }
            }
        }
    }

    /// Recalculate on terminal resize.
    pub fn resize(&mut self, width: u16, height: u16, rng: &mut impl Rng) {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.stars = Self::generate_stars(width, height, rng);
        }
    }

    fn generate_stars(width: u16, height: u16, rng: &mut impl Rng) -> Vec<Star> {
        (0..Self::NUM_STARS)
            .map(|_| Star {
                x: rng.gen_range(0..width.max(1)),
                y: rng.gen_range(0..height.max(1)),
                speed: rng.gen_range(1..=3),
                brightness: rng.gen_range(0..=2),
            })
            .collect()
    }
}

/// Stateless widget that renders a [`Starfield`] reference into a buffer.
pub struct StarfieldWidget<'a> {
    starfield: &'a Starfield,
}

impl<'a> StarfieldWidget<'a> {
    pub fn new(starfield: &'a Starfield) -> Self {
        Self { starfield }
    }
}

impl Widget for StarfieldWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for star in &self.starfield.stars {
            let x = area.x + star.x;
            let y = area.y + star.y;
            if x >= area.right() || y >= area.bottom() {
                continue;
            }
            let (ch, color) = match star.brightness {
                0 => ('·', Color::DarkGray),
                1 => ('•', Color::Gray),
                _ => ('✦', Color::White),
            };
            buf[(x, y)].set_char(ch).set_style(Style::default().fg(color));
        }
    }
}
