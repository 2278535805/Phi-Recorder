# AGENTS.md — Phi Recorder

## Project Overview

Tauri 2.x desktop app for recording/rendering Phigros rhythm game charts to video.
- **Frontend**: Vue 3 + Vuetify + TypeScript + Vite + vue-i18n
- **Backend**: Rust (Tauri) with `macroquad` for rendering, `phire` for chart parsing, `sasa` for audio
- **Package manager**: pnpm
- **Platforms**: Windows, Linux

## Commands

```bash
# Install dependencies
pnpm install

# Dev (frontend only, http://localhost:5173)
pnpm dev

# Dev (full Tauri app)
cargo tauri dev

# Build (frontend + type-check)
pnpm build

# Build full desktop app
cargo tauri build

# Type-check only
pnpm type-check

# Lint
pnpm lint

# Format
pnpm prettier
```

**No test suite exists.** There are no test commands — skip looking for them.

## Architecture

### Frontend (`src/`)
- Entry: `src/main.ts` → `src/App.vue`
- Router: `src/router/index.ts`
- Views: `AboutView`, `BatchView`, `RenderView`, `RPEView`, `SettingsView`, `TasksView`
- Shared components: `src/components/` (ConfigView, TipCombobox, TipSlider, TipSwitch, TipTextField, TooltipIcon)
- i18n: `src/locales/{en,zh-CN}/*.json` — JSON message files, merged in `main.ts`
- `@` alias maps to `src/` (configured in `vite.config.ts`)

### Backend (`src-tauri/`)
- Entry: `src/main.rs` → calls `phi_recorder_lib::run()` in `src/lib.rs`
- Modules: `common`, `ipc`, `preview`, `render`, `task`, `icon`
- All Tauri IPC commands are registered in `lib.rs` via `generate_handler!`
- CLI mode: binary accepts `--render`, `--preview`, `--play`, `--tweakoffset` flags for headless operation
- Config: `src-tauri/config.toml` bundled as resource; runtime config in `common.rs` via `AppConfig`
- External git deps: `phire` (chart parser), `macroquad` (OpenGL rendering), `sasa` (audio)

### IPC
- Frontend calls Rust via `@tauri-apps/api` invoke
- Child process IPC uses JSON on stdout (`ipc.rs`)

## Key Conventions

- **Prettier**: single quotes, bracket same line, print width 180 (see `.prettierrc`)
- **ESLint**: Vue 3 essential + TypeScript (`.eslintrc.cjs`)
- **TypeScript**: project references — `tsconfig.app.json` (frontend), `tsconfig.node.json` (vite config)
- **Rust**: edition 2021, min rustc 1.77.2; release profile uses LTO + strip
- **i18n**: two locales (en, zh-CN). Fallback is `en`. Missing keys with `title-` prefix return empty string.
- **Custom window**: decorations disabled, custom drag-drop enabled, `useHttpsScheme: true`
- **File association**: `.pez` files (RPE Chart Bundle)

## CI

- `.github/workflows/debug.yaml`: builds on push to any branch (when src changes), uploads MSI/NSIS/AppImage/deb artifacts
- `.github/workflows/release.yaml`: builds on `v*` tags, creates draft GitHub release
- Both use `pnpm/action-setup@v2` (version 8) and `dtolnay/rust-toolchain@stable`

## Gotchas

- `beforeBuildCommand` in `tauri.conf.json` runs `pnpm build` (includes type-check), so `cargo tauri build` triggers full frontend build automatically
- `phi-recorder` in `package.json` dependencies is a self-reference (`"file:"`) — do not remove
- Rust dev profile has `opt-level = 2` for all dependencies (faster dev builds)
- The `phire` crate uses a custom macro `tl_file!` for localized error messages
- Console window is hidden on Windows in non-debug mode via WinAPI
