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
- `web/` → résoudre `react`, `react-router`, `vite`, `zustand`, `tailwindcss`

Exemple de workflow attendu :
1. Appel Context7 `resolve-library-id` pour chaque librairie concernée
2. Appel Context7 `query-docs` sur les points précis à implémenter
3. Seulement ensuite : écrire le code

---

## Vue d'ensemble du projet

Reproduction jouable du **Sabacc de Kessel** (variante du jeu Star Wars Outlaws),
sous forme d'un workspace Rust multi-crates avec deux frontends :

- **TUI** via Ratatui (terminal)
- **Web** via React + Vite + 8bitcn (WebAssembly)

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
└── web/                     # frontend React + 8bitcn
    ├── .nvmrc               # Node LTS version (22)
    ├── package.json          # pnpm, scripts build:wasm/dev/build
    ├── vite.config.ts
    ├── tsconfig.json
    ├── components.json       # shadcn + 8bitcn registry
    ├── src/
    │   ├── main.tsx
    │   ├── App.tsx           # HashRouter + Routes
    │   ├── globals.css       # Tailwind v4 + 8-bit theme
    │   ├── lib/
    │   │   ├── wasm.ts       # chargement async du module WASM
    │   │   ├── game-store.ts # store zustand (état de jeu)
    │   │   └── utils.ts      # cn() helper (tailwind-merge)
    │   ├── components/
    │   │   ├── Layout.tsx    # layout partagé + starfield + WASM preload
    │   │   ├── Starfield.tsx # animation canvas 2D (port du TUI)
    │   │   └── ui/           # composants shadcn + 8bitcn
    │   └── pages/
    │       ├── MainMenu.tsx  # menu ASCII art + items
    │       ├── HowToPlay.tsx # règles scrollables
    │       ├── Setup.tsx     # formulaire config
    │       └── Game.tsx      # WASM + placeholder board
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
pub enum Family { Sand, Blood }

pub enum CardValue {
    Number(u8),   // 1 à 6
    Sylop,        // prend la valeur de l'autre carte en main
    Impostor,     // valeur déterminée par lancer de dés à la révélation
}

pub struct Card { pub family: Family, pub value: CardValue }
```

Le deck contient 44 cartes (2 paquets Sand/Blood) :
- 3 cartes × 6 valeurs × 2 familles = 36 numérotées
- 2 Sylops × 2 familles = 4 Sylops
- 2 Imposteurs × 2 familles = 4 Imposteurs

### Main du joueur

Exactement **2 cartes** : une Sand, une Blood.

```rust
pub struct Hand { pub sand: Card, pub blood: Card }
```

### Hiérarchie des mains (du plus fort au plus faible)

1. **Pure Sabacc** — deux Sylops
2. **Prime Sabacc** — via ShiftToken PrimeSabacc (entre Pure et Sylop)
3. **Sylop Sabacc** — un Sylop + numérotée (valeur = 0)
4. **Sabacc** — paire de même valeur (départage : plus basse gagne)
5. **Non-Sabacc** — différence absolue (plus proche de 0, mieux c'est)

```rust
pub enum HandRank {
    PureSabacc,
    PrimeSabacc { value: u8 },
    SylopSabacc { value: u8 },
    Sabacc { pair_value: u8 },
    NonSabacc { difference: u8 },
}
```

### Joueur

```rust
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub chips: u8,         // réserve
    pub pot: u8,           // investis dans la manche
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
    pub turn: u8,          // 1, 2 ou 3
    pub current_player_idx: usize,
    pub phase: GamePhase,
    pub credits_in_pot: u32,
}
```

---

## Règles du jeu

### Mise en place

1. Buy-in (50–200 crédits) → pot
2. 4 à 8 jetons selon la mise
3. 2 cartes par joueur (1 Sand + 1 Blood)
4. 1 carte Sand + 1 Blood en défausse face visible

### Tour (3 tours par manche)

1. **Optionnel** : jouer un ShiftToken
2. **Obligatoire** : Draw (piocher depuis 4 sources, coût 1 jeton, défausser
   piochée ou en main) ou Stand (gratuit)

### Révélation

Après le 3e tour. Imposteur → lancer 2 dés, choisir une valeur.

### Pénalités

- Gagnant récupère ses jetons investis
- Perdant Sabacc → perd 1 jeton
- Perdant non-Sabacc → perd la différence en jetons
- Égalité → tous à égalité récupèrent leurs jetons
- 0 jetons → éliminé. Dernier debout gagne le pot.

---

## Les 16 ShiftTokens

Usage unique par partie, avant Draw/Stand.

```rust
pub enum ShiftToken {
    FreeDraw,                       // piocher sans payer
    Refund,                         // récupérer 2 jetons investis
    ExtraRefund,                    // récupérer 3 jetons investis
    GeneralTariff,                  // tous paient 1 jeton
    TargetTariff(PlayerId),         // ciblé paie 2 jetons
    Embargo,                        // suivant doit Stand
    Markdown,                       // Sylop = 0 (ne matche plus)
    Immunity,                       // immunité ShiftTokens adverses
    GeneralAudit,                   // Stand = paient 2 jetons
    TargetAudit(PlayerId),          // ciblé en Stand paie 3 jetons
    MajorFraud,                     // Imposteur fixé à 6
    Embezzlement,                   // prendre 1 jeton à chacun
    CookTheBooks,                   // inverse le classement
    Exhaustion(PlayerId),           // ciblé repioche une nouvelle main
    DirectTransaction(PlayerId),    // échanger sa main
    PrimeSabacc,                    // lancer 2 dés → meilleur Sabacc
}
```

---

## Conventions de code

- Rust edition 2021
- `clippy` sans warnings — `#[allow(...)]` uniquement si justifié
- Pas de `unwrap()` ni `expect()` en production — propager les erreurs
- Tout type public : doc-comment `///`
- Nommage : snake_case Rust, camelCase TS/JS, PascalCase composants React
- Package manager : **pnpm** uniquement (pas npm, pas yarn)
- Node version : via **nvm** — toujours `nvm use` avant toute commande dans `web/`
- Pas de `any` en TypeScript sauf nécessité justifiée
- Préférer les function components avec hooks
- Commits en anglais, gitmoji + conventionnel : `✨ feat:`, `🐛 fix:`, etc.

---

## Ce que Claude ne doit PAS faire

- Supposer la syntaxe d'une API sans consulter Context7
- Placer de la logique de règle dans `sabacc-cli` ou `sabacc-wasm`
- Utiliser `unwrap()` en dehors des tests
- Générer des assets graphiques — visuels 100% SVG/code
- Modifier les règles du jeu sans demande explicite

---

## Détails externalisés (mémoires)

Les specs détaillées, décisions d'implémentation et lessons learned sont dans
les fichiers mémoire `ref_*`. Consulter selon le besoin :

- **TUI spec** → `ref_tui_spec.md` (layout, keys, colors, file structure)
- **Web spec** → `ref_web_spec.md` (WASM API, Svelte stores, Card.svelte)
- **Décisions core** → `ref_impl_decisions.md` (design patterns, GamePhase, modifiers)
- **Lessons learned** → `ref_lessons_learned.md` (gotchas, rendering, animations)
- **Push flow** → `ref_push_flow.md` (worktree workflow, commit conventions)

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
rtk git git show            # Compact show (80%)
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
