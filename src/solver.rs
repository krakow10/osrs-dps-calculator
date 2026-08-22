use crate::{AttackStyle, Attacker, MeleeDps, Target};

/// A skill to level up. Each can only be trained with one attack style.
#[derive(Clone, Copy, Debug)]
pub enum Skill {
	/// Attack, trained with the Accurate style.
	Attack,
	/// Strength, trained with the Aggressive style.
	Strength,
}

impl Skill {
	/// The attack style used to train this skill.
	pub fn style(self) -> AttackStyle {
		match self {
			Skill::Attack => AttackStyle::Accurate,
			Skill::Strength => AttackStyle::Aggressive,
		}
	}

	/// The display name of the style used to train this skill.
	pub fn style_name(self) -> &'static str {
		match self {
			Skill::Attack => "accurate",
			Skill::Strength => "aggressive",
		}
	}
}

/// The attacker's attack and strength levels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Levels {
	pub attack: u32,
	pub strength: u32,
}

/// A node in the solved grid.
#[derive(Clone, Copy, Debug)]
pub struct GridPoint {
	/// The skill leveled up to reach this node.
	pub came: Skill,
	/// The minimum total time to reach this node from 1/1.
	pub dist: f64,
	/// The DPS at which the level-up into this node was priced.
	pub dps: f64,
}

/// The solved leveling grid: the minimum total time to reach each
/// (attack, strength) node from 1/1, the skill that was leveled up to
/// reach it, and the DPS the level-up was priced at.
pub struct Solver<const MAX: usize> {
	/// `points[a - 1][s - 1]` is node (a, s).
	points: [[GridPoint; MAX]; MAX],
}

/// One step of a leveling path: the grid point the level-up lands on and
/// the destination's levels.
#[derive(Clone, Copy, Debug)]
pub struct Step<'a> {
	/// The skill leveled up.
	pub skill: Skill,
	/// The attack level after the level-up.
	pub attack: u32,
	/// The strength level after the level-up.
	pub strength: u32,
	/// The point the level-up lands on.
	pub point: &'a GridPoint,
}

/// Iterator over the steps of a leveling path, one [`Step`] per level-up.
pub struct GridPointIter<'a, const MAX: usize> {
	solver: &'a Solver<MAX>,
	path: core::slice::Iter<'a, Skill>,
	/// The path's current levels; each step raises one of them by one.
	attack: usize,
	strength: usize,
}

impl<const MAX: usize> Solver<MAX> {
	/// Find the attack/strength leveling path from 1/1 to MAX/MAX that
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
	pub fn new<T: Target, F: Fn(Levels, AttackStyle) -> Attacker>(attacker: F, target: &T) -> Self {
		let level_exp = level_exp_table();
		let exp_gain = |l: usize| (level_exp[l + 1] - level_exp[l]) as f64;
		let levels = |a: usize, s: usize| Levels {
			attack: a as u32,
			strength: s as u32,
		};
		let mut solver = Solver {
			points: [[GridPoint {
				came: Skill::Attack,
				dist: f64::INFINITY,
				dps: 0.0,
			}; MAX]; MAX],
		};
		solver.points[0][0].dist = 0.0;

		for a in 1..=MAX {
			for s in 1..=MAX {
				let d = solver.points[a - 1][s - 1].dist;
				for skill in [Skill::Attack, Skill::Strength] {
					let (attack, strength, level) = match skill {
						Skill::Attack => (a + 1, s, a),
						Skill::Strength => (a, s + 1, s),
					};
					if attack > MAX || strength > MAX {
						continue;
					}
					let dps = dps_of(&attacker, target, levels(a, s), skill.style());
					let nd = d + exp_gain(level) / dps;
					let dest = &mut solver.points[attack - 1][strength - 1];
					if nd < dest.dist {
						dest.dist = nd;
						dest.came = skill;
						dest.dps = dps;
					}
				}
			}
		}

		solver
	}

	/// The optimal sequence of skill level-ups from 1/1 to MAX/MAX, in order.
	pub fn path(&self) -> Vec<Skill> {
		let mut path = Vec::with_capacity(2 * (MAX - 1));
		let (mut a, mut s) = (MAX, MAX);
		while (a, s) != (1, 1) {
			let skill = self.points[a - 1][s - 1].came;
			(a, s) = match skill {
				Skill::Attack => (a - 1, s),
				Skill::Strength => (a, s - 1),
			};
			path.push(skill);
		}
		path.reverse();
		path
	}

