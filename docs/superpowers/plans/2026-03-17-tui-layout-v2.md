# TUI Layout V2 Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the TUI playing screen from the current cramped layout to a 3-column design: Players | Game | Log.

**Architecture:** Replace `render_main_wide`/`render_main_compact` with a single 3-column layout. Minimum terminal size 120×30 enforced. Log gets PageUp/PageDown scroll.

**Tech Stack:** Rust, Ratatui 0.29, crossterm 0.28

---

### Task 1: Add log scroll state + PageUp/PageDown handling

**Files:**
- Modify: `crates/sabacc-cli/src/app.rs`

- [ ] **Step 1: Add scroll fields to TuiState**

Add `log_scroll_offset: usize` and `log_auto_scroll: bool` to `TuiState`, initialize in `Default`.

- [ ] **Step 2: Handle PageUp/PageDown in update_playing**

In `update_playing`, before overlay/quit handling, add PageUp/PageDown key handlers that adjust `log_scroll_offset` and `log_auto_scroll`.

- [ ] **Step 3: Auto-scroll adjustment in push_log**

In `push_log` and `push_error`, if `!log_auto_scroll`, increment `log_scroll_offset` by 1.

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p sabacc-cli`

- [ ] **Step 5: Commit**

```
git add crates/sabacc-cli/src/app.rs
git commit -m "♻️ refactor: add log scroll state and PageUp/PageDown handling"
```

---

### Task 2: Rewrite ui.rs — 3-column layout + min size guard

**Files:**
- Modify: `crates/sabacc-cli/src/ui.rs`

- [ ] **Step 1: Add minimum size guard**

In `render_playing`, check `area.width < 120 || area.height < 30`. If too small, render a centered error message and return.

- [ ] **Step 2: Replace layout with 3 columns**

Replace the current 4-row vertical layout (header/main/actionbar/log) with:
- Vertical: header (2) + main (Min)
- Main horizontal: players (Length 22) + center (Min 60) + log (Length 27)
- Center vertical: tapis (Length 7) + actions (Length 3) + hand (Min 10)

- [ ] **Step 3: Remove render_main_compact**

Delete `render_main_compact` function entirely.

- [ ] **Step 4: Update render calls**

- `widgets::header::render` on header area
- `widgets::players::render` on cols[0]
- `widgets::table::render` on center[0]
- `widgets::actions::render_bar` on center[1]
- `widgets::hand::render` on center[2]
- `widgets::log::render` on cols[2]
- `widgets::actions::render_overlay` on full area (if overlay)

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p sabacc-cli`

- [ ] **Step 6: Commit**

```
git add crates/sabacc-cli/src/ui.rs
git commit -m "♻️ refactor: replace layout with 3-column design + min size guard"
```

---

### Task 3: Rewrite players.rs — 3-line per player format

**Files:**
- Modify: `crates/sabacc-cli/src/widgets/players.rs`

- [ ] **Step 1: Rewrite render function**

Each player gets 3 lines:
- Line 1: `▶ Name` or `  Name` (indicator + name)
- Line 2: `  ●●●○○` (chips visual)
- Line 3: `  X rés. + Y pot` (detail text)
- 1 blank line separator

Eliminated players in compact mode (1 line, name crossed out) when height is tight. Truncation with `+N...` if overflow.

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p sabacc-cli`

- [ ] **Step 3: Commit**

```
git add crates/sabacc-cli/src/widgets/players.rs
git commit -m "♻️ refactor: players panel to 3-line format per player"
```

---

### Task 4: Rewrite table.rs — 4 cards horizontal, no compact

**Files:**
- Modify: `crates/sabacc-cli/src/widgets/table.rs`

- [ ] **Step 1: Rewrite render function**

Remove `render_large`/`render_compact` split. Single render:
- Line 0: Title "TABLE DE JEU" in Sand color bold
- Lines 1-5: 4 CardWidgets side by side: Déf Sand | Deck Sand | Deck Blood | Déf Blood
- Line 6: Labels under each card

Layout: 4 columns of Length(8) with spacers (Length 2 between same-family, Length 4 between families).

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p sabacc-cli`

- [ ] **Step 3: Commit**

```
git add crates/sabacc-cli/src/widgets/table.rs
git commit -m "♻️ refactor: table widget with 4 horizontal cards"
```

---

### Task 5: Rewrite hand.rs — cards + ●○ chips + token list

**Files:**
- Modify: `crates/sabacc-cli/src/widgets/hand.rs`

- [ ] **Step 1: Rewrite render function**

Horizontal split: cards left (Length 20), info right (Min 20).

Left side:
- Title "VOTRE MAIN" in Sand color bold
- 2 CardWidgets (8×5) side by side

Right side:
- `●●●○○` chips visual in Rgb(200,200,100)
- `X réserve + Y investis` detail in DarkGray
- Blank line
- "SHIFT TOKENS" title in Sand color bold
- Each token: name in Cyan, description in DarkGray indented 2 spaces
- If no tokens: `(aucun)` in DarkGray

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p sabacc-cli`

- [ ] **Step 3: Commit**

```
git add crates/sabacc-cli/src/widgets/hand.rs
git commit -m "♻️ refactor: hand widget with chips visual + token descriptions"
```

---

### Task 6: Rewrite log.rs — scroll support + truncation

**Files:**
- Modify: `crates/sabacc-cli/src/widgets/log.rs`

- [ ] **Step 1: Rewrite render function**

Use `app.tui.log_scroll_offset` and `app.tui.log_auto_scroll` for scroll position. Apply the rendering formula from spec:
```rust
let start = total.saturating_sub(visible + offset);
let end = start + visible.min(total - start);
```

Truncate entries with `…` if longer than available width. Show `▼ new` indicator at bottom if scrolled up and auto_scroll is off.

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p sabacc-cli`

- [ ] **Step 3: Commit**

```
git add crates/sabacc-cli/src/widgets/log.rs
git commit -m "♻️ refactor: log widget with PageUp/PageDown scroll"
```

---

### Task 7: Shorten log messages + clippy + fmt

**Files:**
- Modify: `crates/sabacc-cli/src/app.rs` (log messages throughout)

- [ ] **Step 1: Shorten all log messages to ~23 chars**

Replace verbose log messages with concise ones:
- "pioche depuis Deck Sand" → "Draw Deck S"
- "pioche depuis Défausse Blood" → "Draw Déf B"
- "choisit Stand" → "Stand"
- "joue FreeDraw" → "joue FreeDraw"
- etc.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -p sabacc-cli -- -D warnings`

- [ ] **Step 3: Run fmt**

Run: `cargo fmt -p sabacc-cli`

- [ ] **Step 4: Run tests**

Run: `cargo test -p sabacc-cli`

- [ ] **Step 5: Commit**

```
git add crates/sabacc-cli/
git commit -m "♻️ refactor: shorten log messages + clippy + fmt cleanup"
```
