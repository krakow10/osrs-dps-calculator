//! Melee DPS calculator for Old School RuneScape.
//!
//! Implements the formulas from the OSRS wiki "Damage per second/Melee" page:
//! <https://oldschool.runescape.wiki/w/Damage_per_second/Melee>

pub mod attacker;
pub mod dps;
pub mod solver;
pub mod target;
pub mod weapon;

pub use attacker::{AttackPrayer, AttackStyle, Attacker, GearBonus, StrengthPrayer};
pub use dps::MeleeDps;
pub use target::{DefencePrayer, NpcTarget, PlayerTarget, Target};
pub use weapon::{
	ABYSSAL_WHIP, ADAMANT_SCIMITAR, BLACK_SCIMITAR, BLADE_OF_SAELDOR, DRAGON_SCIMITAR, GameTicks,
	IRON_SCIMITAR, MITHRIL_SCIMITAR, RUNE_SCIMITAR, STEEL_SCIMITAR, WEAPONS, WeaponStats,
};
