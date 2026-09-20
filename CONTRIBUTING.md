# Contributing to Pausetta

First off, thank you for considering contributing to Pausetta!

Whether you're fixing a bug, improving documentation, suggesting a feature, or submitting code, every contribution helps make the project better.

Please take a few minutes to read this guide before opening an issue or pull request.


# Code of Conduct

By participating in this project, you agree to follow our Code of Conduct.

Please read:

**CODE_OF_CONDUCT.md**

Be respectful, constructive, and welcoming to everyone.


## Getting Started

1. Fork the repository and clone your fork.
2. Install dependencies: `pnpm install`
3. Run in dev mode: `pnpm tauri dev`

`pnpm dev` alone starts a frontend-only Vite server with no Rust backend — Tauri commands
aren't available there. Use `pnpm tauri dev` to exercise anything that talks to Rust.

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/). Enforced by commitlint.

```
feat(scheduler): add idle-pause support
fix(settings): persist hydration interval on save
docs: update installation guide
```

## Pull Requests

- One feature or fix per PR.
- Target the `main` branch.
- Fill in the PR template.
- Pass lint: `pnpm lint` and Rust checks: `cargo clippy && cargo fmt --check`

## Code Style

**Frontend:** ESLint enforces style automatically via `pnpm lint:fix`.
**Rust:** Run `cargo fmt` before committing.

The reminder scheduler lives in Rust, not the Vue frontend — `setInterval`-style JS timers
drift, get throttled when the webview is hidden, and break across system sleep. Keep timing
logic in `src-tauri/`, not `src/`.

## Reporting Issues

Use the GitHub issue templates for bugs and feature requests.
