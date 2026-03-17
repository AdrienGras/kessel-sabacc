# Spec: sabacc-cli — Frontend TUI Ratatui

**Date:** 2026-03-17
**Status:** Draft
**Crate:** `crates/sabacc-cli`

---

## Contexte

`sabacc-core` (v0.3.0) est feature-complete : 16 ShiftTokens, scoring, bot IA, 96 tests. Il manque un frontend jouable. Cette spec décrit la TUI Ratatui — le premier frontend interactif du Kessel Sabacc.

## Portée

- 1 joueur humain + 1-7 bots (BasicBot)
- Partie complète de bout en bout (setup → game over)
- Pas de multijoueur local (un seul humain au terminal)

---

## Architecture : Hybride Elm + Widgets

État centralisé avec `update()` pur (Elm), rendu distribué par widgets modulaires.

### Flux de données

```
Event (crossterm/tick) → EventHandler → AppEvent
AppEvent → update(state, event) → (new_state, Command)
new_state → render(frame, &app) → widgets::*.render()
Command → side effects (bot actions, quit, etc.)
```

### AppState

```rust
pub struct AppState {
    pub game: GameState,                    // sabacc-core
    pub tui: TuiState,                      // UI state
    pub anim_queue: VecDeque<Animation>,    // pending animations
    pub current_anim: Option<ActiveAnimation>,
    pub log: Vec<LogEntry>,                 // action history
}

pub struct TuiState {
    pub focus: Focus,           // which widget has focus
    pub selected_action: usize, // cursor in ActionBar
    pub selected_token: usize,  // cursor in token list
    pub selected_target: usize, // cursor in TargetPicker
    pub overlay: Option<Overlay>,
    pub should_quit: bool,
    pub show_help: bool,
}

pub enum Overlay {
    SourcePicker,                // choose draw source
    TokenPicker,                 // choose ShiftToken
    TargetPicker,                // choose target player (highlight on board)
    DiscardChoice { drawn: Card }, // keep or discard
    ImpostorChoice { die1: u8, die2: u8 },
    PrimeSabaccChoice { die1: u8, die2: u8 },
    QuitConfirm,
    GameOver { winner_name: String, is_human: bool },
}
```

### AppEvent

```rust
pub enum AppEvent {
    Key(KeyEvent),
    Tick,           // animation frame (~33ms)
    Resize(u16, u16),
}

pub enum Command {
    None,
    RunBots,        // advance_bots() then queue animations
    Quit,
}
```

---

## Structure des fichiers

```
crates/sabacc-cli/
├── Cargo.toml
└── src/
    ├── main.rs          # terminal setup, main loop, cleanup
    ├── app.rs           # AppState, update(), AppEvent, Command
    ├── animation.rs     # Animation, ActiveAnimation, AnimationQueue
    ├── events.rs        # EventHandler: crossterm → AppEvent
    ├── ui.rs            # render() dispatch + responsive layout
    └── widgets/
        ├── mod.rs
        ├── header.rs    # HeaderBar
        ├── players.rs   # PlayersPanel
        ├── table.rs     # TableCenter
        ├── hand.rs      # PlayerHand
        ├── card.rs      # CardWidget (ASCII art)
        ├── actions.rs   # ActionBar + overlays
        └── log.rs       # LogPanel
```

### Dépendances

```toml
[dependencies]
sabacc-core = { path = "../sabacc-core" }
ratatui = { version = ">=0.29", features = ["crossterm"] }
crossterm = ">=0.28"
rand = "0.8"
clap = { version = "4", features = ["derive"] }
```

`clap` pour le parsing CLI args (--quick, --bots, --buy-in, --name, --no-tokens).

---

## Layout responsive

### Large (≥120 cols) — 3 colonnes

