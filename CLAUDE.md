# CLAUDE.md — Kessel Sabacc

## Règle absolue : consulter Context7 avant chaque étape

Avant tout plan d'implémentation ou toute écriture de code, tu dois consulter les
documentations à jour via Context7. Résous les IDs de librairies puis interroge
leur doc. Ne suppose jamais la syntaxe d'une API à partir de tes données
d'entraînement — elles peuvent être obsolètes.

Librairies à résoudre systématiquement selon le crate concerné :

- `sabacc-core` → aucune dépendance externe, pas de consultation nécessaire
- `sabacc-cli` → résoudre `ratatui` et `crossterm`
- `sabacc-wasm` → résoudre `wasm-bindgen` et `wasm-pack`
- `web/` → résoudre `svelte` et `vite`

Exemple de workflow attendu :
1. Appel Context7 `resolve-library-id` pour chaque librairie concernée
2. Appel Context7 `query-docs` sur les points précis à implémenter
3. Seulement ensuite : écrire le code

---

## Vue d'ensemble du projet

Reproduction jouable du **Sabacc de Kessel** (variante du jeu Star Wars Outlaws),
sous forme d'un workspace Rust multi-crates avec deux frontends :

- **TUI** via Ratatui (terminal)
- **Web** via Svelte + Vite (WebAssembly)

Le cœur du jeu est isolé dans un crate Rust pur sans aucune dépendance UI.

---

## Architecture du workspace

```
kessel-sabacc/
├── Cargo.toml               # workspace root
├── CLAUDE.md                # ce fichier
├── crates/
│   ├── sabacc-core/         # logique de jeu pure, zéro I/O
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── card.rs
│   │       ├── deck.rs
│   │       ├── hand.rs
│   │       ├── player.rs
│   │       ├── round.rs
│   │       ├── turn.rs
│   │       ├── shift_token.rs
│   │       ├── scoring.rs
│   │       └── game.rs
│   ├── sabacc-cli/          # frontend Ratatui
│   │   ├── Cargo.toml       # dépend de sabacc-core, ratatui, crossterm
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs       # état de l'application TUI
│   │       ├── ui.rs        # rendu Ratatui
│   │       └── events.rs    # boucle événementielle
│   └── sabacc-wasm/         # bindings WebAssembly
│       ├── Cargo.toml       # dépend de sabacc-core, wasm-bindgen
│       └── src/
│           └── lib.rs       # fonctions exportées vers JS
└── web/                     # frontend Svelte
    ├── package.json
    ├── vite.config.js
    ├── src/
    │   ├── App.svelte
    │   ├── lib/
    │   │   ├── wasm.js      # initialisation et import du module wasm
    │   │   └── stores.js    # stores Svelte (gameStore, playerStore, etc.)
    │   └── components/
    │       ├── Card.svelte  # composant SVG paramétrique
    │       ├── Hand.svelte
    │       ├── Board.svelte
    │       ├── ShiftToken.svelte
    │       └── Dice.svelte
    └── public/
```

---

## Contraintes absolues sur sabacc-core

`sabacc-core` est le seul endroit où réside la logique de jeu. Ces règles sont
non-négociables :

- **Zéro I/O** : pas de `println!`, `eprintln!`, `std::io`, ni lecture de fichier
- **Zéro dépendance UI** : pas de Ratatui, crossterm, wasm-bindgen, ni web-sys
- **Fonctions pures** : chaque fonction prend un état en entrée et retourne un
  nouvel état ou une erreur — aucun side-effect
- **Pas de `thread_rng` global** : passer la source d'aléatoire en paramètre
  pour faciliter les tests déterministes
- Tous les types publics doivent implémenter `Clone`, `Debug`, `PartialEq`
- Utiliser `thiserror` pour les types d'erreurs

---

## Modèle de données (sabacc-core)

### Cartes

```rust
// Deux familles, une carte appartient à exactement une famille
pub enum Family {
    Sand,
    Blood,
}

// Valeur d'une carte
pub enum CardValue {
    Number(u8),   // 1 à 6
    Sylop,        // prend la valeur de l'autre carte en main
    Impostor,     // valeur déterminée par lancer de dés à la révélation
}

pub struct Card {
    pub family: Family,
    pub value: CardValue,
}
```

Le deck contient :
- 3 cartes par valeur (1-6) par famille → 36 cartes numérotées
- 2 Sylops par famille → 4 Sylops
- 2 Imposteurs par famille → 4 Imposteurs
- **Total : 44 cartes** réparties en 2 paquets (Sand et Blood)

### Main du joueur

Un joueur tient exactement **2 cartes** : une Sand, une Blood.

```rust
pub struct Hand {
    pub sand: Card,
    pub blood: Card,
}
```

### Hiérarchie des mains (du plus fort au plus faible)

