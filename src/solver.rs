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
	pub attack: u8,
	pub strength: u8,
}

/// One step of a leveling path: the skill leveled up, the levels after the
/// level-up, and the DPS the level-up is priced at.
#[derive(Clone, Copy, Debug)]
pub struct Step {
	/// The skill leveled up.
	pub skill: Skill,
	/// The attack level after the level-up.
	pub attack: u8,
	/// The strength level after the level-up.
	pub strength: u8,
	/// The DPS the level-up is priced at: the DPS of the levels before the
	/// level-up, in the skill's training style.
	pub dps: f64,
}

impl Step {
	/// The time spent on this step: the experience gained by the level-up,
	/// priced at the step's DPS. This holds for any valid path through the
	/// grid, not just the solver's own optimum.
	pub fn time(self) -> f64 {
		let level = match self.skill {
			Skill::Attack => self.attack - 1,
			Skill::Strength => self.strength - 1,
		};
		let exp_gain = LEVEL_EXP_TABLE[level as usize] as f64;
		exp_gain / self.dps
	}
}

/// The optimal sequence of skill level-ups from 1/1 to MAX/MAX, in order.
pub struct Path {
	skills: Vec<Skill>,
}

impl Path {
	/// Iterate over the path from 1/1, one [`Step`] per level-up.
	///
	/// Each step is priced at the DPS of `attacker` at the path's current
	/// levels, in the step's training style, so the same path can be priced
	/// with a different `attacker` or `target`.
	pub fn iter<'a, F, T>(&'a self, attacker: F, target: &'a T) -> PathIter<'a, F, T>
	where
		F: Fn(Levels, AttackStyle) -> Attacker,
		T: Target,
	{
		PathIter {
			path: self.skills.iter(),
			attacker,
			target,
			attack: 1,
			strength: 1,
		}
	}
}

/// Iterator over the steps of a leveling path, one [`Step`] per level-up.
pub struct PathIter<'a, F, T> {
	path: core::slice::Iter<'a, Skill>,
	attacker: F,
	target: &'a T,
	/// The path's current levels before the next step; each step raises one
	/// of them by one.
	attack: u8,
	strength: u8,
}

impl<'a, F, T> Iterator for PathIter<'a, F, T>
where
	F: Fn(Levels, AttackStyle) -> Attacker,
	T: Target,
{
	type Item = Step;

	fn next(&mut self) -> Option<Self::Item> {
		let skill = *self.path.next()?;
		let levels = Levels {
			attack: self.attack,
			strength: self.strength,
		};
		let dps = dps_of(&self.attacker, self.target, levels, skill.style());
		match skill {
			Skill::Attack => self.attack += 1,
			Skill::Strength => self.strength += 1,
		};
		Some(Step {
			skill,
			attack: self.attack,
			strength: self.strength,
			dps,
		})
	}
}

/// Find the attack/strength leveling path from 1/1 to MAX/MAX that
/// minimizes the total time, and return it.
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
pub fn solve<const MAX: usize, T: Target, F: Fn(Levels, AttackStyle) -> Attacker>(
	attacker: F,
	target: &T,
) -> Path {
	let levels = |a: usize, s: usize| Levels {
		attack: a as u8,
		strength: s as u8,
	};
	let mut came = [[Skill::Attack; MAX]; MAX];
	let mut dist = [[f64::INFINITY; MAX]; MAX];
	dist[0][0] = 0.0;

	for a in 1..=MAX {
		for s in 1..=MAX {
			let d = dist[a - 1][s - 1];
			for skill in [Skill::Attack, Skill::Strength] {
				let (attack, strength) = match skill {
					Skill::Attack => (a + 1, s),
					Skill::Strength => (a, s + 1),
				};
				if attack > MAX || strength > MAX {
					continue;
				}
				let nd = d + step_time(&attacker, target, levels(a, s), skill);
				if nd < dist[attack - 1][strength - 1] {
					dist[attack - 1][strength - 1] = nd;
					came[attack - 1][strength - 1] = skill;
				}
			}
		}
	}

	let mut skills = Vec::with_capacity(2 * (MAX - 1));
	let (mut a, mut s) = (MAX, MAX);
	while (a, s) != (1, 1) {
		let skill = came[a - 1][s - 1];
		(a, s) = match skill {
			Skill::Attack => (a - 1, s),
			Skill::Strength => (a, s - 1),
		};
		skills.push(skill);
	}
	skills.reverse();
	Path { skills }
}

