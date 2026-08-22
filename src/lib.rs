//! Melee DPS calculator for Old School RuneScape.
//!
//! Implements the formulas from the OSRS wiki "Damage per second/Melee" page:
//! <https://oldschool.runescape.wiki/w/Damage_per_second/Melee>

pub mod attacker;
pub mod dps;
pub mod solver;
pub mod target;
pub mod weapon;

pub use attacker::*;
pub use dps::*;
pub use solver::*;
pub use target::*;
pub use weapon::*;
