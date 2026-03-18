/// Phase label helper — used by the border title in ui.rs.
use ratatui::style::Color;

use sabacc_core::game::GamePhase;

/// Returns the phase label and its colour for the playing-screen border title.
pub fn phase_label_styled(phase: &GamePhase, is_human: bool) -> (String, Color) {
    match phase {
        GamePhase::Setup => ("Setup".into(), Color::Gray),
        GamePhase::TurnAction => {
            if is_human {
                ("Your turn".into(), Color::Rgb(232, 192, 80))
            } else {
                ("Bots playing...".into(), Color::DarkGray)
            }
        }
        GamePhase::ChoosingDiscard { .. } => ("Discard Choice".into(), Color::Cyan),
        GamePhase::ImpostorReveal { .. } => ("Impostor Reveal".into(), Color::Magenta),
        GamePhase::PrimeSabaccChoice { .. } => ("Prime Sabacc".into(), Color::Magenta),
        GamePhase::Reveal { .. } => ("Reveal".into(), Color::Yellow),
        GamePhase::RoundEnd => ("Round End".into(), Color::Yellow),
        GamePhase::GameOver { .. } => {
            ("Game Over".into(), Color::Rgb(232, 192, 80))
        }
    }
}
