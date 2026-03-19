# Contributing to Kessel Sabacc

Thanks for your interest in contributing! Whether you're fixing a bug, adding a feature, or improving docs, your help is welcome.

## Prerequisites

- Rust (edition 2021)
- cargo

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch (`git checkout -b feat/my-feature`)

## Build & Test

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```

## Architecture

The project is a Rust workspace with multiple crates:

- **sabacc-core**: Pure game logic — zero I/O, no side effects, no `unwrap()` in production code
- **sabacc-cli**: Terminal UI frontend built with Ratatui
- **sabacc-wasm**: WebAssembly bindings (planned)

All game rules live in `sabacc-core`. Never place game logic in the frontend crates.

## Code Style

- `clippy` must pass with no warnings (`#[allow(...)]` only if justified with a comment)
- All public types must implement `Clone`, `Debug`, `PartialEq`
- All public types must have `///` doc-comments
- Errors via `thiserror` — no `unwrap()` or `expect()` outside tests
- Pass RNG as parameter (no global `thread_rng`) for deterministic tests

## Commit Conventions

We use gitmoji + conventional commits in English:

- `✨ feat:` — new feature
- `🐛 fix:` — bug fix
- `♻️ refactor:` — refactoring
- `📝 docs:` — documentation
- `🧪 test:` — tests
- `🔖 bump:` — version bump

## Pull Requests

- Keep PRs focused — one feature or fix per PR
- Include tests for new functionality
- Ensure `cargo test` and `cargo clippy` pass
- Describe what and why in the PR description

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
