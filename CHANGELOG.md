# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] — 2026-03-19

### Added
- Complete documentation: README, RULES, CONTRIBUTING, CHANGELOG, LICENSE

## [0.16.0] — 2026-03-19

### Changed
- Refactored 32 code smells across sabacc-core and sabacc-cli
- Introduced TurnEphemeral for cleaner turn state management
- Decomposed large functions, deduplicated logic, added Display impls

## [0.15.0] — 2026-03-19

### Added
- ExpertBot with EV-optimised strategy
- BotDifficulty enum (Basic / Expert)
- Lando's Challenge menu mode
- Difficulty selector in Custom Game setup

## [0.14.0] — 2026-03-18

### Added
- 41 integration tests covering modifier interactions, impostor flow, and elimination
- GameStats tracking (draws, stands, tokens played, chips history)
- Enriched GameOver screen with chip history chart

## [0.13.0] — 2026-03-18

### Fixed
- Centralised global hotkeys (?, q) in main update loop
- Bot turn highlight with PlayerHighlight indicator
- Setup screen chrome improvements

## [0.12.0] — 2026-03-18

### Added
- Dice rolling slot-machine animation for Impostor and PrimeSabacc resolution

## [0.11.0] — 2026-03-17

### Changed
- Inline source selection on game table (replaces SourcePicker popup overlay)

## [0.10.0] — 2026-03-17

### Added
- Amber rounded border on playing screen
- Harmonised all panels with BorderType::Rounded

## [0.9.0] — 2026-03-17

### Added
- Immersive main menu with starfield animation
- How to Play screen

## [0.8.0] — 2026-03-17

### Added
- Progress bar on RoundAnnouncement overlay

### Fixed
- GameOver bypass when ≤1 player remaining

## [0.7.0] — 2026-03-17

### Added
- Auto-dismiss RoundAnnouncement overlay between rounds

## [0.6.0] — 2026-03-17

### Fixed
- Reveal animation freeze
- Double impostor crash
- Chip display inconsistencies

## [0.5.0] — 2026-03-17

### Changed
- All TUI text translated from French to English

## [0.4.0] — 2026-03-16

### Added
- sabacc-cli TUI frontend with Ratatui (3-column layout)

## [0.3.0] — 2026-03-16

### Added
- All 16 Shift Tokens implemented
- PrimeSabacc HandRank variant
- TokenDistribution configuration
- Bot AI for token strategy

## [0.2.0] — 2026-03-16

### Added
- Complete sabacc-core game engine
- Card, deck, hand, player, scoring, turn, round, and game modules
- BasicBot with BotStrategy trait
