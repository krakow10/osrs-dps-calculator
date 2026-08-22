use osrs_dps_calculator::*;

/// Highest level we care about for attack and strength.
const MAX_LEVEL: usize = 99;

/// The attacker at the given levels and style: no boosts, no prayers, no
/// void, no gear bonus, wielding the strongest weapon in `WEAPONS` that the
/// attack level allows.
fn attacker_bis(levels: Levels, attack_style: AttackStyle) -> Attacker {
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
		attack_style,
		void: false,
		gear_bonus: GearBonus::None,
	}
}

const fn attacker_rune_scim(levels: Levels, attack_style: AttackStyle) -> Attacker {
	Attacker {
		strength: levels.strength,
		attack: levels.attack,
		strength_boost: 0,
		attack_boost: 0,
		strength_prayer: StrengthPrayer::None,
		attack_prayer: AttackPrayer::None,
		weapon: RUNE_SCIMITAR,
		attack_style,
		void: false,
		gear_bonus: GearBonus::None,
	}
}

const ROCK_CRAB: NpcTarget = NpcTarget {
	defence: 1,
	defence_bonus: 0,
};

fn main() {
	let path_rune_scim = Path::new::<MAX_LEVEL, _, _>(attacker_rune_scim, &ROCK_CRAB);
	let path_bis = Path::new::<MAX_LEVEL, _, _>(attacker_bis, &ROCK_CRAB);

	// How does the rune scim path rate on the BIS level up solver's figures?
	let time_rune_scim: f64 = path_rune_scim.iter(attacker_bis, &ROCK_CRAB).map(Step::time).sum();
	let time_bis: f64 = path_bis.iter(attacker_bis, &ROCK_CRAB).map(Step::time).sum();

	println!(
		"rune scim time = {:.1}s
bis time = {:.1}s
rune scim path is {:.1}% worse",
		time_rune_scim,
		time_bis,
		100.0 * (time_rune_scim / time_bis - 1.0),
	);
}

#[cfg(test)]
mod tests {
	use super::*;
	use osrs_dps_calculator::{
		ABYSSAL_WHIP, ADAMANT_SCIMITAR, BLACK_SCIMITAR, BLADE_OF_SAELDOR, DRAGON_SCIMITAR,
		IRON_SCIMITAR, MITHRIL_SCIMITAR, RUNE_SCIMITAR, STEEL_SCIMITAR, WeaponStats,
	};

	fn weapon(attack: u8) -> WeaponStats {
		attacker_bis(
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
