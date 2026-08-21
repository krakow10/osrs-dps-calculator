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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameTicks(pub u32);

impl GameTicks {
	/// Length of one tick in seconds.
	pub const SECONDS_PER_TICK: f64 = 0.6;

	/// Total time in seconds. Zero ticks is 0.0 seconds.
	pub fn as_seconds(self) -> f64 {
		f64::from(self.0) * Self::SECONDS_PER_TICK
	}
}

/// Bonuses provided by a melee weapon, as listed on the OSRS wiki.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponStats {
	/// Stab attack bonus.
	pub stab: i32,
	/// Slash attack bonus.
	pub slash: i32,
	/// Crush attack bonus.
	pub crush: i32,
	/// Strength bonus.
	pub strength: i32,
	/// Ticks per attack.
	pub attack_speed: GameTicks,
}

impl WeaponStats {
	/// Attack bonus for the given style.
	///
	/// One-handed weapons: only the controlled style uses the stab bonus;
	/// every other style uses the slash bonus.
	pub fn attack_bonus(self, style: AttackStyle) -> i32 {
		match style {
			AttackStyle::Controlled => self.stab,
			_ => self.slash,
		}
	}
}

/// Iron scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const IRON_SCIMITAR: WeaponStats = WeaponStats {
	stab: 2,
	slash: 10,
	crush: -2,
	strength: 9,
	attack_speed: GameTicks(4),
};

/// Steel scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const STEEL_SCIMITAR: WeaponStats = WeaponStats {
	stab: 3,
	slash: 15,
	crush: -2,
	strength: 14,
	attack_speed: GameTicks(4),
};

/// Black scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const BLACK_SCIMITAR: WeaponStats = WeaponStats {
	stab: 4,
	slash: 19,
	crush: -2,
	strength: 14,
	attack_speed: GameTicks(4),
};

/// Mithril scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const MITHRIL_SCIMITAR: WeaponStats = WeaponStats {
	stab: 5,
	slash: 21,
	crush: -2,
	strength: 20,
	attack_speed: GameTicks(4),
};

/// Adamant scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const ADAMANT_SCIMITAR: WeaponStats = WeaponStats {
	stab: 6,
	slash: 29,
	crush: -2,
	strength: 28,
	attack_speed: GameTicks(4),
};

/// Rune scimitar.
/// <https://oldschool.runescape.wiki/w/Rune_scimitar>
pub const RUNE_SCIMITAR: WeaponStats = WeaponStats {
	stab: 7,
	slash: 45,
	crush: -2,
	strength: 44,
	attack_speed: GameTicks(4),
};

/// Dragon scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const DRAGON_SCIMITAR: WeaponStats = WeaponStats {
	stab: 8,
	slash: 67,
	crush: -2,
	strength: 66,
	attack_speed: GameTicks(4),
};

/// Abyssal whip.
/// <https://oldschool.runescape.wiki/w/Abyssal_whip>
pub const ABYSSAL_WHIP: WeaponStats = WeaponStats {
	stab: 0,
	slash: 82,
	crush: 0,
	strength: 82,
	attack_speed: GameTicks(4),
};

/// Blade of Saeldor, fully charged (the inactive blade has all bonuses +0).
/// <https://oldschool.runescape.wiki/w/Blade_of_Saeldor>
pub const BLADE_OF_SAELDOR: WeaponStats = WeaponStats {
	stab: 0,
	slash: 100,
	crush: 0,
	strength: 93,
	attack_speed: GameTicks(4),
};

/// The weapons the attacker progresses through, weakest to strongest, each
/// paired with the minimum attack level required to wield it.
///
/// The scimitars can be wielded at any level; the abyssal whip requires 70
/// attack and the blade of Saeldor 80.
pub const WEAPONS: &[(WeaponStats, u32)] = &[
	(IRON_SCIMITAR, 1),
	(STEEL_SCIMITAR, 5),
	(BLACK_SCIMITAR, 10),
	(MITHRIL_SCIMITAR, 20),
	(ADAMANT_SCIMITAR, 30),
	(RUNE_SCIMITAR, 40),
	(DRAGON_SCIMITAR, 60),
	(ABYSSAL_WHIP, 70),
	(BLADE_OF_SAELDOR, 80),
];

/// The player doing the attacking.
#[derive(Debug, Clone, Copy)]
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
	/// The weapon being wielded.
	pub weapon: WeaponStats,
	pub attack_style: AttackStyle,
	/// Wearing full melee void.
	pub void: bool,
	/// Damage/accuracy bonus from a salve amulet or slayer helm. Only applies
	/// to NPC targets.
	pub gear_bonus: GearBonus,
}

/// The target being attacked.
///
/// A trait (rather than an enum) so the target-specific logic lives on the
/// target type itself and `MeleeDps::calculate` dispatches statically to
/// the concrete type at compile time.
pub trait Target {
	/// Whether the attacker's salve amulet / slayer helm bonus applies.
	///
	/// Per the wiki, these bonuses only work on monsters.
	const ACCEPTS_GEAR_BONUS: bool;

