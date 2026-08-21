use osrs_dps_calculator::{
	AttackPrayer, AttackStyle, Attacker, GearBonus, NpcTarget, RUNE_SCIMITAR, StrengthPrayer,
	grid::{Grid, print_path},
};

/// Highest level we care about for attack and strength.
const MAX_LEVEL: usize = 99;

fn base_attacker() -> Attacker {
	// High-level melee setup: 99/99, full melee void, 44 str / 45 atk
	// equipment bonus, 4-tick (2.4s) attack speed. The aggressive style is
	// just the default; `dps_of` picks the style that trains each skill.
	Attacker {
		strength: 99,
		attack: 99,
		strength_boost: 0,
		attack_boost: 0,
		strength_prayer: StrengthPrayer::None,
		attack_prayer: AttackPrayer::None,
		weapon: RUNE_SCIMITAR,
		attack_style: AttackStyle::Aggressive,
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
	let attacker = base_attacker();
	let target = test_target();

	let grid = Grid::<MAX_LEVEL>::new(&attacker, &target);
	let path = grid.path();
	print_path(&grid, &path);
}