1. **Pure Sabacc** — deux Sylops (une paire Sand Sylop + Blood Sylop)
2. **Sylop Sabacc** — un Sylop + n'importe quelle carte numérotée (valeur = 0)
3. **Sabacc** — paire de cartes de même valeur numérique (différence = 0)
   - Départage : la valeur la plus basse l'emporte (1/1 > 6/6)
4. **Main non-Sabacc** — différence absolue entre les deux valeurs
   - Plus la différence est proche de 0, mieux c'est
   - Ex : Sand 6 + Blood 2 → différence = 4

```rust
pub enum HandRank {
    PureSabacc,
    SylopSabacc { value: u8 },
    Sabacc { pair_value: u8 },
    NonSabacc { difference: u8 },
}
```

La résolution d'une `Hand` avec des Imposteurs nécessite la valeur de dés en
paramètre — la fonction de scoring ne lance pas les dés elle-même.

### Joueur

```rust
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub chips: u8,         // jetons restants (réserve)
    pub pot: u8,           // jetons investis dans la manche courante
    pub hand: Option<Hand>,
    pub shift_tokens: Vec<ShiftToken>,
    pub is_eliminated: bool,
}
```

### État du jeu

```rust
pub struct GameState {
    pub players: Vec<Player>,
    pub sand_deck: Vec<Card>,
    pub blood_deck: Vec<Card>,
    pub sand_discard: Vec<Card>,
    pub blood_discard: Vec<Card>,
    pub round: u8,
    pub turn: u8,          // 1, 2 ou 3 dans la manche
    pub current_player_idx: usize,
    pub phase: GamePhase,
    pub credits_in_pot: u32,
}

pub enum GamePhase {
    Setup,
    TurnAction,            // le joueur peut jouer un ShiftToken, puis Draw ou Stand
    ImpostorReveal,        // un ou plusieurs joueurs ont un Imposteur → lancer de dés
    Reveal,                // tous montrent leur main
    Penalty,               // calcul et application des pénalités
    RoundEnd,
    GameOver { winner: PlayerId },
}
```

---

## Règles du jeu (référence complète)

### Mise en place

1. Chaque joueur paie le buy-in (50–200 crédits selon la table)
2. Les crédits vont dans le pot
3. Chaque joueur reçoit 4 à 8 jetons selon la mise (4 pour 50 crédits, 8 pour 200)
4. Le dealer distribue 2 cartes à chaque joueur (une Sand, une Blood)
5. 1 carte Sand et 1 carte Blood sont retournées face visible → défausses initiales

### Tour (Turn)

Chaque manche se compose de **3 tours**. À son tour, un joueur :

1. **Optionnel** : jouer un ShiftToken (avant toute autre action)
2. **Obligatoire** : choisir Draw ou Stand

**Draw** : piocher une carte depuis l'un des 4 emplacements possibles :
- Paquet Sand face cachée
- Paquet Blood face cachée
- Défausse Sand face visible (carte du dessus)
- Défausse Blood face visible (carte du dessus)

Après avoir pioché, le joueur défausse soit la carte piochée, soit la carte
correspondante déjà en main (il conserve toujours exactement 1 Sand + 1 Blood).
**Coût : 1 jeton**.

**Stand** : ne rien faire. Gratuit, mais certains ShiftTokens pénalisent les
joueurs en Stand.

### Révélation

Après le 3e tour, tous les joueurs révèlent leur main dans le sens horaire depuis
le dealer.

Si une main contient un **Imposteur** : le joueur lance les 2 dés Sabacc et
choisit l'une des deux valeurs obtenues pour remplacer l'Imposteur (entre 1 et 6).
Répéter pour chaque Imposteur en main.

### Résultat et pénalités

Le gagnant de la manche récupère l'intégralité de ses jetons investis ce tour.

Les perdants sont taxés :
- **Main Sabacc perdante** → perd **1 jeton**
- **Main non-Sabacc** → perd un nombre de jetons égal à la **différence** de ses
  deux cartes

Les jetons taxés sont détruits (retirés du jeu, pas redistribués).

**Égalité** : tous les joueurs à égalité récupèrent leurs jetons. Les pénalités
s'appliquent normalement aux autres.

Un joueur est **éliminé** quand il n'a plus aucun jeton.

### Fin de partie

Le dernier joueur encore en possession de jetons remporte la partie et les crédits
du pot.

---

## Les 16 ShiftTokens

Chaque ShiftToken ne peut être utilisé **qu'une seule fois par partie** (pas par
manche), avant une action Draw ou Stand.

