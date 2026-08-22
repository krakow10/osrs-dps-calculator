use crate::weapon::WeaponStats;

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
	pub(crate) fn multiplier(self) -> f64 {
		match self {
			GearBonus::None => 1.0,
			GearBonus::Slayer => 7.0 / 6.0,
			GearBonus::EnhancedSalve => 1.2,
		}
	}
}

/// Bonuses provided by gear other than the weapon (helm, body, amulet,
/// shield, etc.), as listed on the OSRS wiki.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GearStats {
	/// Stab attack bonus.
	pub stab: i32,
	/// Slash attack bonus.
	pub slash: i32,
	/// Crush attack bonus.
	pub crush: i32,
	/// Strength bonus. Feeds into max hit, like the weapon's.
	pub strength: i32,
}

impl GearStats {
	/// No gear attack bonuses.
	pub const NONE: GearStats = GearStats {
		stab: 0,
		slash: 0,
		crush: 0,
		strength: 0,
	};

	/// Total attack bonus for the given style.
	///
	/// Like [`WeaponStats::attack_bonus`]: the controlled style uses the stab
	/// bonus; every other style uses the slash bonus.
	pub fn attack_bonus(self, style: AttackStyle) -> i32 {
		match style {
			AttackStyle::Controlled => self.stab,
			_ => self.slash,
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
	pub(crate) fn multiplier(self) -> f64 {
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
	pub(crate) fn multiplier(self) -> f64 {
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

/// The player doing the attacking.
#[derive(Debug, Clone, Copy)]
pub struct Attacker {
	pub strength: u8,
	pub attack: u8,
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
	/// Bonuses from gear other than the weapon slot (helm, body, amulet,
	/// shield, etc.). The attack bonuses feed into the attack roll; the
	/// strength bonus feeds into max hit.
	pub gear: GearStats,
	pub attack_style: AttackStyle,
	/// Wearing full melee void.
	pub void: bool,
	/// Damage/accuracy bonus from a salve amulet or slayer helm. Only applies
	/// to NPC targets.
	pub gear_bonus: GearBonus,
}
