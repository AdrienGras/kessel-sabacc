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

### Layout recommandé

```
┌─────────────────────────────────────────────────────────────┐
│  KESSEL SABACC              Manche 2 · Tour 1/3             │
├──────────────────┬──────────────────┬───────────────────────┤
│  Joueur 1  ●●●●  │  Joueur 2  ●●○○  │  Joueur 3  ●○○○       │
│  [SAND: 3][BLD:?]│  [SAND: 1][BLD:1]│  [SAND:5][BLD:2]      │
├──────────────────┴──────────────────┴───────────────────────┤
│                    Table                                     │
│          [SAND ▲]   [BLOOD ▲]   ← paquets                  │
│          [SAND: 4]  [BLOOD: 2]  ← défausses                 │
├─────────────────────────────────────────────────────────────┤
│  Votre main : [SAND: 3] [BLOOD: ?]   Jetons: ●●●○           │
│  ShiftTokens: [FreeDraw] [Immunity]                          │
│  > [DRAW] [STAND]                                           │
└─────────────────────────────────────────────────────────────┘
│  Log: Joueur 2 a pioché une carte Blood.                     │
└─────────────────────────────────────────────────────────────┘
```

Les cartes sont rendues en blocs ASCII avec bordures Unicode et couleurs :
- Sand → `Color::Rgb(232, 192, 80)` (ambre chaud)
- Blood → `Color::Rgb(232, 72, 72)` (rouge sang)
- Sylop → `Color::Rgb(144, 144, 224)` (violet)
- Imposteur → `Color::DarkGray`

### Interactions clavier

| Touche  | Action                           |
| ------- | -------------------------------- |
| `Tab`   | Naviguer entre les actions       |
| `Enter` | Confirmer l'action sélectionnée  |
| `1`–`4` | Sélectionner la source de pioche |
| `s`     | Jouer un ShiftToken              |
| `q`     | Quitter                          |
| `?`     | Aide / règles                    |

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