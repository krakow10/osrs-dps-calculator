use osrs_dps_calculator::{
	AttackPrayer, AttackStyle, Attacker, GameTicks, GearBonus, MeleeDps, StrengthPrayer, Target,
};

fn level_exp(level: u8) -> u32 {
	(1..level)
		.map(|l| {
			let l = l as f64;
			(l + 300.0 * 2.0f64.powf(l / 7.0)) as u32
		})
		.sum::<u32>()
		/ 4
}

fn main() {
	// High-level melee setup: 99/99, +19/+19 boosts, piety, aggressive
	// style, full melee void, ~105 str / ~80 atk equipment bonus,
	// 2-tick (1.2s) attack speed.
	let mut current = Attacker {
		strength: 99,
		attack: 99,
		strength_boost: 19,
		attack_boost: 19,
		strength_prayer: StrengthPrayer::Piety,
		attack_prayer: AttackPrayer::Piety,
		equipment_strength_bonus: 105,
		equipment_attack_bonus: 80,
		attack_style: AttackStyle::Aggressive,
		void: true,
		gear_bonus: GearBonus::None,
		attack_speed: GameTicks(4),
	};

	// PvM: NPC with 40 def and 0 def bonus.
	let target = Target::Npc {
		defence: 40,
		defence_bonus: 0,
	};

	let mut plot = Vec::new();

	let mut dps = MeleeDps::calculate(&current, &target);

	let mut decrement_att = current.clone();
	let mut decrement_str = current.clone();
	loop {
		plot.push((current.attack, current.strength));

		decrement_att.attack = current.attack - 1;
		decrement_att.strength = current.strength;
		decrement_str.attack = current.attack;
		decrement_str.strength = current.strength - 1;

		let dps_att = MeleeDps::calculate(&decrement_att, &target);
		let dps_str = MeleeDps::calculate(&decrement_str, &target);

		// pick whichever increases the most dps per exp
		let exp_att = level_exp(current.attack as u8) - level_exp(decrement_att.attack as u8);
		let exp_str = level_exp(current.strength as u8) - level_exp(decrement_str.strength as u8);

		if (dps.dps - dps_att.dps) / (exp_att as f64) < (dps.dps - dps_str.dps) / (exp_str as f64) {
			// the increase in strength dps per exp was larger, decrement attack to lose less dps
			dps = dps_att;
			core::mem::swap(&mut current, &mut decrement_att);
		} else {
			dps = dps_str;
			core::mem::swap(&mut current, &mut decrement_str);
		}

		if current.attack == 1 || current.strength == 1 {
			break;
		}
	}

	plot.reverse();
	for (att, str) in plot {
		println!("att={att:02} str={str:02}");
	}
}