	/// Whether protect from melee is active.
	fn protect_from_melee(&self) -> bool {
		false
	}

	/// `(effective_defence, defence_roll)` for this target.
	///
	/// NPCs have no effective level and use `(defence + 9) * (defence bonus + 64)`;
	/// players use the effective defence level derived from their base level,
	/// boost, prayer, and attack style.
	fn defence(&self) -> (Option<u32>, u32);
}

/// NPC target.
#[derive(Debug, Clone)]
pub struct NpcTarget {
	/// Base defence level: the shield-icon value on the wiki.
	pub defence: u32,
	/// Target's stab/slash/crush defence bonus matching the attack type.
	pub defence_bonus: i32,
}

impl Target for NpcTarget {
	const ACCEPTS_GEAR_BONUS: bool = true;

	fn defence(&self) -> (Option<u32>, u32) {
		// NPC: (defence level + 9) * (defence bonus + 64)
		(
			None,
			((u64::from(self.defence) + 9) * (self.defence_bonus as u64 + 64)) as u32,
		)
	}
}

/// Player target.
#[derive(Debug, Clone)]
pub struct PlayerTarget {
	/// Base defence level.
	pub defence: u32,
	/// Temporary defence boost.
	pub defence_boost: i32,
	/// Defence prayer being used.
	pub defence_prayer: DefencePrayer,
	/// Attack style (affects defence style bonus).
	pub attack_style: AttackStyle,
	/// Target's stab/slash/crush defence bonus matching the attack type.
	pub defence_bonus: i32,
	/// Protect from Melee active.
	pub protect_from_melee: bool,
}

impl Target for PlayerTarget {
	const ACCEPTS_GEAR_BONUS: bool = false;

	fn protect_from_melee(&self) -> bool {
		self.protect_from_melee
	}

	fn defence(&self) -> (Option<u32>, u32) {
		let def_style_bonus = match self.attack_style {
			AttackStyle::Defensive => 3,
			AttackStyle::Controlled => 1,
			_ => 0,
		};
		let eff = effective_level(
			self.defence,
			self.defence_boost,
			self.defence_prayer.multiplier(),
			def_style_bonus,
			false,
		);
		(
			Some(eff),
			(eff as u64 * (self.defence_bonus as u64 + 64)) as u32,
		)
	}
}

/// All intermediate values and the final DPS.
#[derive(Debug)]
pub struct MeleeDps {
	pub effective_strength: u32,
	pub max_hit: u32,
	pub effective_attack: u32,
	pub attack_roll: u32,
	/// `Some` for player targets, `None` for NPCs.
	pub effective_defence: Option<u32>,
	pub defence_roll: u32,
	pub hit_chance: f64,
	pub average_damage_per_attack: f64,
	pub dps: f64,
}

