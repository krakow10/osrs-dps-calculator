//! Melee DPS calculator for Old School RuneScape.
//!
//! Implements the formulas from the OSRS wiki "Damage per second/Melee" page:
//! <https://oldschool.runescape.wiki/w/Damage_per_second/Melee>

/// Melee attack style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackStyle {
	/// +3 attack.
	Accurate,
	/// +3 strength.
	Aggressive,
	/// +1 attack and +1 strength.
	Controlled,
	/// +3 defence (defender only).
	Defensive,
}

/// Target-specific gear bonus.
///
/// Per the wiki, the slayer helm and salve amulet bonuses do NOT stack;
/// use the better of the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GearBonus {
	/// No gear bonus (1.0).
	#[default]
	None,
	/// 7/6 — slayer helm (on task) or salve amulet (undead).
	Slayer,
	/// 1.2 — salve amulet (e) or (ei).
	EnhancedSalve,
}

impl GearBonus {
	fn multiplier(self) -> f64 {
		match self {
			GearBonus::None => 1.0,
			GearBonus::Slayer => 7.0 / 6.0,
			GearBonus::EnhancedSalve => 1.2,
		}
	}
}

/// Prayer boosting strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrengthPrayer {
	/// No prayer (1.0).
	#[default]
	None,
	/// Burst of Strength (1.05).
	BurstOfStrength,
	/// Superhuman Strength (1.10).
	SuperhumanStrength,
	/// Ultimate Strength (1.15).
	UltimateStrength,
	/// Chivalry (1.18).
	Chivalry,
	/// Piety (1.23).
	Piety,
}

impl StrengthPrayer {
	fn multiplier(self) -> f64 {
		match self {
			StrengthPrayer::None => 1.0,
			StrengthPrayer::BurstOfStrength => 1.05,
			StrengthPrayer::SuperhumanStrength => 1.10,
			StrengthPrayer::UltimateStrength => 1.15,
			StrengthPrayer::Chivalry => 1.18,
			StrengthPrayer::Piety => 1.23,
		}
	}
}

/// Prayer boosting attack (melee hit chance).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackPrayer {
	/// No prayer (1.0).
	#[default]
	None,
	/// Clarity of Thought (1.05).
	ClarityOfThought,
	/// Improved Reflexes (1.10).
	ImprovedReflexes,
	/// Incredible Reflexes (1.15).
	IncredibleReflexes,
	/// Chivalry (1.15).
	Chivalry,
	/// Piety (1.20).
	Piety,
}

impl AttackPrayer {
	fn multiplier(self) -> f64 {
		match self {
			AttackPrayer::None => 1.0,
			AttackPrayer::ClarityOfThought => 1.05,
			AttackPrayer::ImprovedReflexes => 1.10,
			AttackPrayer::IncredibleReflexes => 1.15,
			AttackPrayer::Chivalry => 1.15,
			AttackPrayer::Piety => 1.20,
		}
	}
}

/// Prayer boosting defence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DefencePrayer {
	/// No prayer (1.0).
	#[default]
	None,
	/// Rock Skin (1.05).
	RockSkin,
	/// Superhuman Defence (1.10).
	SuperhumanDefence,
	/// Ultimate Defence (1.15).
	UltimateDefence,
	/// Chivalry (1.15).
	Chivalry,
	/// Piety (1.20).
	Piety,
}

impl DefencePrayer {
	fn multiplier(self) -> f64 {
		match self {
			DefencePrayer::None => 1.0,
			DefencePrayer::RockSkin => 1.05,
			DefencePrayer::SuperhumanDefence => 1.10,
			DefencePrayer::UltimateDefence => 1.15,
			DefencePrayer::Chivalry => 1.15,
			DefencePrayer::Piety => 1.20,
		}
	}
}

/// A number of game ticks. One tick is exactly 0.6 seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameTicks(pub u32);

impl GameTicks {
	/// Length of one tick in seconds.
	pub const SECONDS_PER_TICK: f64 = 0.6;

	/// Total time in seconds. Zero ticks is 0.0 seconds.
	pub fn as_seconds(self) -> f64 {
		f64::from(self.0) * Self::SECONDS_PER_TICK
	}
}