```rust
pub enum ShiftToken {
    FreeDraw,           // piocher sans payer 1 jeton ce tour
    Refund,             // récupérer 2 jetons investis ce tour (min 1 investi)
    ExtraRefund,        // récupérer 3 jetons investis ce tour
    GeneralTariff,      // tous les autres joueurs paient 1 jeton
    TargetTariff(PlayerId), // un joueur ciblé paie 2 jetons
    Embargo,            // le joueur suivant doit obligatoirement Stand
    Markdown,           // valeur Sylop = 0 jusqu'à la révélation (Sylop ne matche plus)
    Immunity,           // immunité contre les effets de ShiftTokens adverses jusqu'à révélation
    GeneralAudit,       // tous les joueurs en Stand paient 2 jetons
    TargetAudit(PlayerId), // un joueur ciblé en Stand paie 3 jetons
    MajorFraud,         // valeur Imposteur fixée à 6 jusqu'à la révélation
    Embezzlement,       // prendre 1 jeton à chaque autre joueur
    CookTheBooks,       // inverse le classement Sabacc jusqu'à révélation (6/6 devient le meilleur)
    Exhaustion(PlayerId), // le joueur ciblé défausse et repioche une nouvelle main complète
    DirectTransaction(PlayerId), // échanger sa main avec un joueur ciblé
    PrimeSabacc,        // lancer 2 dés, la valeur choisie devient le meilleur Sabacc
}
```

---

## Frontend TUI (sabacc-cli / Ratatui)

> **Consulter Context7 pour `ratatui` et `crossterm` avant d'implémenter.**

### Architecture

Modèle événementiel classique Ratatui :

```
loop {
    terminal.draw(|frame| ui::render(frame, &app))?;
    let event = events::next()?;
    app = app.update(event);
    if app.should_quit { break; }
}
```

`app.rs` contient `AppState` qui encapsule `GameState` + état TUI (curseur,
dialogue actif, animation en cours). `app.update()` est une fonction pure :
`AppState → Event → AppState`.

### Layout V2 — 3 colonnes (min 120×30)

```
┌─ Header ─────────────────────────────────────────────────────────────────────────────────┐
├──────────────────┬────────────────────────────────────────────────────┬───────────────────┤
│ JOUEURS (22ch)   │ TABLE DE JEU (flexible, min 60ch)                 │ LOG (27ch)        │
│                  │   Déf Sand  Deck Sand  Deck Blood  Déf Blood      │ (scrollable)      │
│ ▶ Vous           │   [card]    [card]     [card]      [card]          │                   │
│   ●●●●●○○        │                                                    │                   │
│   5 rés. + 2 pot │ ACTIONS                                           │                   │
│                  │   [Draw] [Stand] [Token (s)]                      │                   │
│   Lando          │                                                    │                   │
│   ●●●○○          │ VOTRE MAIN                                        │                   │
│   3 rés. + 2 pot │   ●●●●●○○    [Sand][Blood]    SHIFT TOKENS       │                   │
│                  │   5r+2p       [card][card]      FreeDraw           │                   │
│                  │                                  — Piocher gratuit │                   │
├──────────────────┴────────────────────────────────────────────────────┴───────────────────┤
```

- Col 1 (JOUEURS) : 3 lignes/joueur (nom, ●○, détail), éliminés compactés si nécessaire
- Col 2 (JEU) : 3 blocs bordés — Tapis (expand, cartes centrées), Actions (fixe), Main (fixe bas, 3 sous-colonnes 1/3 : jetons | cartes | tokens)
- Col 3 (LOG) : scrollable PageUp/PageDown, troncature `…`, auto-scroll
- Pas de mode compact — message d'erreur si terminal < 120×30
- Overlays rendus sur `frame.area()` entier, centrés

Les cartes sont rendues en blocs ASCII 8×5 avec bordures Unicode et couleurs :
- Sand → `Color::Rgb(232, 192, 80)` (ambre chaud)
- Blood → `Color::Rgb(232, 72, 72)` (rouge sang)
- Sylop → `Color::Rgb(144, 144, 224)` (violet)
- Imposteur → `Color::DarkGray`

### Interactions clavier

| Touche      | Action                           |
| ----------- | -------------------------------- |
| `Tab`       | Naviguer entre les actions       |
| `Enter`     | Confirmer l'action sélectionnée  |
| `1`–`4`     | Sélectionner la source de pioche |
| `s`         | Jouer un ShiftToken              |
| `PageUp/Dn` | Scroller le log                 |
| `Space`     | Skip les animations              |
| `q`         | Quitter (avec confirmation)      |
| `?`         | Aide / règles                    |

---

## Frontend Web (web/ · Svelte + Vite + WASM)

> **Consulter Context7 pour `svelte` et `vite` avant d'implémenter.**

### Intégration WASM

`sabacc-wasm` expose via `wasm-bindgen` des fonctions qui prennent et retournent
du JSON sérialisé (via `serde_json`) — pas de types complexes partagés.

```rust
// sabacc-wasm/src/lib.rs — exemple de surface exposée
#[wasm_bindgen]
pub fn new_game(config_json: &str) -> String { ... }

#[wasm_bindgen]
pub fn draw_card(state_json: &str, player_id: u8, source: &str) -> String { ... }

#[wasm_bindgen]
pub fn stand(state_json: &str, player_id: u8) -> String { ... }

#[wasm_bindgen]
pub fn reveal(state_json: &str, impostor_values: &str) -> String { ... }

#[wasm_bindgen]
pub fn play_shift_token(state_json: &str, token: &str, target: Option<u8>) -> String { ... }
```

