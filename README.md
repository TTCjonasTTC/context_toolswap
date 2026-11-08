# Context Tool Swap (`toolswap`)

A **Rust native mod (hnmod) for [Horizon](https://playhorizon.dev)** — the open-source Minecraft launcher with native (Rust) client mods. It targets Minecraft 26.3.

When you start breaking a block (or switch the block you're breaking), Context Tool Swap automatically selects the best tool for that block from your hotbar (slots 1–9). Vanilla restarts the break progress when the held item changes, so the swap happens at the start of a new break — never mid-break.

## What it does

- Picks the correct tool for drops (`correct_for_drops`) or any tool with mining speed ≥ 2.0 (so swords and the bare hand are never selected).
- Ordering: `(correct-for-drops, speed)` when the block requires a tool, otherwise pure speed. Ties keep your current slot.
- Skips: creative & spectator mode, non-playing screens, unloaded/out-of-bounds targets, unbreakable blocks (hardness < 0) and instant-break blocks (hardness = 0), and broken tools.

## Commands

| Command | Effect |
|---|---|
| `/toolswap` or `/toolswap status` | Show current state |
| `/toolswap on` / `off` / `toggle` | Enable/disable (per world, default: on) |

No config file; state is stored per world.

## Installation

This is a native Horizon mod (`.hnmod`), not a Fabric/Forge mod.

1. Download [`toolswap.hnmod`](toolswap.hnmod).
2. Copy it into your Horizon **rust** instance's `horizon-native-mods/` directory
   (e.g. `~/.local/share/horizon/instances/<instance>/horizon-native-mods/`).
3. Add `mod.toolswap:true` to `horizon-mods.txt` in the same directory.
4. Launch — Horizon will run a native build (first build can take a while) and ask you to trust the mod once.

## Compatibility / pins

| | |
|---|---|
| Game | Minecraft 26.3 |
| Horizon revision | `6475f62f3c834b7489e1816bf47f47ea5eb2b879` |
| horizon-mixins tool | `e8cd0aa38cffef49cb2e088ef716baa107148c02` (0.6.0) |

The mod pins these revisions in `horizon-native.json` and `mixins/toolswap/horizon-mixin.toml`. If your instance builds at a different Horizon revision, the launcher's "Build anyway" staged-repin usually handles it; otherwise update both pins to your build revision.

## How it works

- Module `toolswap` is merged into `crates/horizon-client/src/game.rs` (per-world state keyed by world root, veinminer-style lease pattern).
- A HEAD injection into `Game::tick_local_hand` checks the previous tick's destroy state and swaps the hotbar slot.
- A HEAD injection into `Game::run_command` implements `/toolswap`.

## Building the `.hnmod`

```bash
zip -q -X -r toolswap.hnmod horizon-native.json mixins/toolswap
```

Then drop the resulting file into `horizon-native-mods/` as above — Horizon compiles it against the pinned client source on the next build.

## License

MIT — see [LICENSE](LICENSE).