```
┌─ Header ────────────────────────────────────────────────────┐
├──────────────┬──────────────────────────┬───────────────────┤
│ PlayersPanel │      TableCenter         │   PlayerHand      │
│ (joueurs +   │  (decks + défausses)     │   + ActionBar     │
│  jetons)     │                          │   + ShiftTokens   │
├──────────────┴──────────────────────────┴───────────────────┤
│ LogPanel                                                     │
└──────────────────────────────────────────────────────────────┘
```

### Compact (<120 cols) — empilé

```
┌─ Header ─────────────────────────────┐
│ PlayersPanel (inline) + TableCenter  │
├──────────────────────────────────────┤
│ PlayerHand + ActionBar               │
├──────────────────────────────────────┤
│ LogPanel                             │
└──────────────────────────────────────┘
```

Le breakpoint est dynamique : `if area.width >= 120 { large } else { compact }`.

---

## Rendu des cartes (CardWidget)

Blocs ASCII riches avec box-drawing Unicode et couleurs RGB.

```
┌──────┐
│ SAND │   couleur: #E8C050 (ambre)
│  △   │   symbole: △ triangle
│  3   │
└──────┘

┌──────┐
│BLOOD │   couleur: #E84848 (rouge)
│  ◇   │   symbole: ◇ losange
│  5   │
└──────┘

┌──────┐
│SYLOP │   couleur: #9090E0 (violet)
│  ◎   │   symbole: ◎ double-cercle
│      │
└──────┘

┌──────┐
│IMPOST│   couleur: #707070 (gris)
│  ?   │   symbole: ?
│      │
└──────┘

┌──────┐
│ ???? │   face cachée: fond sombre
│  ▓▓  │
│  ▓▓  │
└──────┘
```

En mode compact (<120 cols), les cartes passent en format inline : `[S△ 3]` `[B◇ 5]`.

---

## Système d'animations

### Types d'animation

```rust
pub enum Animation {
    CardFlash { player_id: PlayerId, family: Family, duration_ms: u64 },
    ChipChange { player_id: PlayerId, delta: i8, duration_ms: u64 },
    PlayerHighlight { player_id: PlayerId, color: Color, duration_ms: u64 },
    CardReveal { player_id: PlayerId, delay_ms: u64 },
    LogMessage { text: String },
    Pause { duration_ms: u64 },
}

pub struct ActiveAnimation {
    pub animation: Animation,
    pub elapsed_ms: u64,
}
```

### Tick loop

```
Mode Idle (pas d'animation) :
  → poll crossterm avec timeout infini → 0% CPU

Mode Animating :
  → poll crossterm avec timeout 33ms (~30fps)
  → si Tick: elapsed_ms += 33, vérifier si animation finie
  → si finie: passer à la suivante dans la queue
  → si queue vide: retour en mode Idle
```

### Skip

`Space` pendant une animation → vide la queue, applique tous les effets restants instantanément.

### Séquence de révélation

```
Pour chaque joueur (dans l'ordre du dealer) :
  1. PlayerHighlight { color: Yellow, 300ms }
  2. CardReveal { delay: 500ms }
  3. LogMessage { "Bot_1: Sand 3 + Blood 3 → Sabacc (3)" }
  4. Pause { 300ms }
Puis :
  5. LogMessage { "Gagnant: Vous ! Pure Sabacc" }
  6. ChipChange pour chaque perdant
```

---

## Mapping GamePhase → Écran TUI

| GamePhase | Écran TUI | Input attendu |
|-----------|-----------|---------------|
| `Setup` | Menu interactif (ou skip avec --quick) | Tab/Enter dans le formulaire |
| `TurnAction` (humain) | ActionBar active: Draw / Stand / Token | Tab + Enter, ou `s` pour tokens |
| `TurnAction` (bot) | `advance_bots()` → animations | Automatique, Space pour skip |
| `ChoosingDiscard` | Overlay: carte piochée + choix | Enter sur Garder/Défausser |
| `PrimeSabaccChoice` | Overlay: 2 valeurs de dé | Enter sur la valeur choisie |
| `ImpostorReveal` (humain) | Overlay: 2 valeurs de dé | Enter sur la valeur choisie |
| `ImpostorReveal` (bot) | Animation automatique | Automatique |
| `Reveal` | Révélation séquentielle animée | Space pour skip, Enter pour continuer |
| `RoundEnd` | Log résumé + bouton Manche Suivante | Enter → AdvanceRound |
| `GameOver` | Écran plein: victoire/défaite | Enter: nouvelle partie, q: quitter |

