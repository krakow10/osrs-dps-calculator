# osrs-dps-calculator

Melee DPS calculator for Old School RuneScape, in Rust. Implements the formulas from the [OSRS wiki "Damage per second/Melee" page](https://oldschool.runescape.wiki/w/Damage_per_second/Melee).

- **Library** — `MeleeDps::calculate(&attacker, &target)` computes DPS plus all intermediate values (effective levels, max hit, attack/defence rolls, hit chance, average damage). Supports level boosts, prayers, void, and salve/slayer gear bonuses; targets can be NPCs (`NpcTarget`) or players (`PlayerTarget`, with Protect from Melee).
- **Solver** — `Path::new` finds the order of attack/strength level-ups from 1/1 to 99/99 that minimizes total time (each level's exp divided by the DPS at the time it's trained), via exact DP over the level grid.
- **Binary** (`src/main.rs`) — prints the optimal leveling path, one line per level-up, for an attacker wielding the best weapon their attack level allows.

```sh
cargo run   # print the optimal 1/1 -> 99/99 leveling path
cargo test
```
