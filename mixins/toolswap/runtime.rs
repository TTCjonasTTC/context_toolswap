// Context Tool Swap: when the player starts breaking a block, switch the
// hotbar to the best tool for that block among the nine hotbar slots.
// Vanilla restarts the break progress when the held item changes, so the
// swap happens at the start of a new break (or when the target block
// changes) and never mid-break.
use super::{Game, Screen};

pub(super) struct State {
    enabled: bool,
    target: Option<[i32; 3]>,
}
impl Default for State {
    fn default() -> Self {
        Self { enabled: true, target: None }
    }
}

thread_local! {
    static STATES: std::cell::RefCell<std::collections::BTreeMap<std::path::PathBuf, State>> = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}
fn with_state<T>(game: &mut Game<'_>, run: impl FnOnce(&mut Game<'_>, &mut State) -> T) -> T {
    struct Lease {
        key: std::path::PathBuf,
        state: State,
    }
    impl Drop for Lease {
        fn drop(&mut self) {
            STATES.with(|states| {
                states
                    .borrow_mut()
                    .insert(self.key.clone(), std::mem::take(&mut self.state));
            });
        }
    }
    let key = game.world_root.clone();
    let state = STATES.with(|states| states.borrow_mut().remove(&key).unwrap_or_default());
    let mut lease = Lease { key, state };
    run(game, &mut lease.state)
}

/// A stack is broken (beyond repair) when its damage reached its max.
fn broken(items: &horizon_client::items::Items, stack: &horizon_client::items::ItemStack) -> bool {
    items
        .get(&stack.item)
        .is_some_and(|info| info.max_damage > 0 && stack.damage >= info.max_damage)
}

impl Game<'_> {
    pub(super) fn toolswap_tick(&mut self) {
        with_state(self, |game, state| {
            let active = game.clicks.destroying.active;
            let pos = game.clicks.destroying.pos;
            if !active
                || game.screen != Screen::Playing
                || game.creative
                || game.player.is_none()
                || game.spectator()
            {
                state.target = None;
                return;
            }
            // One evaluation per break target; re-arm when the block changes.
            if state.target == Some(pos) {
                return;
            }
            state.target = Some(pos);
            if !state.enabled {
                return;
            }
            if pos[1] < game.world.shape.min_y
                || pos[1] >= game.world.shape.max_y()
                || !game.world.loaded(pos[0], pos[2])
            {
                return;
            }
            let facts = game.res.facts().get(game.world.block(pos[0], pos[1], pos[2]));
            // Unbreakable (-1) and instant-break (0) blocks need no tool.
            if facts.hardness <= 0.0 {
                return;
            }
            let block = facts.name.clone();
            let items = game.items();
            let current = game.selected.min(8);
            let current_stack = game.inventory.get(current).and_then(|slot| slot.as_ref());
            let mut best = current;
            let mut best_score = (
                items.correct_for_drops(current_stack, &block),
                items.mining_speed(current_stack, &block),
            );
            for slot in 0..9 {
                let Some(stack) = game.inventory.get(slot).and_then(|slot| slot.as_ref()) else {
                    continue;
                };
                if broken(items, stack) {
                    continue;
                }
                let speed = items.mining_speed(Some(stack), &block);
                let correct = items.correct_for_drops(Some(stack), &block);
                // A tool counts when it is correct for drops or clearly
                // faster than the bare hand (excludes swords' 1.5 default).
                if !correct && speed < 2.0 {
                    continue;
                }
                let score = if facts.requires_tool {
                    (correct, speed)
                } else {
                    (false, speed)
                };
                // Strictly-greater: ties keep the current slot (no jitter).
                if score > best_score {
                    best = slot;
                    best_score = score;
                }
            }
            if best != game.selected {
                game.selected = best;
            }
        })
    }
    pub(super) fn toolswap_command(&mut self, command: &str) -> Option<bool> {
        with_state(self, |game, state| {
            let mut words = command.split_whitespace();
            if words.next()? != "toolswap" {
                return None;
            }
            let args: Vec<_> = words.collect();
            let message = match args.as_slice() {
                [] | ["status"] => format!(
                    "Context Tool Swap: {}",
                    if state.enabled { "on" } else { "off" }
                ),
                ["toggle"] | ["on"] | ["off"] => {
                    let on = match args[0] {
                        "on" => true,
                        "off" => false,
                        _ => !state.enabled,
                    };
                    state.enabled = on;
                    format!("Context Tool Swap: {}", if on { "on" } else { "off" })
                }
                _ => {
                    game.command_failure(horizon_client::text::Text::literal(
                        "Usage: /toolswap [status|toggle|on|off]",
                    ));
                    return Some(false);
                }
            };
            game.system_message(horizon_client::text::Text::literal(message));
            Some(true)
        })
    }
}