Chaque fonction retourne un `Result<GameState, GameError>` sérialisé en JSON.
Le frontend n'effectue aucun calcul de règle — il appelle le WASM et met à jour
les stores.

### Stores Svelte

```javascript
// src/lib/stores.js
export const gameState = writable(null);   // GameState désérialisé
export const localPlayer = writable(null); // joueur local (index)
export const ui = writable({
    selectedSource: null,  // source de pioche sélectionnée
    selectedToken: null,   // ShiftToken sélectionné
    phase: 'idle',         // 'idle' | 'draw' | 'token' | 'reveal' | 'dice'
    diceResult: null,
});
```

### Composant Card.svelte

Composant SVG paramétrique — aucun asset externe.

```
Props:
  family: "sand" | "blood" | "sylop" | "impostor"
  value: 1 | 2 | 3 | 4 | 5 | 6 | null   (null = face cachée)
  faceDown: boolean
  selected: boolean
  onClick: () => void

Palette:
  Sand   → fond #1A1208, accent #E8C050, symbole triangle
  Blood  → fond #180808, accent #E84848, symbole losange
  Sylop  → fond #0C0C18, accent #9090E0, symbole double-cercle
  Imposteur → fond #0F0E0D, accent #707070, symbole "?"
```

---

## Ordre de développement recommandé

### Phase 1 — Core (sabacc-core)

1. Consulter Context7 sur les crates utilitaires Rust pertinents
2. Implémenter les types de données (`card.rs`, `hand.rs`, `player.rs`)
3. Implémenter la logique de deck et le mélange (`deck.rs`)
4. Implémenter le scoring et la hiérarchie des mains (`scoring.rs`)
5. Implémenter la machine à états du jeu (`game.rs`, `round.rs`, `turn.rs`)
6. Implémenter les ShiftTokens (`shift_token.rs`)
7. Tests unitaires exhaustifs — couvrir les cas limites :
   - Égalité entre deux mains identiques
   - Imposteur avec valeur de dés donnée en paramètre
   - Sylop avec Markdown actif
   - CookTheBooks inversant le classement
   - Joueur éliminé en cours de manche

### Phase 2 — TUI (sabacc-cli)

1. Consulter Context7 pour `ratatui` (layout, widgets, event loop)
2. Consulter Context7 pour `crossterm` (raw mode, events)
3. Mettre en place la boucle événementielle (`main.rs`, `events.rs`)
4. Implémenter le layout principal (`ui.rs`)
5. Implémenter le rendu des cartes ASCII (`ui.rs`)
6. Connecter les interactions clavier aux actions du core

### Phase 3 — WASM + Web

1. Consulter Context7 pour `wasm-bindgen` et la sérialisation JSON
2. Implémenter `sabacc-wasm` avec `wasm-pack` comme outil de build
3. Consulter Context7 pour `vite` (plugin wasm)
4. Mettre en place le projet Svelte et l'import du module WASM
5. Implémenter les stores et `wasm.js`
6. Implémenter `Card.svelte` (SVG paramétrique)
7. Implémenter les autres composants et le flux de jeu complet

---

## Conventions de code

- Rust edition 2021
- `clippy` sans warnings — utiliser `#[allow(...)]` uniquement si justifié en commentaire
- Pas de `unwrap()` ni `expect()` dans le code de production — propager les erreurs
- Tout type public doit avoir un doc-comment `///`
- Nommage : snake_case Rust, camelCase JS/Svelte, PascalCase composants Svelte
- Commits en anglais, conventionnel : `feat:`, `fix:`, `test:`, `refactor:`

---

## Ce que Claude ne doit PAS faire

- Supposer la syntaxe d'une API Ratatui, wasm-bindgen ou Svelte sans avoir
  consulté Context7 au préalable
- Placer de la logique de règle dans `sabacc-cli` ou `sabacc-wasm`
- Utiliser `unwrap()` en dehors des tests
- Générer des assets graphiques — les visuels sont 100% SVG/code
- Modifier les règles du jeu telles que définies dans ce fichier sans demande
  explicite

---

## Décisions d'implémentation (Phase 1)

### Design patterns retenus

| Sujet | Décision | Raison |
|-------|----------|--------|
| RNG | `rand` 0.8 + `&mut impl Rng` en paramètre | Tests déterministes avec `SmallRng::seed_from_u64` |
| Machine à états | `GamePhase` enum + `apply_action(state, action, rng) -> Result` | Fonctions pures, pas de mutation cachée |
| Draw flow | 2 étapes : `Draw(source)` → `ChoosingDiscard` → `ChooseDiscard` | Permet au frontend de montrer la carte piochée avant le choix |
| HandRank | `strength_key() -> (u8, u8)` explicite, pas de `derive(Ord)` | Contrôle fin du classement, support des modifiers (CookTheBooks) |
| Bot | Trait `BotStrategy` + `BasicBot` | Extensible pour des stratégies plus avancées |
| Versions | `version.workspace = true` dans les sub-crates | Centralise le versioning dans le Cargo.toml racine |
| Modifiers Phase 2 | `ActiveModifiers` struct passée à `evaluate_hand` et `compare_ranks` | Hooks pour ShiftTokens sans changer l'API |

