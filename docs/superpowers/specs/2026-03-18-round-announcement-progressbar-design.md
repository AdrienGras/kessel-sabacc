# Round Announcement Progress Bar

**Date:** 2026-03-18
**Status:** Approved
**Scope:** `sabacc-cli` — `widgets/actions.rs` only

## Summary

Add a visual progress bar to the `RoundAnnouncement` overlay that fills from 0% to 100% over the ~2s auto-dismiss countdown, giving the player a visual cue of the remaining time.

## Design

### Visual

- Characters: `▰` (filled) / `▱` (empty)
- Color: `Color::Rgb(232, 192, 80)` (amber Sand)
- Position: last line inside the overlay, before the bottom border
- Width: fills the inner width of the popup (minus padding)

### Example (mid-progress ~1s)

```
┌─────────────────────────────┐
│        ⚔ Round 3 ⚔         │
│                             │
│   3 players remaining       │
│   Chip leader: Lando (5)    │
│                             │
│   ▰▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱   │
└─────────────────────────────┘
```

### Behavior

- Bar fills left-to-right as time passes (0% at start → 100% at dismiss)
- Progress ratio: `(TOTAL_TICKS - ticks_remaining) / TOTAL_TICKS` where `TOTAL_TICKS = 60`
- Skip (Enter/Space) dismisses the overlay as before — bar disappears with it
- No new state fields needed — reuses existing `ticks_remaining` from `Overlay::RoundAnnouncement`

### Style choices

| Decision | Choice | Reason |
|----------|--------|--------|
| Fill direction | 0% → 100% (filling) | Feels like "loading next round" |
| Characters | `▰▱` (thin blocks) | Finer, more polished than `█░` |
| Color | Amber Sand | Consistent with game theme |
| Position | Footer of overlay | Non-intrusive, doesn't compete with round info |

## Files affected

| File | Change |
|------|--------|
| `crates/sabacc-cli/src/widgets/actions.rs` | Add progress bar rendering in `RoundAnnouncement` overlay draw code |

No changes to `app.rs`, `animation.rs`, or tick handler — `ticks_remaining` is already decremented each tick.

## Constants

```rust
const ROUND_ANNOUNCE_TOTAL_TICKS: u16 = 60;
const PROGRESS_FILLED: char = '▰';
const PROGRESS_EMPTY: char = '▱';
```
