use osrs_dps_calculator::{
	AttackPrayer, AttackStyle, Attacker, GearBonus, MeleeDps, NpcTarget, RUNE_SCIMITAR,
	StrengthPrayer, Target, WEAPONS, WeaponStats,
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

/// A node in the solved grid.
#[derive(Clone, Copy, Debug)]
struct GridPoint {
	/// The skill leveled up to reach this node.
	came: Skill,
	/// The minimum total time to reach this node from 1/1.
	dist: f64,
	/// The DPS at which the level-up into this node was priced.
	dps: f64,
}

/// The solved leveling grid: the minimum total time to reach each
/// (attack, strength) node from 1/1, the skill that was leveled up to
/// reach it, and the DPS the level-up was priced at.
struct Grid {
	max: u32,
	/// `points[idx(a, s)]` is node (a, s).
	points: Vec<GridPoint>,
}

impl Grid {
	/// Find the attack/strength leveling path from 1/1 to max/max that
	/// minimizes the total time, and return the solved grid.
	///
	/// The time spent on each level-up is proportional to the exp needed
	/// for that level divided by the DPS at the state you're in while
	/// leveling it up. The attack style used while leveling matters: attack
	/// can only be leveled with the Accurate style and strength with the
	/// Aggressive style, so each level-up edge is priced at the DPS of the
	/// style that trains it.
	///
	/// The leveling graph is a DAG (edges only increase attack or
	/// strength), so a topological-order DP finds the optimum exactly.
	fn new<T: Target>(attacker: &Attacker, target: &T, max: u32) -> Self {
		let level_exp = level_exp_table();
		let exp_gain = |l: u32| (level_exp[(l + 1) as usize] - level_exp[l as usize]) as f64;
		let levels = |a: u32, s: u32| Levels {
			attack: a,
			strength: s,
		};
		let n = max as usize;
		let mut points = vec![
			GridPoint {
				came: Skill::Attack,
				dist: f64::INFINITY,
				dps: 0.0,
			};
			n * n
		];
		points[0].dist = 0.0;
		let mut grid = Grid { max, points };

		for a in 1..=max {
			for s in 1..=max {
				let i = grid.idx(a, s);
				let d = grid.points[i].dist;
				for skill in [Skill::Attack, Skill::Strength] {
					let (attack, strength, level) = match skill {
						Skill::Attack => (a + 1, s, a),
						Skill::Strength => (a, s + 1, s),
					};
					if attack > max || strength > max {
						continue;
					}
					let j = grid.idx(attack, strength);
					let dps = dps_of(attacker, target, levels(a, s), skill.style());
					let nd = d + exp_gain(level) / dps;
					if nd < grid.points[j].dist {
						grid.points[j].dist = nd;
						grid.points[j].came = skill;
						grid.points[j].dps = dps;
					}
				}
			}
		}

		grid
	}

	/// The flat index of the (attack, strength) node.
	fn idx(&self, attack: u32, strength: u32) -> usize {
		let n = self.max as usize;
		(attack - 1) as usize * n + (strength - 1) as usize
	}

	/// The optimal sequence of skill level-ups from 1/1 to max/max, in order.
	fn path(&self) -> Vec<Skill> {
		let mut path = Vec::with_capacity(2 * (self.max as usize - 1));
		let (mut a, mut s) = (self.max, self.max);
		while (a, s) != (1, 1) {
			let skill = self.points[self.idx(a, s)].came;
			(a, s) = match skill {
				Skill::Attack => (a - 1, s),
				Skill::Strength => (a, s - 1),
			};
			path.push(skill);
		}
		path.reverse();
		path
	}
}

/// The strongest weapon in `WEAPONS` that can be wielded at the given
/// attack level, so the weapon switches out as the attacker levels up
/// attack and reaches each weapon's requirement.
fn weapon_for_attack(attack: u32) -> WeaponStats {
	WEAPONS
		.iter()
		.rev()
		.find(|(_, min_attack)| attack >= *min_attack)
		.map(|(stats, _)| *stats)
		.expect("WEAPONS always contains a weapon with no level requirement")
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
/// attacker setup and target fixed. The wielded weapon is whatever the
/// attacker can use at that attack level, so it switches out as each
/// weapon's requirement is reached.
fn dps_of<T: Target>(attacker: &Attacker, target: &T, levels: Levels, style: AttackStyle) -> f64 {
	let atk = Attacker {
		attack: levels.attack,
		strength: levels.strength,
		weapon: weapon_for_attack(levels.attack),
		attack_style: style,
		..*attacker
	};
	MeleeDps::calculate(&atk, target).dps
}

/// Print the path one line per level-up, showing the step time, the
/// cumulative time, and the style (and DPS) the step was priced at. The
/// per-step values are read straight from the grid's points.
fn print_path(grid: &Grid, path: &[Skill]) {
	println!("Optimal leveling path 1/1 -> {MAX_LEVEL}/{MAX_LEVEL} (minimizes sum of exp / dps):");
	let mut a = 1u32;
	let mut s = 1u32;
	let mut total = 0.0f64;
	for &skill in path {
		// The step's dist and dps are stored on the destination point; the
		// DPS was measured in the state before leveling its skill.
		(a, s) = match skill {
			Skill::Attack => (a + 1, s),
			Skill::Strength => (a, s + 1),
		};
		let i = grid.idx(a, s);
		let prev_i = match skill {
			Skill::Attack => grid.idx(a - 1, s),
			Skill::Strength => grid.idx(a, s - 1),
		};
		let time = grid.points[i].dist - grid.points[prev_i].dist;
		total = grid.points[i].dist;
		let dps = grid.points[i].dps;

		println!(
			"att={:02} str={:02}  step={:>12.4}  total={:>12.4}  {:>10} dps={:.4}",
			a,
			s,
			time,
			total,
			skill.style_name(),
			dps
		);
	}
	println!("Total time: {:.4}", total);
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

	let grid = Grid::new(&attacker, &target, MAX_LEVEL);
	let path = grid.path();
	print_path(&grid, &path);
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Exhaustive check of the optimum over every monotone path on a small
	/// grid, as an independent verification of `Grid::new`'s DP.
	#[test]
	fn grid_matches_brute_force() {
		const MAX: u32 = 6;
		let attacker = base_attacker();
		let target = test_target();

		let grid = Grid::new(&attacker, &target, MAX);
		let total = grid.points[grid.idx(MAX, MAX)].dist;

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
			let levels = Levels {
				attack: a,
				strength: s,
			};
			let exp_gain = |l: u32| (level_exp[(l + 1) as usize] - level_exp[l as usize]) as f64;
			// Mirror `Grid::new`: attack level-ups are priced at the accurate style,
			// strength level-ups at the aggressive style.
			if a < max {
				let dps = dps_of(attacker, target, levels, Skill::Attack.style());
				rec(
					a + 1,
					s,
					max,
					acc + exp_gain(a) / dps,
					level_exp,
					attacker,
					target,
					best,
				);
			}
			if s < max {
				let dps = dps_of(attacker, target, levels, Skill::Strength.style());
				rec(
					a,
					s + 1,
					max,
					acc + exp_gain(s) / dps,
					level_exp,
					attacker,
					target,
					best,
				);
			}
		}
		let level_exp = level_exp_table();
		let mut best = f64::INFINITY;
		rec(1, 1, MAX, 0.0, &level_exp, &attacker, &target, &mut best);
		assert!((total - best).abs() < 1e-9);

		// Each step raises exactly one skill by one, `total` is the running
		// sum of `time` from the 1/1 start, and the path ends at MAX/MAX.
		let mut running = 0.0;
		let mut a = 1u32;
		let mut s = 1u32;
		for skill in grid.path() {
			(a, s) = match skill {
				Skill::Attack => (a + 1, s),
				Skill::Strength => (a, s + 1),
			};
			let i = grid.idx(a, s);
			let prev_i = match skill {
				Skill::Attack => grid.idx(a - 1, s),
				Skill::Strength => grid.idx(a, s - 1),
			};
			let step_total = grid.points[i].dist;
			let step_time = grid.points[i].dist - grid.points[prev_i].dist;
			assert!((step_total - running - step_time).abs() < 1e-9);
			running = step_total;
		}
		assert_eq!((a, s), (MAX, MAX));
	}

	/// The weapon switches out exactly when the attacker reaches each
	/// weapon's required attack level.
	#[test]
	fn weapon_switches_at_required_attack_level() {
		use osrs_dps_calculator::{
			ABYSSAL_WHIP, ADAMANT_SCIMITAR, BLACK_SCIMITAR, BLADE_OF_SAELDOR, DRAGON_SCIMITAR,
			IRON_SCIMITAR, MITHRIL_SCIMITAR, RUNE_SCIMITAR, STEEL_SCIMITAR,
		};
		// Iron scimitar at 1, until 5 attack.
		assert_eq!(weapon_for_attack(1), IRON_SCIMITAR);
		assert_eq!(weapon_for_attack(4), IRON_SCIMITAR);
		// Steel scimitar at 5, until 10 attack.
		assert_eq!(weapon_for_attack(5), STEEL_SCIMITAR);
		assert_eq!(weapon_for_attack(9), STEEL_SCIMITAR);
		// Black scimitar at 10, until 20 attack.
		assert_eq!(weapon_for_attack(10), BLACK_SCIMITAR);
		assert_eq!(weapon_for_attack(19), BLACK_SCIMITAR);
		// Mithril scimitar at 20, until 30 attack.
		assert_eq!(weapon_for_attack(20), MITHRIL_SCIMITAR);
		assert_eq!(weapon_for_attack(29), MITHRIL_SCIMITAR);
		// Adamant scimitar at 30, until 40 attack.
		assert_eq!(weapon_for_attack(30), ADAMANT_SCIMITAR);
		assert_eq!(weapon_for_attack(39), ADAMANT_SCIMITAR);
		// Rune scimitar at 40, until 60 attack.
		assert_eq!(weapon_for_attack(40), RUNE_SCIMITAR);
		assert_eq!(weapon_for_attack(59), RUNE_SCIMITAR);
		// Dragon scimitar at 60, until 70 attack.
		assert_eq!(weapon_for_attack(60), DRAGON_SCIMITAR);
		assert_eq!(weapon_for_attack(69), DRAGON_SCIMITAR);
		// Abyssal whip at 70, until 80 attack.
		assert_eq!(weapon_for_attack(70), ABYSSAL_WHIP);
		assert_eq!(weapon_for_attack(79), ABYSSAL_WHIP);
		// Blade of Saeldor at 80.
		assert_eq!(weapon_for_attack(80), BLADE_OF_SAELDOR);
		assert_eq!(weapon_for_attack(99), BLADE_OF_SAELDOR);
	}
}
