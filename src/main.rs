use osrs_dps_calculator::{
	AttackPrayer, AttackStyle, Attacker, GearBonus, NpcTarget, StrengthPrayer, WEAPONS,
	solver::{Levels, Solver, print_path},
};

/// Highest level we care about for attack and strength.
const MAX_LEVEL: usize = 99;

/// The attacker at the given levels and style: no boosts, no prayers, no
/// void, no gear bonus, wielding the strongest weapon in `WEAPONS` that the
/// attack level allows.
fn attacker(levels: Levels, style: AttackStyle) -> Attacker {
	let weapon = WEAPONS
		.iter()
		.rev()
		.find(|(_, min_attack)| levels.attack >= *min_attack)
		.map(|(stats, _)| *stats)
		.expect("WEAPONS always contains a weapon with no level requirement");
	Attacker {
		strength: levels.strength,
		attack: levels.attack,
		strength_boost: 0,
		attack_boost: 0,
		strength_prayer: StrengthPrayer::None,
		attack_prayer: AttackPrayer::None,
		weapon,
		attack_style: style,
		void: false,
		gear_bonus: GearBonus::None,
	}
}

fn test_target() -> NpcTarget {
	// PvM: NPC with 1 def and 0 def bonus.
	NpcTarget {
		defence: 1,
		defence_bonus: 0,
	}
}

fn main() {
	let target = test_target();

	let solver = Solver::<MAX_LEVEL>::new(attacker, &target);
	let path = solver.path();
	print_path(&solver, &path);
}

#[cfg(test)]
mod tests {
	use super::*;
	use osrs_dps_calculator::{
		ABYSSAL_WHIP, ADAMANT_SCIMITAR, BLACK_SCIMITAR, BLADE_OF_SAELDOR, DRAGON_SCIMITAR,
		IRON_SCIMITAR, MITHRIL_SCIMITAR, RUNE_SCIMITAR, STEEL_SCIMITAR, WeaponStats,
	};

	fn weapon(attack: u8) -> WeaponStats {
		attacker(
			Levels {
				attack,
				strength: 1,
			},
			AttackStyle::Aggressive,
		)
		.weapon
	}

	/// The weapon switches out exactly when the attacker reaches each
	/// weapon's required attack level.
	#[test]
	fn weapon_switches_at_required_attack_level() {
		// Iron scimitar at 1, until 5 attack.
		assert_eq!(weapon(1), IRON_SCIMITAR);
		assert_eq!(weapon(4), IRON_SCIMITAR);
		// Steel scimitar at 5, until 10 attack.
		assert_eq!(weapon(5), STEEL_SCIMITAR);
		assert_eq!(weapon(9), STEEL_SCIMITAR);
		// Black scimitar at 10, until 20 attack.
		assert_eq!(weapon(10), BLACK_SCIMITAR);
		assert_eq!(weapon(19), BLACK_SCIMITAR);
		// Mithril scimitar at 20, until 30 attack.
		assert_eq!(weapon(20), MITHRIL_SCIMITAR);
		assert_eq!(weapon(29), MITHRIL_SCIMITAR);
		// Adamant scimitar at 30, until 40 attack.
		assert_eq!(weapon(30), ADAMANT_SCIMITAR);
		assert_eq!(weapon(39), ADAMANT_SCIMITAR);
		// Rune scimitar at 40, until 60 attack.
		assert_eq!(weapon(40), RUNE_SCIMITAR);
		assert_eq!(weapon(59), RUNE_SCIMITAR);
		// Dragon scimitar at 60, until 70 attack.
		assert_eq!(weapon(60), DRAGON_SCIMITAR);
		assert_eq!(weapon(69), DRAGON_SCIMITAR);
		// Abyssal whip at 70, until 80 attack.
		assert_eq!(weapon(70), ABYSSAL_WHIP);
		assert_eq!(weapon(79), ABYSSAL_WHIP);
		// Blade of Saeldor at 80.
		assert_eq!(weapon(80), BLADE_OF_SAELDOR);
		assert_eq!(weapon(99), BLADE_OF_SAELDOR);
	}
}