	/// Iterate over the steps of `path` from 1/1, one [`Step`] per level-up.
	///
	/// `path` must be a valid 1/1 -> MAX/MAX path, such as the one returned
	/// by [`Solver::path`].
	pub fn iter<'a>(&'a self, path: &'a [Skill]) -> GridPointIter<'a, MAX> {
		GridPointIter {
			solver: self,
			path: path.iter(),
			attack: 1,
			strength: 1,
		}
	}
}

impl<'a, const MAX: usize> Iterator for GridPointIter<'a, MAX> {
	type Item = Step<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let skill = *self.path.next()?;
		match skill {
			Skill::Attack => self.attack += 1,
			Skill::Strength => self.strength += 1,
		};
		let point = &self.solver.points[self.attack - 1][self.strength - 1];
		Some(Step {
			skill,
			attack: self.attack as u32,
			strength: self.strength as u32,
			point,
		})
	}
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

/// The DPS of the attacker at the given levels and attack style, against
/// the target.
fn dps_of<T: Target, F: Fn(Levels, AttackStyle) -> Attacker>(
	attacker: &F,
	target: &T,
	levels: Levels,
	style: AttackStyle,
) -> f64 {
	MeleeDps::calculate(&attacker(levels, style), target).dps
}

/// Print the path one line per level-up, showing the step time, the
/// cumulative time, and the style (and DPS) the step was priced at. The
/// per-step values are read straight from the grid's points.
pub fn print_path<const MAX: usize>(solver: &Solver<MAX>, path: &[Skill]) {
	println!("Optimal leveling path 1/1 -> {MAX}/{MAX} (minimizes sum of exp / dps):");
	let mut total = 0.0f64;
	for step in solver.iter(path) {
		// The step's dist and dps are stored on the destination point; the
		// DPS was measured in the state before leveling its skill.
		let time = step.point.dist - total;
		total = step.point.dist;
		let dps = step.point.dps;

		println!(
			"att={:02} str={:02}  step={:>12.4}  total={:>12.4}  {:>10} dps={:.4}",
			step.attack,
			step.strength,
			time,
			total,
			step.skill.style_name(),
			dps
		);
	}
	println!("Total time: {:.4}", total);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{AttackPrayer, GearBonus, NpcTarget, StrengthPrayer, WEAPONS};

	/// The test attacker: the fixed setup, wielding the strongest weapon in
	/// `WEAPONS` that the attack level allows.
	fn test_attacker(levels: Levels, style: AttackStyle) -> Attacker {
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

	/// Exhaustive check of the optimum over every monotone path on a small
	/// grid, as an independent verification of `Solver::new`'s DP.
	#[test]
	fn solver_matches_brute_force() {
		const MAX: usize = 6;
		let target = test_target();

		let solver = Solver::<MAX>::new(test_attacker, &target);
		let total = solver.points[MAX - 1][MAX - 1].dist;

		fn rec<T: Target, F: Fn(Levels, AttackStyle) -> Attacker>(
			a: usize,
			s: usize,
			max: usize,
			acc: f64,
			level_exp: &[u32; 100],
			attacker: &F,
			target: &T,
			best: &mut f64,
		) {
			if (a, s) == (max, max) {
				*best = best.min(acc);
				return;
			}
			let levels = Levels {
				attack: a as u32,
				strength: s as u32,
			};
			let exp_gain = |l: usize| (level_exp[l + 1] - level_exp[l]) as f64;
			// Mirror `Solver::new`: attack level-ups are priced at the accurate style,
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
		rec(
			1,
			1,
			MAX,
			0.0,
			&level_exp,
			&test_attacker,
			&target,
			&mut best,
		);
		assert!((total - best).abs() < 1e-9);

		// Each step raises exactly one skill by one, `total` is the running
		// sum of `time` from the 1/1 start, and the path ends at MAX/MAX.
		let path = solver.path();
		let mut end = (1u32, 1);
		let mut it = solver.iter(&path);
		let mut last = it.next().unwrap();
		let mut running = last.point.dist;
		for step in it {
			let step_total = step.point.dist;
			let step_time = step.point.dist - last.point.dist;
			assert!((step_total - running - step_time).abs() < 1e-9);
			running = step_total;
			last = step;
			end = (step.attack, step.strength);
		}
		assert_eq!(end, (MAX as u32, MAX as u32));
	}
}
