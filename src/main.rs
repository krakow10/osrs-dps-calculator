use osrs_dps_calculator::{
	AttackPrayer, AttackStyle, Attacker, GearBonus, MeleeDps, NpcTarget, RUNE_SCIMITAR,
	StrengthPrayer, Target,
};

/// Highest level we care about for attack and strength.
const MAX_LEVEL: u32 = 99;

/// A skill to level up. Each can only be trained with one attack style.
#[derive(Clone, Copy, Debug)]
enum Skill {
	/// Attack, trained with the Accurate style.
	Attack,
	/// Strength, trained with the Aggressive style.
	Strength,
}

impl Skill {
	/// The attack style used to train this skill.
	fn style(self) -> AttackStyle {
		match self {
			Skill::Attack => AttackStyle::Accurate,
			Skill::Strength => AttackStyle::Aggressive,
		}
	}

	/// The display name of the style used to train this skill.
	fn style_name(self) -> &'static str {
		match self {
			Skill::Attack => "accurate",
			Skill::Strength => "aggressive",
		}
	}
}

/// The attacker's attack and strength levels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Levels {
	attack: u32,
	strength: u32,
}

/// One level-up on the optimal path: the levels after the level-up, which
/// skill was leveled up, the time that took, and the cumulative time from
/// the start.
#[derive(Clone, Copy, Debug)]
struct Step {
	levels: Levels,
	skill: Skill,
	time: f64,
	total: f64,
}

/// `t[l]` is the total experience needed to reach level `l` from level 1.
fn level_exp_table() -> [u32; 100] {
	let mut t = [0u32; 100];
	let mut sum = 0u32;
	for l in 1..=99 {
		t[l] = sum / 4;
		sum += (l as f64 + 300.0 * 2.0f64.powf(l as f64 / 7.0)) as u32;
	}
	t
}

/// DPS at the given levels and attack style, keeping the rest of the
/// attacker setup and target fixed.
fn dps_of<T: Target>(attacker: &Attacker, target: &T, levels: Levels, style: AttackStyle) -> f64 {
	let atk = Attacker {
		attack: levels.attack,
		strength: levels.strength,
		attack_style: style,
		..*attacker
	};
	MeleeDps::calculate(&atk, target).dps
}

/// Find the attack/strength leveling path from 1/1 to max/max that
/// minimizes the total time, where the time spent on each level-up is
/// proportional to the exp needed for that level divided by the DPS at
/// the state you're in while leveling it up.
///
/// The attack style used while leveling matters: attack can only be
/// leveled with the Accurate style and strength with the Aggressive style,
/// so each level-up edge is priced at the DPS of the style that trains it.
///
/// The leveling graph is a DAG (edges only increase attack or strength),
/// so a topological-order DP finds the optimum exactly.
///
/// Returns the level-ups from start to goal, in order.
fn solve<T: Target>(attacker: &Attacker, target: &T, max: u32) -> Vec<Step> {
	let n = max as usize;
	let idx = |a: u32, s: u32| (a - 1) as usize * n + (s - 1) as usize;
	let level_exp = level_exp_table();
	let exp_gain = |l: u32| (level_exp[(l + 1) as usize] - level_exp[l as usize]) as f64;
	let levels = |a: u32, s: u32| Levels { attack: a, strength: s };

	let mut dist = vec![f64::INFINITY; n * n];
	// The skill leveled up to reach each node.
	let mut came = vec![Skill::Attack; n * n];
	dist[idx(1, 1)] = 0.0;

	for a in 1..=max {
		for s in 1..=max {
			let i = idx(a, s);
			let d = dist[i];
			for skill in [Skill::Attack, Skill::Strength] {
				let (attack, strength, level) = match skill {
					Skill::Attack => (a + 1, s, a),
					Skill::Strength => (a, s + 1, s),
				};
				if attack > max || strength > max {
					continue;
				}
				let j = idx(attack, strength);
				let nd = d + exp_gain(level) / dps_of(attacker, target, levels(a, s), skill.style());
				if nd < dist[j] {
					dist[j] = nd;
					came[j] = skill;
				}
			}
		}
	}

	// Reconstruct the path by walking back from the goal.
	let mut path = Vec::with_capacity(2 * (n - 1));
	let (mut a, mut s) = (max, max);
	while (a, s) != (1, 1) {
		let i = idx(a, s);
		let skill = came[i];
		let prev = match skill {
			Skill::Attack => idx(a - 1, s),
			Skill::Strength => idx(a, s - 1),
		};
		path.push(Step {
			levels: levels(a, s),
			skill,
			time: dist[i] - dist[prev],
			total: dist[i],
		});
		(a, s) = match skill {
			Skill::Attack => (a - 1, s),
			Skill::Strength => (a, s - 1),
		};
	}
	path.reverse();
	path
}