impl MeleeDps {
	/// Calculate melee DPS using the formulas from the OSRS wiki.
	pub fn calculate<T: Target>(attacker: &Attacker, target: &T) -> MeleeDps {
		// Salve amulet and slayer helm bonuses only work on monsters.
		let gear = if T::ACCEPTS_GEAR_BONUS {
			attacker.gear_bonus
		} else {
			GearBonus::None
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
		let max_hit_base = ((effective_strength as f64 * (attacker.weapon.strength as f64 + 64.0)
			+ 320.0) / 640.0)
			.floor() as u64;
		let mut max_hit = (max_hit_base as f64 * gear.multiplier()).floor() as u64;
		if target.protect_from_melee() {
			max_hit = (max_hit as f64 * 0.6).floor() as u64;
		}
		let max_hit = max_hit as u32;

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
			* (attacker.weapon.attack_bonus(attacker.attack_style) as f64 + 64.0)
			* gear.multiplier())
		.floor() as u32;

		// --- Steps five & six: defence roll ------------------------------------
		let (effective_defence, defence_roll) = target.defence();

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
		let average_damage_per_attack = if max_hit == 0 {
			hit_chance
		} else {
			hit_chance * (max_hit as f64 / 2.0 + 1.0 / max_hit as f64 + 1.0)
		};
		let dps = average_damage_per_attack / attacker.weapon.attack_speed.as_seconds();

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
fn effective_level(level: u32, boost: i32, prayer_mult: f64, style_bonus: u32, void: bool) -> u32 {
	let mut lvl = (level as f64 + boost as f64) * prayer_mult;
	lvl = lvl.floor() + style_bonus as f64 + 8.0;
	if void {
		lvl *= 1.1;
	}
	lvl.floor() as u32
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
		// no void, weapon with 80 atk / 60 str bonus, 2-tick speed.
		let attacker = Attacker {
			strength: 99,
			attack: 99,
			strength_boost: 10,
			attack_boost: 10,
			strength_prayer: StrengthPrayer::Piety,
			attack_prayer: AttackPrayer::Piety,
			weapon: WeaponStats {
				stab: 80,
				slash: 80,
				crush: 0,
				strength: 60,
				attack_speed: GameTicks(2),
			},
			attack_style: AttackStyle::Aggressive,
			void: false,
			gear_bonus: GearBonus::None,
		};
		// NPC with 50 def, 40 def bonus.
		let target = NpcTarget {
			defence: 50,
			defence_bonus: 40,
		};

		let r = MeleeDps::calculate(&attacker, &target);

		// eff strength: floor(109 * 1.23) = 134; 134 + 3 (aggressive) + 8 = 145
		assert_eq!(r.effective_strength, 145);
		// max hit: floor((145 * 124 + 320) / 640) = floor(28.59) = 28
		assert_eq!(r.max_hit, 28);
		// eff attack: floor(109 * 1.20) = 130; 130 + 0 (aggressive) + 8 = 138
		assert_eq!(r.effective_attack, 138);
		// attack roll: 138 * 144 = 19872
		assert_eq!(r.attack_roll, 19_872);
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
			weapon: WeaponStats {
				stab: 80,
				slash: 80,
				crush: 0,
				strength: 60,
				attack_speed: GameTicks(2),
			},
			attack_style: AttackStyle::Aggressive,
			void: false,
			// Salve bonuses don't apply to player targets, so this must be ignored.
			gear_bonus: GearBonus::EnhancedSalve,
		};
		// Player target: 99 def +15, piety (1.20), defensive style, 100 def bonus, PFM.
		let target = PlayerTarget {
			defence: 99,
			defence_boost: 15,
			defence_prayer: DefencePrayer::Piety,
			attack_style: AttackStyle::Defensive,
			defence_bonus: 100,
			protect_from_melee: true,
		};

		let r = MeleeDps::calculate(&attacker, &target);

		// max hit: no gear bonus vs. players, so 28; then PFM: floor(28 * 0.6) = 16
		assert_eq!(r.max_hit, 16);
		// attack roll ignores the salve (e) gear bonus vs. players: 138 * 144 = 19872
		assert_eq!(r.attack_roll, 19_872);
		// eff def: floor(114 * 1.20) = 136; 136 + 3 (defensive) + 8 = 147
		assert_eq!(r.effective_defence, Some(147));
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
			// Weapon with -64 str zeroes the strength term: (eff_str * 0 + 320) / 640 = 0.5 -> 0
			weapon: WeaponStats {
				stab: 0,
				slash: 0,
				crush: 0,
				strength: -64,
				attack_speed: GameTicks(2),
			},
			attack_style: AttackStyle::Aggressive,
			void: false,
			gear_bonus: GearBonus::None,
		};
		let target = NpcTarget {
			defence: 1,
			defence_bonus: 0,
		};

		let r = MeleeDps::calculate(&attacker, &target);
		assert_eq!(r.max_hit, 0);
		// Every successful hit rolls 0 and is bumped up to 1.
		assert!((r.average_damage_per_attack - r.hit_chance).abs() < 1e-12);
	}

	#[test]
	fn blade_of_saeldor_aggressive() {
		// 99/99, no boosts or prayers, aggressive style, full void,
		// fully charged blade of Saeldor vs. an NPC with 1 def and 0 def bonus.
		let attacker = Attacker {
			strength: 99,
			attack: 99,
			strength_boost: 0,
			attack_boost: 0,
			strength_prayer: StrengthPrayer::None,
			attack_prayer: AttackPrayer::None,
			weapon: BLADE_OF_SAELDOR,
			attack_style: AttackStyle::Aggressive,
			void: true,
			gear_bonus: GearBonus::None,
		};
		let target = NpcTarget {
			defence: 1,
			defence_bonus: 0,
		};

		let r = MeleeDps::calculate(&attacker, &target);

		// eff strength: floor((99 + 3 + 8) * 1.1) = 121
		assert_eq!(r.effective_strength, 121);
		// max hit: floor((121 * 157 + 320) / 640) = floor(30.18) = 30
		assert_eq!(r.max_hit, 30);
		// eff attack: floor((99 + 0 + 8) * 1.1) = floor(117.7) = 117
		assert_eq!(r.effective_attack, 117);
		// attack roll: 117 * (100 + 64) = 19188
		assert_eq!(r.attack_roll, 19_188);
		// def roll: (1 + 9) * (0 + 64) = 640
		assert_eq!(r.defence_roll, 640);
	}

	#[test]
	fn blade_of_saeldor_controlled() {
		// Same setup, but the controlled style uses the weapon's stab bonus (0)
		// instead of its slash bonus (100).
		let attacker = Attacker {
			strength: 99,
			attack: 99,
			strength_boost: 0,
			attack_boost: 0,
			strength_prayer: StrengthPrayer::None,
			attack_prayer: AttackPrayer::None,
			weapon: BLADE_OF_SAELDOR,
			attack_style: AttackStyle::Controlled,
			void: true,
			gear_bonus: GearBonus::None,
		};
		let target = NpcTarget {
			defence: 1,
			defence_bonus: 0,
		};

		let r = MeleeDps::calculate(&attacker, &target);

		// eff attack: floor((99 + 1 + 8) * 1.1) = floor(118.8) = 118
		assert_eq!(r.effective_attack, 118);
		// attack roll: 118 * (0 + 64) = 7552
		assert_eq!(r.attack_roll, 7_552);
	}
}
