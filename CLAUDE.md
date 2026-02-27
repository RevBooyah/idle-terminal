# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build          # Debug build
cargo run            # Launch the game
cargo test           # Run all tests (game logic only, no UI tests)
cargo run -- --reset # Delete save data and start fresh
cargo run -- --version
```

Rust 2024 edition — `gen` is a reserved keyword; use `r#gen` / `r#gen_range` when calling rand methods.

## Architecture

```
src/
  main.rs           Entry point, CLI arg parsing
  app.rs            Main event loop (game tick 4Hz, render 30fps, input immediate)
  tui.rs            Terminal init/restore via crossterm
  event.rs          Async event stream (tokio::select! with 3 channels)
  action.rs         Action enum — all mutations flow through dispatch_action()
  layout.rs         6-pane tmux-like layout (PaneId, PaneLayout, compute_layout)
  theme.rs          Color constants (Matrix green palette) and style helpers
  components/       UI layer — reads &GameState to render, emits Actions
    header.rs       Title bar with tick count, prestige level, clock
    dashboard.rs    Resource display + sparkline + prestige progress
    server_rack.rs  Building list + upgrade research view (toggle with 'r')
    network_map.rs  Real hostname/interfaces + ASCII topology tree
    task_terminal.rs  Interactive TypeCommand/IncidentResponse tasks
    log_stream.rs   Scrolling event log with severity colors
    status_bar.rs   Context-sensitive keybind hints
  game/             Pure logic — zero ratatui imports, fully unit-testable
    state.rs        GameState: single source of truth, tick(), prestige(), check_achievements()
    resources.rs    Resources struct + format_si() helper
    buildings.rs    21 building definitions (6 tiers + 3 specials)
    formulas.rs     Cost curve (base * 1.15^count) and production formulas
    upgrades.rs     14 upgrades with prerequisite tree
    tasks.rs        Task definitions and spawn logic
    events.rs       Random events (DDoS, traffic spikes, bonus drops, etc.)
    progression.rs  Prestige formulas + achievement definitions
    network_info.rs Real network discovery (hostname, interfaces, DNS, gateway)
    save.rs         JSON save/load to ~/.local/share/idle-terminal/
```

## Key Patterns

- **Game/UI boundary**: `game/` has no UI imports. Components get `&GameState` for rendering and return `Action`s for mutation.
- **Focus dispatch**: Focused component handles keys first via `handle_key_with_state()`. Unhandled keys fall through to global bindings in `app.rs`.
- **Serde compat**: New GameState fields use `#[serde(default)]` so old saves load correctly.
- **Rust 2024 quirk**: rand's `.gen()` and `.gen_range()` must be called as `.r#gen()` and `.r#gen_range()`.