/// `LEVEL_EXP_TABLE[l]` is the experience needed to go from level `l` to
/// level `l + 1`.
pub const LEVEL_EXP_TABLE: [u32; 99] = [
	0, 83, 91, 102, 112, 124, 138, 151, 168, 185, 204, 226, 249, 274, 304, 335, 369, 408, 450, 497,
	548, 606, 667, 737, 814, 898, 990, 1094, 1207, 1332, 1470, 1623, 1791, 1977, 2182, 2409, 2658,
	2935, 3240, 3576, 3947, 4358, 4810, 5310, 5863, 6471, 7144, 7887, 8707, 9612, 10612, 11715,
	12934, 14278, 15764, 17404, 19214, 21212, 23420, 25856, 28546, 31516, 34795, 38416, 42413,
	46826, 51699, 57079, 63019, 69576, 76818, 84812, 93638, 103383, 114143, 126022, 139138, 153619,
	169608, 187260, 206750, 228269, 252027, 278259, 307221, 339198, 374502, 413482, 456519, 504037,
	556499, 614422, 678376, 748985, 826944, 913019, 1008052, 1112977, 1228825,
];

/// The time to level up `skill` from the given levels: the experience
/// needed for the level-up, divided by the DPS of `attacker` against
/// `target` in the skill's training style at those levels.
fn step_time<T: Target, F: Fn(Levels, AttackStyle) -> Attacker>(
	attacker: &F,
	target: &T,
	levels: Levels,
	skill: Skill,
) -> f64 {
	let level = match skill {
		Skill::Attack => levels.attack,
		Skill::Strength => levels.strength,
	};
	let exp_gain = LEVEL_EXP_TABLE[level as usize] as f64;
	exp_gain / dps_of(attacker, target, levels, skill.style())
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{AttackPrayer, GearBonus, NpcTarget, StrengthPrayer, WEAPONS};

	#[test]
	fn test_exp_table() {
		// `total` is the experience needed to reach level `l` from level 1: a
		// floor of the running sum `sum` of the raw per-level experience.
		// `LEVEL_EXP_TABLE[l]` is the diff of consecutive totals, so each entry
		// can be checked directly as the loop advances the sum.
		let mut sum = 0u32;
		let mut prev_total = 0u32;
		for l in 1..=99 {
			let total = sum / 4;
			assert_eq!(total - prev_total, LEVEL_EXP_TABLE[l - 1]);
			sum += (l as f64 + 300.0 * 2.0f64.powf(l as f64 / 7.0)) as u32;
			prev_total = total;
		}
	}

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
	/// grid, as an independent verification of `solve`'s DP.
	#[test]
	fn solver_matches_brute_force() {
		const MAX: usize = 6;
		let target = test_target();

		fn rec<T: Target, F: Fn(Levels, AttackStyle) -> Attacker>(
			a: usize,
			s: usize,
			max: usize,
			acc: f64,
			attacker: &F,
			target: &T,
			best: &mut f64,
		) {
			if (a, s) == (max, max) {
				*best = best.min(acc);
				return;
			}
			let levels = Levels {
				attack: a as u8,
				strength: s as u8,
			};
			// Mirror `solve`: attack level-ups are priced at the accurate style,
			// strength level-ups at the aggressive style.
			if a < max {
				let dps = dps_of(attacker, target, levels, Skill::Attack.style());
				rec(
					a + 1,
					s,
					max,
					acc + LEVEL_EXP_TABLE[a] as f64 / dps,
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
					acc + LEVEL_EXP_TABLE[s] as f64 / dps,
					attacker,
					target,
					best,
				);
			}
		}
		let mut best = f64::INFINITY;
		rec(1, 1, MAX, 0.0, &test_attacker, &target, &mut best);

		let path = solve::<MAX, _, _>(test_attacker, &target);
		let total: f64 = path.iter(test_attacker, &target).map(Step::time).sum();
		assert!((total - best).abs() < 1e-9);

		// Each step raises exactly one of the current levels by one, and
		// the path ends at MAX/MAX.
		let mut end = (1u8, 1);
		for step in path.iter(test_attacker, &target) {
			assert_eq!(
				(step.attack, step.strength),
				match step.skill {
					Skill::Attack => (end.0 + 1, end.1),
					Skill::Strength => (end.0, end.1 + 1),
				},
			);
			end = (step.attack, step.strength);
		}
		assert_eq!(end, (MAX as u8, MAX as u8));
	}
}