/// Print the path one line per level-up, showing the step time, the
/// cumulative time, and the style (and DPS) the step was priced at.
fn print_path<T: Target>(attacker: &Attacker, target: &T, path: &[Step]) {
	println!("Optimal leveling path 1/1 -> {MAX_LEVEL}/{MAX_LEVEL} (minimizes sum of exp / dps):");
	for step in path {
		// The step was taken from the state before leveling its skill, which
		// is where the style's DPS was measured.
		let prev = match step.skill {
			Skill::Attack => Levels { attack: step.levels.attack - 1, ..step.levels },
			Skill::Strength => Levels { strength: step.levels.strength - 1, ..step.levels },
		};
		let dps = dps_of(attacker, target, prev, step.skill.style());
		println!(
			"att={:02} str={:02}  step={:>12.4}  total={:>12.4}  {:>10} dps={:.4}",
			step.levels.attack,
			step.levels.strength,
			step.time,
			step.total,
			step.skill.style_name(),
			dps
		);
	}
	println!(
		"Total time: {:.4}",
		path.last().expect("path is non-empty").total
	);
}

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
		void: true,
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

	let path = solve(&attacker, &target, MAX_LEVEL);
	print_path(&attacker, &target, &path);
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Exhaustive check of the optimum over every monotone path on a small
	/// grid, as an independent verification of `solve`'s DP.
	#[test]
	fn solve_matches_brute_force() {
		const MAX: u32 = 6;
		let attacker = base_attacker();
		let target = test_target();

		let path = solve(&attacker, &target, MAX);
		let total = path.last().expect("path is non-empty").total;

		fn rec<T: Target>(
			a: u32,
			s: u32,
			max: u32,
			acc: f64,
			level_exp: &[u32; 100],
			attacker: &Attacker,
			target: &T,
			best: &mut f64,
		) {
			if (a, s) == (max, max) {
				*best = best.min(acc);
				return;
			}
			let levels = Levels { attack: a, strength: s };
			let exp_gain = |l: u32| (level_exp[(l + 1) as usize] - level_exp[l as usize]) as f64;
			// Mirror `solve`: attack level-ups are priced at the accurate style,
			// strength level-ups at the aggressive style.
			if a < max {
				let dps = dps_of(attacker, target, levels, Skill::Attack.style());
				rec(a + 1, s, max, acc + exp_gain(a) / dps, level_exp, attacker, target, best);
			}
			if s < max {
				let dps = dps_of(attacker, target, levels, Skill::Strength.style());
				rec(a, s + 1, max, acc + exp_gain(s) / dps, level_exp, attacker, target, best);
			}
		}
		let level_exp = level_exp_table();
		let mut best = f64::INFINITY;
		rec(1, 1, MAX, 0.0, &level_exp, &attacker, &target, &mut best);
		assert!((total - best).abs() < 1e-9);

		// Each step raises exactly one skill by one, `total` is the running
		// sum of `time` from the 1/1 start, and the path ends at MAX/MAX.
		let mut running = 0.0;
		let mut prev = Levels { attack: 1, strength: 1 };
		for step in &path {
			let da = step.levels.attack as i32 - prev.attack as i32;
			let ds = step.levels.strength as i32 - prev.strength as i32;
			assert!(
				(da, ds) == (1, 0) || (da, ds) == (0, 1),
				"step from ({}, {}) to ({}, {}) is not a single level-up",
				prev.attack,
				prev.strength,
				step.levels.attack,
				step.levels.strength
			);
			assert!((step.total - running - step.time).abs() < 1e-9);
			running = step.total;
			prev = step.levels;
		}
		assert_eq!(prev, Levels { attack: MAX, strength: MAX });
	}
}