### Fichiers ajoutés à Phase 1 (non dans le plan original)

- `bot.rs` — IA basique incluse dans le core (pas un crate séparé)
- `tests/full_game.rs` — tests d'intégration avec seed fixe

### GamePhase final (diffère légèrement de la spec initiale)

```rust
pub enum GamePhase {
    Setup,
    TurnAction,
    ChoosingDiscard { player_id, drawn_card },  // état intermédiaire après pioche
    ImpostorReveal { pending, submitted },        // avec tracking des choix soumis
    Reveal { results },
    PrimeSabaccChoice { player_id, die1, die2 }, // Phase 2 — choix de dé PrimeSabacc
    RoundEnd,
    GameOver { winner },
}
```

Note : `Penalty` a été fusionné dans `Reveal` → `AdvanceRound` → `RoundEnd`.

---

## Décisions d'implémentation (Phase 2 — ShiftTokens)

### Nouvelles phases et actions

```rust
// Nouveau variant GamePhase
PrimeSabaccChoice { player_id, die1, die2 }

// Nouvelle action
SubmitPrimeSabaccChoice { player_id, chosen_value }
```

### Nouveau HandRank

`PrimeSabacc { value }` inséré entre PureSabacc (0,0) et SylopSabacc (1,x) avec
strength_key `(0, 1)`.

### ActiveModifiers étendu

```rust
pub struct ActiveModifiers {
    pub markdown_active: bool,
    pub cook_the_books_active: bool,
    pub major_fraud_active: bool,           // NEW
    pub immune_players: Vec<PlayerId>,      // NEW
    pub prime_sabacc: Option<PrimeSabaccModifier>, // NEW
}
```

### État par tour sur GameState

```rust
pub stood_this_turn: Vec<PlayerId>,
pub embargoed_player: Option<PlayerId>,
pub token_played_this_turn: bool,
pub free_draw_active: bool,
pub pending_audit: PendingAudit,
```

Ces champs sont reset à chaque nouveau tour (dans `advance_turn`) et nouvelle
manche (dans `apply_advance_round`).

### TokenDistribution

```rust
pub enum TokenDistribution {
    Random { tokens_per_player: usize },
    Fixed(Vec<ShiftToken>),
    None,  // Phase 1 compat
}
```

### Bot IA tokens

Le trait `BotStrategy` a été étendu avec :
- `choose_token()` → ~30% chance par tour, heuristique par type de token
- `choose_prime_sabacc()` → valeur de dé la plus proche des cartes en main
- Targeting stratégique via `most_threatening()` (joueur avec le plus de chips)

### Audit resolution

Les audits sont résolus en fin de **tour** (quand tous les joueurs ont agi),
pas en fin de manche. Le source du GeneralAudit est exclu de son propre effet.

### Fichiers ajoutés à Phase 2

- `tests/shift_token_tests.rs` — 32 tests unitaires et d'intégration

---

## Décisions d'implémentation (Phase 3 — TUI sabacc-cli)

### Architecture TUI (Elm / TEA)

| Sujet | Décision | Raison |
|-------|----------|--------|
| Pattern | `update(state, event) → (state, Command)` pur | Testable, pas de side-effects dans la logique |
| Side-effects | `Command::RunBots` déclenché par le main loop | Sépare les effets (bots) de la logique pure |
| Bots | Boucle manuelle 1 bot à la fois (pas `advance_bots`) | Permet de logger chaque bot individuellement |
| Overlays | Rendus sur `frame.area()` entier, pas dans un sous-pane | Évite le clipping dans les petites zones |
| Log scroll | `log_scroll_offset` (offset depuis le bas), PageUp/PageDown | Convention "offset from bottom" avec auto-scroll |
| Min terminal | 120×30, message d'erreur si trop petit | Pas de mode compact — simplifie le code |

### Layout V2 — Contraintes Ratatui

```rust
// 3 colonnes
Layout::horizontal([Length(22), Min(60), Length(27)])
// Centre : tapis (expand) + actions (fixe) + main (fixe bas)
Layout::vertical([Min(9), Length(5), Length(10)])
// Main : 3 sous-colonnes égales
Layout::horizontal([Ratio(1,3), Ratio(1,3), Ratio(1,3)])
```

### Structure des fichiers sabacc-cli