---

## Navigation clavier

| Touche | Contexte | Action |
|--------|----------|--------|
| `Tab` / `Shift+Tab` | ActionBar, overlays, TargetPicker | Naviguer entre options |
| `Enter` | Partout | Confirmer la sélection |
| `1`-`4` | Source picker | Sélection directe de la source |
| `s` | TurnAction | Ouvrir la liste des ShiftTokens |
| `Esc` | Overlay ouvert | Fermer l'overlay / annuler |
| `q` | Partout | Quitter (avec confirmation) |
| `?` | Partout | Aide / règles |
| `Space` | Animation en cours | Skip toutes les animations |

### Ciblage (TargetPicker)

Pour les ShiftTokens ciblés, les joueurs sont highlight sur le PlayersPanel. `Tab` navigue entre les cibles possibles (joueurs non éliminés, pas soi-même), `Enter` confirme.

---

## Écran de setup

### Menu interactif (défaut)

Formulaire navigable avec :
- Nom du joueur (input texte)
- Nombre de bots (1-7, ◀▶)
- Buy-in (50/100/150/200, ◀▶, affiche les jetons correspondants)
- ShiftTokens on/off (checkbox)
- Distribution (Random 4/joueur, Fixed, None — si tokens on)
- Bouton [LANCER LA PARTIE]

### CLI args (--quick)

```
sabacc-cli                           # → menu interactif
sabacc-cli --quick                   # → 3 bots, 100cr, tokens on
sabacc-cli --bots 2 --buy-in 50     # → config directe, pas de menu
sabacc-cli --name "Lando"            # → nom custom + menu pour le reste
sabacc-cli --no-tokens               # → désactive les ShiftTokens
```

Si au moins `--bots` ET `--buy-in` sont fournis, le menu est skip.

---

## Notes d'implémentation

### Initialisation de la partie

`new_game(config, rng)` crée le `GameState` initial, puis `apply_action(state, Action::StartGame, rng)` distribue les cartes. Ces deux appels sont distincts dans le core.

### Dual Impostor

Un joueur peut avoir 2 Imposteurs (sand + blood). L'overlay `ImpostorChoice` doit gérer ce cas : le core envoie `ImpostorReveal { pending }` avec potentiellement le même player_id pour les deux familles. L'overlay doit permettre de choisir une valeur par Imposteur.

### Transition Reveal → RoundEnd

`AdvanceRound` est accepté à la fois en phase `Reveal` (après les animations) et `RoundEnd`. L'UI doit envoyer `AdvanceRound` quand le joueur presse Enter après la révélation.

### Distribution Fixed

En mode `TokenDistribution::Fixed`, le setup menu ne propose pas de sélection manuelle — c'est un set prédéfini (utile pour le debug). Le menu affiche juste "Fixed (debug)" comme option.

---

## Gestion d'erreurs

- Les erreurs `GameError` de sabacc-core sont affichées dans le LogPanel en rouge
- Si une erreur invalide l'action (ex: pas assez de jetons), le joueur peut réessayer
- Panic-free : `main()` wrappe tout dans un catch pour restaurer le terminal proprement
- Le terminal est restauré (leave raw mode, show cursor) même en cas de panic via un panic hook

---

## Tests

- **Pas de tests UI automatisés** pour la v1 — les widgets sont du rendu pur
- **Tests manuels** : lancer `cargo run -p sabacc-cli` et jouer une partie complète
- `app.rs` update() est testable unitairement (état → événement → nouvel état)
- `animation.rs` est testable unitairement (progression, queue)
- Vérification clippy : `cargo clippy -p sabacc-cli -- -D warnings`