/// The player doing the attacking.
#[derive(Debug, Clone)]
pub struct Attacker {
	pub strength: u32,
	pub attack: u32,
	/// Temporary strength level boost (potion, cape, etc.).
	pub strength_boost: i32,
	/// Temporary attack level boost (potion, cape, etc.).
	pub attack_boost: i32,
	/// Strength prayer being used.
	pub strength_prayer: StrengthPrayer,
	/// Attack prayer being used.
	pub attack_prayer: AttackPrayer,
	/// "Melee Strength" bonus from the equipment stats window.
	pub equipment_strength_bonus: i32,
	/// Stab/Slash/Crush bonus matching the weapon's attack type.
	pub equipment_attack_bonus: i32,
	pub attack_style: AttackStyle,
	/// Wearing full melee void.
	pub void: bool,
	/// Damage/accuracy bonus from a salve amulet or slayer helm. Only applies
	/// to NPC targets.
	pub gear_bonus: GearBonus,
	/// Weapon attack speed, in ticks per attack.
	pub attack_speed: GameTicks,
}

/// The target being attacked (an NPC or a player).
#[derive(Debug, Clone)]
pub enum Target {
	/// NPC target.
	Npc {
		/// Base defence level: the shield-icon value on the wiki.
		defence: u32,
		/// Target's stab/slash/crush defence bonus matching the attack type.
		defence_bonus: i32,
	},
	/// Player target.
	Player {
		/// Base defence level.
		defence: u32,
		/// Temporary defence boost.
		defence_boost: i32,
		/// Defence prayer being used.
		defence_prayer: DefencePrayer,
		/// Attack style (affects defence style bonus).
		attack_style: AttackStyle,
		/// Target's stab/slash/crush defence bonus matching the attack type.
		defence_bonus: i32,
		/// Protect from Melee active.
		protect_from_melee: bool,
	},
}

/// All intermediate values and the final DPS.
#[derive(Debug, Clone, PartialEq)]
pub struct MeleeDps {
	pub effective_strength: f64,
	pub max_hit: f64,
	pub effective_attack: f64,
	pub attack_roll: f64,
	/// `Some` for player targets, `None` for NPCs.
	pub effective_defence: Option<f64>,
	pub defence_roll: u32,
	pub hit_chance: f64,
	pub average_damage_per_attack: f64,
	pub dps: f64,
}

