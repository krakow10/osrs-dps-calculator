use osrs_dps_calculator::*;

/// Highest level we care about for attack and strength.
const MAX_LEVEL: usize = 99;

/// The attacker at the given levels and style: no boosts, no prayers, no
/// void, no gear bonus, wielding the strongest weapon in `WEAPONS` that the
/// attack level allows.
fn attacker_bis(levels: Levels, attack_style: AttackStyle) -> Attacker {
	Attacker {
		strength: levels.strength,
		attack: levels.attack,
		strength_boost: 0,
		attack_boost: 0,
		strength_prayer: StrengthPrayer::None,
		attack_prayer: AttackPrayer::None,
		weapon: strongest_scimitar(levels.attack),
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
	let start = Levels {
		attack: 60,
		strength: 60,
	};
	let path_rune_scim = Path::new::<MAX_LEVEL, _, _>(start, attacker_rune_scim, &ROCK_CRAB);
	let path_bis = Path::new::<MAX_LEVEL, _, _>(start, attacker_bis, &ROCK_CRAB);

	// How does the rune-scim-only optimal path rate when using the BIS weapon?
	let time_rune_scim: f64 = path_rune_scim
		.iter()
		.levels(start)
		.steps(attacker_bis, &ROCK_CRAB)
		.map(Step::time)
		.sum();
	let time_bis: f64 = path_bis
		.iter()
		.levels(start)
		.steps(attacker_bis, &ROCK_CRAB)
		.map(Step::time)
		.sum();

	println!(
		"bis time = {:.1}s
rune scim time = {:.1}s
rune scim path is {:.1}% worse",
		time_bis,
		time_rune_scim,
		100.0 * (time_rune_scim / time_bis - 1.0),
	);
}
