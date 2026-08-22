use crate::attacker::AttackStyle;
use crate::dps::effective_level;

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
