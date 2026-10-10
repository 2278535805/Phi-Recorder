# Agent notes

## Commands

- Install dependencies with `pnpm install` (the repo uses pnpm).
- `pnpm dev` runs the frontend only; use `pnpm exec tauri dev` for the desktop app.
- `pnpm build` runs the frontend type-check and Vite build in parallel. `pnpm exec tauri build` also runs this via `src-tauri/tauri.conf.json`.
- `pnpm type-check` checks the app TypeScript project. For Rust-only changes, use `cargo check --manifest-path src-tauri/Cargo.toml`.
- `pnpm lint` runs ESLint with `--fix`; `pnpm prettier` runs Prettier `--write .` across the repo. Both modify files—review the resulting diff.
- No test script is defined in `package.json`; CI builds installers but does not run a separate test or lint step.

## Structure and gotchas

- Frontend starts at `src/main.ts`; routes are in `src/router/index.ts`. UI translations are split by feature in `src/locales/{en,zh-CN}/`; keep corresponding keys aligned in both locales.
- The Rust crate is under `src-tauri/`: `src/main.rs` delegates to `src/lib.rs`, where Tauri commands are registered. The renderer, preview, and task queue are in `render.rs`, `preview.rs`, and `task.rs`.
- Child renderer processes send newline-delimited JSON events over stdout (`ipc.rs`, parsed by `task.rs`). Keep non-protocol output off their stdout; diagnostics belong on stderr/logging.
- `src-tauri/config.toml` is the bundled default for renderer settings. `common::AppConfig` is persisted separately in the app config directory; do not treat these as the same config.
- Renderer/chart/audio dependencies `macroquad`, `phire`, and `sasa` are Git dependencies in `src-tauri/Cargo.toml`. The crate requires Rust 1.77.2 or newer.
- `package.json`'s `phi-recorder: "file:"` self-reference is intentional; do not remove it as an apparent unused dependency.
- Prettier uses single quotes and a print width of 180 (`.prettierrc`).