```
crates/sabacc-cli/src/
├── main.rs          # terminal setup, panic hook, clap args, main loop
├── app.rs           # AppState, TuiState, update(), Command, run_bots()
├── animation.rs     # Animation queue, tick, skip
├── events.rs        # crossterm → AppEvent (Key/Tick/Resize)
├── ui.rs            # render() dispatch, setup screen, 3-col layout, help
└── widgets/
    ├── mod.rs
    ├── card.rs      # CardWidget ASCII 8×5 + inline [S△3]
    ├── header.rs    # Titre + manche/tour/phase
    ├── players.rs   # 3 lignes/joueur, compact éliminés, +N... troncature
    ├── table.rs     # 4 cartes horizontales centrées + labels
    ├── actions.rs   # ActionBar bordée + 7 overlays (source, discard, token, target, dés, quit, gameover)
    ├── hand.rs      # 3 colonnes 1/3 : jetons ●○ | cartes centrées | tokens liste
    └── log.rs       # Scroll PageUp/PageDown, word-wrap, préfixe ›, indicateur ▼ new
```

### Dépendances

```toml
sabacc-core = { path = "../sabacc-core" }
ratatui = { version = "0.29", features = ["crossterm"] }
crossterm = "0.28"
rand = { version = "0.8", features = ["small_rng"] }
clap = { version = "4", features = ["derive"] }
```

### CLI args (clap)

```
sabacc-cli                           # menu interactif
sabacc-cli --quick                   # 3 bots, 100cr, tokens on
sabacc-cli --bots 2 --buy-in 50     # skip menu
sabacc-cli --name "Lando"            # nom custom
sabacc-cli --no-tokens               # désactive ShiftTokens
```

### Tests : 8 tests CLI + 96 tests core

---

## Lessons learned

### Commitizen + Cargo workspaces

Le provider Cargo de `commitizen` (`version_provider = "cargo"`) ne supporte pas
bien les workspaces Rust :
- Il cherche `[package]` dans le `Cargo.toml` racine, pas `[workspace.package]`
- Le `set_lock_version` crash si le workspace n'a pas de `[package].name`
- **Workaround** : utiliser `[workspace.package].version` et faire le bump
  manuellement (modifier le TOML + tag + commit), ou commit gitmoji manuellement
  puis `git tag`

### Deck total_cards après deal_hands

`deal_hands` pioche 1 carte initiale de défausse puis la replace dans la pile de
défausse du même deck → `total_cards()` ne change pas pour cette opération.
Seules les cartes distribuées aux joueurs réduisent le total.

### Clippy et mut

Clippy est strict sur les `mut` inutiles et les imports non utilisés dans les
modules de test. Toujours vérifier clippy après compilation mais avant commit.

### Overlays doivent être rendus sur frame.area() entier

Les overlays (popups) doivent être rendus sur la zone complète du terminal, pas
dans un sous-pane comme l'action bar. Sinon ils sont clippés à la taille du pane
(3 lignes) et invisibles. Bug découvert dès la première session de test.

### advance_bots() joue TOUS les bots d'un coup

La fonction `advance_bots()` du core boucle en interne jusqu'au tour d'un humain.
Si on veut logger chaque bot individuellement, il faut boucler manuellement avec
`BotStrategy::choose_action()` + `apply_action()` un bot à la fois.

### GameState est consommé par apply_action()

`apply_action(state, action, rng)` consomme `state`. Si l'action échoue, l'état
est perdu. Solution : `.take()` + `.clone()` avant si on veut pouvoir retry,
ou accepter la perte et logguer l'erreur.

### rand small_rng feature

`SmallRng` nécessite la feature `small_rng` dans `rand = { version = "0.8",
features = ["small_rng"] }`. Sans elle, l'import compile pas.

### Layout TUI : centrer les éléments dans les panes expansibles

Quand un pane utilise `Min(N)` et prend tout l'espace disponible, les éléments à
l'intérieur (cartes, texte) doivent être centrés horizontalement et verticalement
avec des offsets calculés : `offset = (available - content) / 2`.

### Bordures Ratatui : compter +2 pour la hauteur

Quand on utilise `Block::default().borders(Borders::ALL)`, l'inner area perd 2
lignes (top+bottom) et 2 colonnes. Ajuster les `Constraint::Length()` en
conséquence.

### Tokens overflow : mode compact adaptatif

Si les shift tokens ne tiennent pas en 2 lignes/token (nom + desc), basculer
automatiquement en 1 ligne/token (nom — desc sur la même ligne).

### Messages de log concis pour colonne étroite

La colonne log fait ~23 chars utiles. Les messages doivent être abrégés :
"Draw Deck S" au lieu de "pioche depuis Deck Sand". Le log fait du word-wrap
avec préfixe `›` pour le premier segment et indentation `  ` pour les suivants.

### ImpostorReveal / PrimeSabaccChoice : ne PAS utiliser is_human_turn()

`is_human_turn()` vérifie `current_player_idx` (le tour de jeu), pas qui a un
imposteur ni qui doit choisir pour PrimeSabacc. Ces phases ont leur propre logique :
- `ImpostorReveal` → vérifier si `pending.contains(&human_id)`
- `PrimeSabaccChoice` → vérifier si `player_id == 0`
Si la condition est fausse → `Command::RunBots` pour que les bots résolvent.
Le tick handler doit aussi relancer les bots pour ces phases, pas seulement
`TurnAction`. Bug de freeze si oublié.