impl MeleeDps {
	/// Calculate melee DPS using the formulas from the OSRS wiki.
	pub fn calculate(attacker: &Attacker, target: &Target) -> MeleeDps {
		// Salve amulet and slayer helm bonuses only work on monsters.
		let gear = match target {
			Target::Npc { .. } => attacker.gear_bonus,
			Target::Player { .. } => GearBonus::None,
		};
		// --- Step one: effective strength level -------------------------------
		let strength_style_bonus = match attacker.attack_style {
			AttackStyle::Aggressive => 3,
			AttackStyle::Controlled => 1,
			_ => 0,
		};
		let effective_strength = effective_level(
			attacker.strength,
			attacker.strength_boost,
			attacker.strength_prayer.multiplier(),
			strength_style_bonus,
			attacker.void,
		);

		// --- Step two: max hit ------------------------------------------------
		// eff_strength * (str bonus + 64) + 320, / 640, floor, then gear bonus, floor.
		let max_hit_base = ((effective_strength as f64
			* (attacker.equipment_strength_bonus as f64 + 64.0)
			+ 320.0) / 640.0);
		let mut max_hit = (max_hit_base as f64 * gear.multiplier());
		if let Target::Player {
			protect_from_melee: true,
			..
		} = target
		{
			max_hit = max_hit as f64 * 0.6;
		}

		// --- Step three: effective attack level -------------------------------
		let attack_style_bonus = match attacker.attack_style {
			AttackStyle::Accurate => 3,
			AttackStyle::Controlled => 1,
			_ => 0,
		};
		let effective_attack = effective_level(
			attacker.attack,
			attacker.attack_boost,
			attacker.attack_prayer.multiplier(),
			attack_style_bonus,
			attacker.void,
		);

		// --- Step four: attack roll --------------------------------------------
		let attack_roll = (effective_attack as f64
			* (attacker.equipment_attack_bonus as f64 + 64.0)
			* gear.multiplier());

		// --- Steps five & six: defence roll ------------------------------------
		let (effective_defence, defence_roll) = match target {
			Target::Player {
				defence,
				defence_boost,
				defence_prayer,
				attack_style,
				defence_bonus,
				..
			} => {
				let def_style_bonus = match attack_style {
					AttackStyle::Defensive => 3,
					AttackStyle::Controlled => 1,
					_ => 0,
				};
				let eff = effective_level(
					*defence,
					*defence_boost,
					defence_prayer.multiplier(),
					def_style_bonus,
					false,
				);
				(Some(eff), eff as u64 * (*defence_bonus as u64 + 64))
			}
			// NPC: (defence level + 9) * (defence bonus + 64)
			Target::Npc {
				defence,
				defence_bonus,
			} => (
				None,
				(u64::from(*defence) + 9) * (*defence_bonus as u64 + 64),
			),
		};
		let defence_roll = defence_roll as u32;

		// --- Step seven: hit chance --------------------------------------------
		let (a, d) = (attack_roll as f64, defence_roll as f64);
		let hit_chance = if a > d {
			1.0 - (d + 2.0) / (2.0 * (a + 1.0))
		} else {
			a / (2.0 * (d + 1.0))
		};

		// --- Step eight: damage output ------------------------------------------
		// Hit chance * (max_hit / 2 + 1 / max_hit + 1). The +1/max_hit term
		// accounts for a 0 roll on a successful hit being bumped up to 1.
		// (If max hit is 0, every successful hit deals exactly 1.)
		let average_damage_per_attack = if max_hit == 0.0 {
			hit_chance
		} else {
			hit_chance * (max_hit as f64 / 2.0 + 1.0 / max_hit as f64 + 1.0)
		};
		let dps = average_damage_per_attack / attacker.attack_speed.as_seconds();

		MeleeDps {
			effective_strength,
			max_hit,
			effective_attack,
			attack_roll,
			effective_defence,
			defence_roll,
			hit_chance,
			average_damage_per_attack,
			dps,
		}
	}
}

/// Steps one/three/five: (level + boost) * prayer, floor, + style bonus, +8,
/// optional *1.1 void, floor.
fn effective_level(level: u32, boost: i32, prayer_mult: f64, style_bonus: u32, void: bool) -> f64 {
	let mut lvl = (level as f64 + boost as f64) * prayer_mult;
	lvl = lvl + style_bonus as f64 + 8.0;
	if void {
		lvl *= 1.1;
	}
	lvl
}

