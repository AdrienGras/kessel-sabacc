# Round Announcement Progress Bar

**Date:** 2026-03-18
**Status:** Approved
**Scope:** `sabacc-cli` — `widgets/actions.rs` (rendering) + `app.rs` (shared constant)

## Summary

Add a visual progress bar to the `RoundAnnouncement` overlay that fills from 0% to 100% over the ~2s auto-dismiss countdown, giving the player a visual cue of the remaining time.

## Design

### Visual

- Characters: `▰` (filled) / `▱` (empty) — single-width Unicode, 3 bytes each but 1 column in terminal
- Color: `Color::Rgb(232, 192, 80)` (amber Sand)
- Position: last line inside the overlay, before the bottom border, horizontally centered
- Width: `inner.width - 2` (1 char padding each side)

### Example (illustrative, actual popup is 27 chars wide / 25 inner)

```
┌─────────────────────────┐
│       ⚔ Round 3 ⚔       │
│                         │
│  3 players remaining    │
│  Chip leader: Lando (5) │
│                         │
│  ▰▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱  │
└─────────────────────────┘
```

### Behavior

- Bar fills left-to-right as time passes (0% at start → 100% at dismiss)
- Progress formula (integer math): `let filled = ((ROUND_ANNOUNCE_TOTAL_TICKS - ticks_remaining) as usize * bar_width) / ROUND_ANNOUNCE_TOTAL_TICKS as usize;`
- Skip (Enter/Space) dismisses the overlay as before — bar disappears with it
- No new state fields needed — reuses existing `ticks_remaining` from `Overlay::RoundAnnouncement`

### Style choices

| Decision | Choice | Reason |
|----------|--------|--------|
| Fill direction | 0% → 100% (filling) | Feels like "loading next round" |
| Characters | `▰▱` (thin blocks) | Finer, more polished than `█░` |
| Color | Amber Sand | Consistent with game theme |
| Position | Footer of overlay, centered | Non-intrusive, consistent with centered title |

## Files affected

| File | Change |
|------|--------|
| `crates/sabacc-cli/src/app.rs` | Extract `pub const ROUND_ANNOUNCE_TOTAL_TICKS: u16 = 60;` (replaces hardcoded `60` in overlay creation) |
| `crates/sabacc-cli/src/widgets/actions.rs` | Bind `ticks_remaining` in match arm (currently `..`), add progress bar rendering, increase popup height from `7` to `8` |

## Implementation notes

- The match arm for `Overlay::RoundAnnouncement` currently uses `..` to ignore `ticks_remaining` — must destructure it explicitly
- Popup height changes from `centered_popup(area, 27, 7)` to `centered_popup(area, 27, 8)` to accommodate the new line
- The constant `ROUND_ANNOUNCE_TOTAL_TICKS` should be defined in `app.rs` and used both at overlay creation (`ticks_remaining: ROUND_ANNOUNCE_TOTAL_TICKS`) and in the rendering formula to avoid magic number duplication

## Constants

```rust
// In app.rs (shared)
pub const ROUND_ANNOUNCE_TOTAL_TICKS: u16 = 60;

// In widgets/actions.rs (rendering only)
const PROGRESS_FILLED: char = '▰';
const PROGRESS_EMPTY: char = '▱';
```
