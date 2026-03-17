# Spec: sabacc-cli — Layout V2

**Date:** 2026-03-17
**Status:** Draft
**Crate:** `crates/sabacc-cli`
**Supersedes:** La section "Layout responsive" et "Layout recommandé" de `2026-03-17-tui-design.md`. Les autres sections de la spec originale restent valides (setup, overlays, animations, phase mapping, navigation clavier hors PageUp/PageDown, gestion d'erreurs, tests).

---

## Contexte

Le layout actuel du TUI a plusieurs problèmes d'ergonomie : la main du joueur et les infos de jeu sont écrasées dans une petite colonne à droite, les shift tokens sont illisibles, le tapis n'est pas clair (2 rangées au lieu d'un alignement horizontal), et le log en bas ne prend que 6 lignes fixes. Ce redesign réorganise l'interface en 3 colonnes spécialisées.

## Portée

- Refonte du layout de l'écran de jeu (`Screen::Playing`)
- Pas de changement sur l'écran de setup
- Pas de mode compact — minimum 120 colonnes, 30 lignes requis
- Le mode compact est supprimé (`render_main_compact` retiré)

---

## Layout : 3 colonnes

### Structure globale

```
┌─ Header ──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
├───────────────────┬───────────────────────────────────────────────────────────────────────────┬────────────────────┤
│ Col 1 — JOUEURS   │ Col 2 — JEU                                                              │ Col 3 — LOG        │
│ (~20 chars)       │ (flexible, min ~60 chars)                                                 │ (~25 chars)        │
│                   │                                                                           │                    │
│                   │ ┌─ TAPIS ─────────────────────────────────────────────────────────────┐   │                    │
│                   │ │  Déf Sand   Deck Sand    Deck Blood   Déf Blood                    │   │                    │
│                   │ └────────────────────────────────────────────────────────────────────┘   │                    │
│                   │                                                                           │                    │
│                   │ ┌─ ACTIONS ──────────────────────────────────────────────────────────┐   │                    │
│                   │ │  [Draw] [Stand] [Token (s)]                                        │   │                    │
│                   │ └────────────────────────────────────────────────────────────────────┘   │                    │
│                   │                                                                           │                    │
│                   │ ┌─ VOTRE MAIN ───────────────────────────────────────────────────────┐   │                    │
│                   │ │  Cartes + Jetons ●○ + Shift Tokens (liste verticale)               │   │                    │
│                   │ └────────────────────────────────────────────────────────────────────┘   │                    │
├───────────────────┴───────────────────────────────────────────────────────────────────────────┴────────────────────┤
```

### Taille minimale

Si le terminal fait moins de 120 colonnes ou moins de 30 lignes, afficher un message centré :

```
Terminal trop petit (XXx×YY)
Minimum requis : 120×30
```

Pas de mode compact. Pas de dégradation gracieuse. L'utilisateur doit agrandir son terminal.

Le minimum de 30 lignes garantit l'affichage de 7 joueurs (7 × 4 = 28 lignes, dans 28 lignes de hauteur utile) et de la zone de jeu complète.

---

## Col 1 — Joueurs (22 chars)

Chaque joueur occupe 3 lignes + 1 ligne vide de séparation.

Si la hauteur disponible ne suffit pas pour afficher tous les joueurs en format 3 lignes (ex : 8 joueurs = 31 lignes), les joueurs éliminés sont réduits à 1 ligne (nom barré uniquement) et la ligne vide de séparation est supprimée entre eux. Si ça ne suffit toujours pas, les joueurs sont tronqués en bas avec un indicateur `+N...`.

```
▶ Vous
  ●●●●●○○
  5 rés. + 2 pot

  Lando
  ●●●○○
  3 rés. + 2 pot

  Han
  ●●●●○
  4 rés. + 1 pot

  Chewie (éliminé)
  ●
  1 rés. + 0 pot
```

### Styles

| État | Nom | Jetons |
|------|-----|--------|
| Tour actif | `▶` préfixe, Yellow + Bold | `Rgb(200, 200, 100)` |
| Normal | White | `Rgb(200, 200, 100)` |
| Éliminé | DarkGray + barré | DarkGray |
| Highlight (animation) | Yellow | — |

- `●` = jetons en réserve
- `○` = jetons investis (pot)
- Ligne 3 : texte détaillé `X rés. + Y pot` en DarkGray

---

## Col 2 — Jeu (flexible, min 60 chars)

Trois blocs empilés verticalement.

### Bloc 1 : Tapis

4 emplacements alignés horizontalement dans l'ordre, sans bordure de bloc (pas de `Borders::ALL`, juste le titre en haut) :

```
TABLE DE JEU
  ┌──────┐  ┌──────┐    ┌──────┐  ┌──────┐
  │SAND  │  │ ???? │    │ ???? │  │BLOOD │
  │  △   │  │  ▓▓  │    │  ▓▓  │  │  ◇   │
  │  4   │  │  ▓▓  │    │  ▓▓  │  │  2   │
  └──────┘  └──────┘    └──────┘  └──────┘
  Déf Sand  Deck(19)    Deck(17)  Déf Blood
```

Hauteur totale : 7 lignes (1 titre + 5 carte + 1 label).

- Défausses : carte face visible (CardWidget 8×5) ou "vide" si pile vide
- Decks : CardWidget face cachée (▓▓)
- Labels sous chaque carte : nom + count pour les decks
- Ordre logique : Sand à gauche, Blood à droite, défausses aux extrémités
- Espacement : 2 chars entre cartes Sand/Sand et Blood/Blood, 4 chars entre Deck Sand et Deck Blood (séparation visuelle familles)

### Bloc 2 : Actions

Identique à l'implémentation actuelle :

```
ACTIONS
 [Draw]  [Stand]  [Token (s)]
 Tab: naviguer · Enter: confirmer
```

- Bouton sélectionné : fond Yellow, texte Black, bold
- Hint en DarkGray sous les boutons
- Adapté à la phase (`TurnAction` humain, attente bots, Reveal/RoundEnd, GameOver)

### Bloc 3 : Infos joueur

Divisé en deux zones côte à côte : cartes à gauche, infos à droite.

```
VOTRE MAIN
┌──────┐  ┌──────┐      ●●●●●○○
│SAND  │  │BLOOD │      5 réserve + 2 investis
│  △   │  │  ◇   │
│  3   │  │  5   │      SHIFT TOKENS
└──────┘  └──────┘      FreeDraw
                          Piocher sans payer
                        Immunity
                          Immunité aux tokens
                        GeneralTariff
                          Tous paient 1 jeton
```

- **Cartes** : 2 CardWidgets pleine taille (8×5) côte à côte, `8 + 2 + 8 = 18` chars de large
- **Jetons** : `●●●●●○○` en `Rgb(200, 200, 100)`, même format que colonne joueurs
- **Détail** : `X réserve + Y investis` en DarkGray
- **Shift Tokens** : liste verticale, chaque token sur 2 lignes :
  - Ligne 1 : nom du token en Cyan
  - Ligne 2 : description courte en DarkGray, indentée de 2 espaces
  - Tronqué à la hauteur disponible si le joueur a beaucoup de tokens (max ~5 visibles, suffisant en pratique car 4 tokens par joueur par défaut)
- Si aucun token : afficher `(aucun)` en DarkGray

---

## Col 3 — Log (27 chars, ~23 utiles après bordures)

### Affichage

- Titre `LOG` en haut
- Bordure `Borders::ALL`
- Chaque entrée sur une ligne, tronquée avec `…` en fin de ligne si trop longue
- Les messages de jeu doivent être concis pour tenir (~23 chars) : utiliser des abréviations (ex : "Lando: Draw Deck S" au lieu de "Lando: pioche depuis Deck Sand")
- Auto-scroll vers le bas par défaut

### Scroll

- **PageUp** : remonter d'une page (hauteur visible)
- **PageDown** : descendre d'une page
- Quand l'utilisateur scrolle manuellement, désactiver l'auto-scroll
- Revenir en bas (PageDown quand déjà en bas, ou nouvelle entrée) réactive l'auto-scroll
- Indicateur visuel optionnel : `▼ nouvelles entrées` si scrollé vers le haut et nouvelles entrées ajoutées

### Styles

- Entrées normales : DarkGray
- Entrées erreur : Red

---

## Header

Identique à l'implémentation actuelle, pleine largeur :

```
 KESSEL SABACC   Manche 2 · Tour 1/3  │  Action
─────────────────────────────────────────────────
```

- Titre : `Rgb(232, 192, 80)` Bold
- Round/Turn : White
- Phase : Cyan
- Bordure basse : DarkGray

---

## Overlays

Les overlays (SourcePicker, DiscardChoice, TokenPicker, TargetPicker, ImpostorChoice, PrimeSabaccChoice, QuitConfirm, GameOver) sont rendus sur `frame.area()` entier, centrés, par-dessus le layout. Pas de changement par rapport à l'implémentation actuelle.

---

## Contraintes Ratatui

### Layout principal

```rust
// Vertical: header + main
let main_layout = Layout::vertical([
    Constraint::Length(2),   // header
    Constraint::Min(20),     // main content
]).split(frame.area());

// Horizontal: 3 colonnes
let cols = Layout::horizontal([
    Constraint::Length(22),   // joueurs
    Constraint::Min(60),      // jeu (centre)
    Constraint::Length(27),   // log
]).split(main_layout[1]);
```

### Colonne jeu (centre)

```rust
// Vertical: tapis + actions + infos joueur
let center = Layout::vertical([
    Constraint::Length(7),    // tapis (1 titre + 5 carte + 1 label, pas de bordure bloc)
    Constraint::Length(3),    // actions
    Constraint::Min(10),      // infos joueur (cartes + tokens)
]).split(cols[1]);
```

### Bloc infos joueur

```rust
// Horizontal: cartes à gauche, infos à droite
let hand_area = Layout::horizontal([
    Constraint::Length(20),   // 2 cartes (8+2+8) + 2 marge
    Constraint::Min(20),      // jetons + tokens
]).split(center[2]);
```

---

## Scroll state

Ajouter à `TuiState` :

```rust
pub log_scroll_offset: usize,  // 0 = en bas (auto-scroll)
pub log_auto_scroll: bool,     // true par défaut
```

### Comportement

`log_scroll_offset` est un offset depuis le bas (0 = dernières entrées visibles).

- `PageUp` : `log_scroll_offset += visible_height`, clamp à `max(0, total - visible)`, `log_auto_scroll = false`
- `PageDown` : `log_scroll_offset = log_scroll_offset.saturating_sub(visible_height)`, si `offset == 0` → `log_auto_scroll = true`
- Nouvelle entrée de log + `log_auto_scroll == true` : `log_scroll_offset = 0`
- Nouvelle entrée de log + `log_auto_scroll == false` : `log_scroll_offset += 1` (pour rester à la même position visuelle)

### Formule de rendu

```rust
let visible = inner.height as usize;
let total = app.log.len();
let start = total.saturating_sub(visible + log_scroll_offset);
let end = start + visible.min(total - start);
// Render app.log[start..end]
```

---

## Navigation clavier (récapitulatif)

| Touche | Contexte | Action |
|--------|----------|--------|
| `Tab` / `Shift+Tab` | ActionBar, overlays | Naviguer entre options |
| `Enter` | Partout | Confirmer la sélection |
| `1`-`4` | SourcePicker | Sélection directe |
| `s` | TurnAction | Ouvrir TokenPicker |
| `Esc` | Overlay ouvert | Fermer / annuler |
| `q` | Partout | Quitter (avec confirmation) |
| `?` | Partout | Aide |
| `Space` | Animation en cours | Skip animations |
| `PageUp` | Jeu | Remonter le log |
| `PageDown` | Jeu | Descendre le log |

---

## Fichiers impactés

| Fichier | Changement |
|---------|-----------|
| `src/ui.rs` | Refonte layout : 3 colonnes, minimum 120×24, suppression mode compact |
| `src/widgets/players.rs` | Format 3 lignes par joueur (nom / ●○ / détail) |
| `src/widgets/table.rs` | 4 cartes alignées horizontalement, suppression mode compact |
| `src/widgets/hand.rs` | Cartes + jetons ●○ + tokens en liste verticale avec descriptions |
| `src/widgets/log.rs` | Scroll PageUp/PageDown, auto-scroll toggle |
| `src/app.rs` | Ajouter `log_scroll_offset`, `log_auto_scroll` à TuiState, gérer PageUp/PageDown |
| `src/widgets/actions.rs` | Pas de changement majeur (déjà séparé bar/overlay) |
| `src/widgets/header.rs` | Pas de changement |