### Tous les overlays doivent supporter ↑↓ en plus de Tab

Les overlays de sélection (dés, sources, tokens) doivent accepter `KeyCode::Up`
et `KeyCode::Down` en plus de `Tab`/`Left`/`Right`. L'utilisateur s'attend à ce
que les flèches fonctionnent partout.

### Action bar : afficher un message pour CHAQUE GamePhase

Le `render_bar` doit avoir un cas explicite pour chaque `GamePhase` :
`TurnAction`, `Reveal`, `RoundEnd`, `GameOver`, `ImpostorReveal`,
`PrimeSabaccChoice`, `ChoosingDiscard`. Si un variant tombe dans `_ => {}`,
l'utilisateur voit une action bar vide et pense que le jeu est figé.

### Token descriptions dupliquées : actions.rs (long) et hand.rs (short)

Les descriptions de ShiftTokens existent en **2 copies** :
- `widgets/actions.rs::token_description()` → version longue pour le popup overlay
- `widgets/hand.rs::token_description()` → version courte pour la sidebar (colonne étroite)

Lors de toute modification de texte de token, penser à mettre à jour les deux.

### Labels de table : 8 chars max

Les labels sous les cartes de la table ("Dis Sand", "Deck(N)", "Dis Blood") sont
limités à **8 caractères** par la largeur des colonnes de cartes. Utiliser des
abréviations : "Dis" (Discard), "Deck" (Draw pile).

### UI language : English

Depuis v0.5.0, toute l'interface du jeu est en anglais. Les messages de log, les
overlays, les labels, les descriptions de tokens, l'écran de setup et l'aide sont
en anglais. Maintenir cette cohérence pour tout nouveau texte ajouté.

### is_animating() doit couvrir TOUTES les animations visuelles

`AppState::is_animating()` contrôle si le main loop génère des ticks (33ms) ou
bloque en attente d'input. Si une animation visuelle (overlay reveal, etc.) n'est
pas couverte par `is_animating()`, les ticks cessent et l'animation gèle.
Actuellement couvre : `AnimationQueue` + `RoundResults` reveal. Tout nouvel effet
animé doit être ajouté à cette vérification.

### Double impostor : overlay en deux étapes

Quand un joueur a Sand+Blood impostors, l'overlay `ImpostorChoice` doit proposer
deux choix de dés consécutifs (Sand puis Blood) avant de soumettre un seul
`ImpostorChoice` au core avec `sand_choice` ET `blood_choice` remplis. Le champ
`has_blood_impostor` sur l'overlay contrôle ce flow en deux étapes.

### RoundResults chips_before : réserve seule, pas réserve+pot

`chips_before` dans `RoundResultDisplay` doit être `p.chips` (réserve seule), pas
`p.chips + p.pot`. Le pot est "investi et à risque". `chips_after` = réserve + pot
retourné (winner) ou réserve - pénalité (loser). Cela montre la transition visible
du pot qui revient dans la réserve du gagnant.

### Log : word-wrap + préfixe `›` pour distinguer les messages

Pour un log dans une colonne étroite, ne pas tronquer — wrapper intelligemment.
Utiliser un symbole (`›`) en début de chaque nouveau message et indenter les
lignes de continuation. Le split se fait aux espaces (ou hard-break si un mot
est trop long). Compter les **chars** pas les **bytes** pour les largeurs
(UTF-8 multi-bytes comme `é`, `→`, `●`).

---

## Flow de push (référence rapide)

Workflow standard pour une feature :

```bash
# 1. Créer un worktree isolé
git worktree add .worktrees/<feature-name> -b feat/<feature-name>
cd .worktrees/<feature-name>

# 2. Développer + tester
cargo test -p <crate>
cargo clippy -p <crate> -- -D warnings

# 3. Commit dans le worktree (format gitmoji)
git add <fichiers>
git commit -m "✨ feat: description courte

Description longue si nécessaire.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"

# 4. Retour sur main + merge
cd <project-root>
git checkout main
git merge feat/<feature-name>

# 5. Bump version
# Option A — cz bump (si ça marche avec le workspace)
cz bump --yes

# Option B — manuel (si cz crashe)
# Modifier [workspace.package].version dans Cargo.toml racine
# cargo generate-lockfile
# git add Cargo.toml Cargo.lock crates/*/Cargo.toml
# git commit -m "🔖 bump: version X.Y.Z → A.B.C"
# git tag A.B.C

# 6. Push avec tags
git push --follow-tags

# 7. Cleanup
git worktree remove .worktrees/<feature-name>  # --force si fichiers non trackés
git branch -d feat/<feature-name>
```

### Convention de commits (gitmoji + conventional)

| Emoji | Type | Usage |
|-------|------|-------|
| ✨ | feat | Nouvelle fonctionnalité |
| 🐛 | fix | Correction de bug |
| ♻️ | refactor | Refactoring sans changement fonctionnel |
| ✅ | test | Ajout/modification de tests |
| 🔧 | chore | Configuration, tooling |
| 🔖 | bump | Changement de version |
| 🎉 | init | Commit initial |