impl std::fmt::Display for MeleeDps {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "  Effective strength: {}", self.effective_strength)?;
		writeln!(f, "  Max hit:            {}", self.max_hit)?;
		writeln!(f, "  Effective attack:   {}", self.effective_attack)?;
		writeln!(f, "  Attack roll:        {}", self.attack_roll)?;
		if let Some(eff_def) = self.effective_defence {
			writeln!(f, "  Effective defence:  {eff_def}")?;
		}
		writeln!(f, "  Defence roll:       {}", self.defence_roll)?;
		writeln!(
			f,
			"  Hit chance:         {:.4} ({:.2}%)",
			self.hit_chance,
			self.hit_chance * 100.0
		)?;
		writeln!(
			f,
			"  Avg damage/hit:     {:.4}",
			self.average_damage_per_attack
		)?;
		write!(f, "  DPS:              {:.4}", self.dps)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn npc_target_matches_hand_calculation() {
		// 99/99 with +10/+10, piety (1.23 str / 1.20 atk), aggressive style,
		// no void, str bonus 60, atk bonus 80, 1.0s weapon.
		let attacker = Attacker {
			strength: 99,
			attack: 99,
			strength_boost: 10,
			attack_boost: 10,
			strength_prayer: StrengthPrayer::Piety,
			attack_prayer: AttackPrayer::Piety,
			equipment_strength_bonus: 60,
			equipment_attack_bonus: 80,
			attack_style: AttackStyle::Aggressive,
			void: false,
			gear_bonus: GearBonus::None,
			attack_speed: GameTicks(2),
		};
		// NPC with 50 def, 40 def bonus.
		let target = Target::Npc {
			defence: 50,
			defence_bonus: 40,
		};

		let r = MeleeDps::calculate(&attacker, &target);

		// eff strength: floor(109 * 1.23) = 134; 134 + 3 (aggressive) + 8 = 145
		assert_eq!(r.effective_strength, 145.0);
		// max hit: floor((145 * 124 + 320) / 640) = floor(28.59) = 28
		assert_eq!(r.max_hit, 28.0);
		// eff attack: floor(109 * 1.20) = 130; 130 + 0 (aggressive) + 8 = 138
		assert_eq!(r.effective_attack, 138.0);
		// attack roll: 138 * 144 = 19872
		assert_eq!(r.attack_roll, 19_872.0);
		// def roll: (50 + 9) * (40 + 64) = 6136
		assert_eq!(r.defence_roll, 6_136);
		assert_eq!(r.effective_defence, None);

		let expected_hit_chance = 1.0 - 6138.0 / (2.0 * 19_873.0);
		assert!((r.hit_chance - expected_hit_chance).abs() < 1e-12);

		// 2 ticks = 1.2s per attack.
		let expected_dps = expected_hit_chance * (14.0 + 1.0 / 28.0 + 1.0) / 1.2;
		assert!((r.dps - expected_dps).abs() < 1e-9);
		assert!((r.dps - 10.5948).abs() < 1e-3);
	}

	#[test]
	fn player_target_with_gear_and_pfm() {
		let attacker = Attacker {
			strength: 99,
			attack: 99,
			strength_boost: 10,
			attack_boost: 10,
			strength_prayer: StrengthPrayer::Piety,
			attack_prayer: AttackPrayer::Piety,
			equipment_strength_bonus: 60,
			equipment_attack_bonus: 80,
			attack_style: AttackStyle::Aggressive,
			void: false,
			// Salve bonuses don't apply to player targets, so this must be ignored.
			gear_bonus: GearBonus::EnhancedSalve,
			attack_speed: GameTicks(2),
		};
		// Player target: 99 def +15, piety (1.20), defensive style, 100 def bonus, PFM.
		let target = Target::Player {
			defence: 99,
			defence_boost: 15,
			defence_prayer: DefencePrayer::Piety,
			attack_style: AttackStyle::Defensive,
			defence_bonus: 100,
			protect_from_melee: true,
		};

		let r = MeleeDps::calculate(&attacker, &target);

		// max hit: no gear bonus vs. players, so 28; then PFM: floor(28 * 0.6) = 16
		assert_eq!(r.max_hit, 16.0);
		// attack roll ignores the salve (e) gear bonus vs. players: 138 * 144 = 19872
		assert_eq!(r.attack_roll, 19_872.0);
		// eff def: floor(114 * 1.20) = 136; 136 + 3 (defensive) + 8 = 147
		assert_eq!(r.effective_defence, Some(147.0));
		// def roll: 147 * 164 = 24108
		assert_eq!(r.defence_roll, 24_108);
	}

	#[test]
	fn zero_max_hit_is_handled() {
		let attacker = Attacker {
			strength: 1,
			attack: 1,
			strength_boost: 0,
			attack_boost: 0,
			strength_prayer: StrengthPrayer::None,
			attack_prayer: AttackPrayer::None,
			// Bonus of -64 zeroes the strength term: (eff_str * 0 + 320) / 640 = 0.5 -> 0
			equipment_strength_bonus: -64,
			equipment_attack_bonus: 0,
			attack_style: AttackStyle::Aggressive,
			void: false,
			gear_bonus: GearBonus::None,
			attack_speed: GameTicks(2),
		};
		let target = Target::Npc {
			defence: 1,
			defence_bonus: 0,
		};

		let r = MeleeDps::calculate(&attacker, &target);
		assert_eq!(r.max_hit, 0.0);
		// Every successful hit rolls 0 and is bumped up to 1.
		assert!((r.average_damage_per_attack - r.hit_chance).abs() < 1e-12);
	}
}