<!-- rtk-instructions v2 -->
# RTK (Rust Token Killer) - Token-Optimized Commands

## Golden Rule

**Always prefix commands with `rtk`**. If RTK has a dedicated filter, it uses it. If not, it passes through unchanged. This means RTK is always safe to use.

**Important**: Even in command chains with `&&`, use `rtk`:
```bash
# ❌ Wrong
git add . && git commit -m "msg" && git push

# ✅ Correct
rtk git add . && rtk git commit -m "msg" && rtk git push
```

## RTK Commands by Workflow

### Build & Compile (80-90% savings)
```bash
rtk cargo build         # Cargo build output
rtk cargo check         # Cargo check output
rtk cargo clippy        # Clippy warnings grouped by file (80%)
rtk tsc                 # TypeScript errors grouped by file/code (83%)
rtk lint                # ESLint/Biome violations grouped (84%)
rtk prettier --check    # Files needing format only (70%)
rtk next build          # Next.js build with route metrics (87%)
```

### Test (90-99% savings)
```bash
rtk cargo test          # Cargo test failures only (90%)
rtk vitest run          # Vitest failures only (99.5%)
rtk playwright test     # Playwright failures only (94%)
rtk test <cmd>          # Generic test wrapper - failures only
```

### Git (59-80% savings)
```bash
rtk git status          # Compact status
rtk git log             # Compact log (works with all git flags)
rtk git diff            # Compact diff (80%)
rtk git show            # Compact show (80%)
rtk git add             # Ultra-compact confirmations (59%)
rtk git commit          # Ultra-compact confirmations (59%)
rtk git push            # Ultra-compact confirmations
rtk git pull            # Ultra-compact confirmations
rtk git branch          # Compact branch list
rtk git fetch           # Compact fetch
rtk git stash           # Compact stash
rtk git worktree        # Compact worktree
```

Note: Git passthrough works for ALL subcommands, even those not explicitly listed.

### GitHub (26-87% savings)
```bash
rtk gh pr view <num>    # Compact PR view (87%)
rtk gh pr checks        # Compact PR checks (79%)
rtk gh run list         # Compact workflow runs (82%)
rtk gh issue list       # Compact issue list (80%)
rtk gh api              # Compact API responses (26%)
```

### JavaScript/TypeScript Tooling (70-90% savings)
```bash
rtk pnpm list           # Compact dependency tree (70%)
rtk pnpm outdated       # Compact outdated packages (80%)
rtk pnpm install        # Compact install output (90%)
rtk npm run <script>    # Compact npm script output
rtk npx <cmd>           # Compact npx command output
rtk prisma              # Prisma without ASCII art (88%)
```

### Files & Search (60-75% savings)
```bash
rtk ls <path>           # Tree format, compact (65%)
rtk read <file>         # Code reading with filtering (60%)
rtk grep <pattern>      # Search grouped by file (75%)
rtk find <pattern>      # Find grouped by directory (70%)
```

### Analysis & Debug (70-90% savings)
```bash
rtk err <cmd>           # Filter errors only from any command
rtk log <file>          # Deduplicated logs with counts
rtk json <file>         # JSON structure without values
rtk deps                # Dependency overview
rtk env                 # Environment variables compact
rtk summary <cmd>       # Smart summary of command output
rtk diff                # Ultra-compact diffs
```

### Infrastructure (85% savings)
```bash
rtk docker ps           # Compact container list
rtk docker images       # Compact image list
rtk docker logs <c>     # Deduplicated logs
rtk kubectl get         # Compact resource list
rtk kubectl logs        # Deduplicated pod logs
```

### Network (65-70% savings)
```bash
rtk curl <url>          # Compact HTTP responses (70%)
rtk wget <url>          # Compact download output (65%)
```

### Meta Commands
```bash
rtk gain                # View token savings statistics
rtk gain --history      # View command history with savings
rtk discover            # Analyze Claude Code sessions for missed RTK usage
rtk proxy <cmd>         # Run command without filtering (for debugging)
rtk init                # Add RTK instructions to CLAUDE.md
rtk init --global       # Add RTK to ~/.claude/CLAUDE.md
```

## Token Savings Overview

| Category | Commands | Typical Savings |
|----------|----------|-----------------|
| Tests | vitest, playwright, cargo test | 90-99% |
| Build | next, tsc, lint, prettier | 70-87% |
| Git | status, log, diff, add, commit | 59-80% |
| GitHub | gh pr, gh run, gh issue | 26-87% |
| Package Managers | pnpm, npm, npx | 70-90% |
| Files | ls, read, grep, find | 60-75% |
| Infrastructure | docker, kubectl | 85% |
| Network | curl, wget | 65-70% |

Overall average: **60-90% token reduction** on common development operations.
<!-- /rtk-instructions -->