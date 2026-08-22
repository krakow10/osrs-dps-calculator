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

impl Levels {
	/// The level of `skill` in these levels.
	pub fn skill_level(self, skill: Skill) -> u8 {
		match skill {
			Skill::Attack => self.attack,
			Skill::Strength => self.strength,
		}
	}
}

/// One level-up of a path, priced: the skill leveled up, the levels before
/// the level-up, and the experience gained per second the level-up is
/// priced at.
#[derive(Clone, Copy, Debug)]
pub struct Step {
	/// The skill leveled up.
	pub skill: Skill,
	/// The levels before the level-up.
	pub levels: Levels,
	/// The experience gained per second the level-up is priced at: the
	/// experience gained per second of `levels`, in the skill's training
	/// style.
	pub exp_per_second: f64,
}

impl Step {
	/// The time spent on this level-up: the experience needed for it, priced
	/// at the step's experience gained per second. This holds for any valid
	/// path through the grid, not just the solver's own optimum.
	pub fn time(self) -> f64 {
		let exp_gain = LEVEL_EXP_TABLE[self.levels.skill_level(self.skill) as usize] as f64;
		exp_gain / self.exp_per_second
	}
}

/// The optimal sequence of skill level-ups from 1/1 to MAX/MAX, in order.
pub struct Path {
	skills: Vec<Skill>,
}

impl Path {
	/// Find the attack/strength leveling path from 1/1 to MAX/MAX that
	/// minimizes the total time, and return it.
	///
	/// The time spent on each level-up is the exp needed for that level
	/// divided by the experience gained per second at the state you're in
	/// while leveling it up. The attack style used while leveling matters:
	/// attack
	/// can only be leveled with the Accurate style and strength with the
	/// Aggressive style, so each level-up edge is priced at the experience
	/// gained per second of the style that trains it.
	///
	/// The leveling graph is a DAG (edges only increase attack or
	/// strength), so a topological-order DP finds the optimum exactly.
	pub fn new<const MAX: usize, F: Fn(Levels, AttackStyle) -> Attacker, T: Target>(
		attacker: F,
		target: &T,
	) -> Self {
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
					let levels = levels(a, s);
					let exp_per_second =
						exp_per_second_of(&attacker, target, levels, skill.style());
					let nd = d + Step {
						skill,
						levels,
						exp_per_second,
					}
					.time();
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

	/// Iterate over the path from 1/1, one [`Skill`] per level-up, in order.
	///
	/// Chain [`PathIter::levels`] to pair each level-up with the levels
	/// before it, then [`LevelsIter::steps`] to price each at the experience
	/// gained per second and derive times.
	pub fn iter(&self) -> PathIter<'_> {
		PathIter(self.skills.iter())
	}
}

/// An iterator over a path's level-ups, one [`Skill`] per level-up, created
/// by [`Path::iter`].
pub struct PathIter<'a>(core::slice::Iter<'a, Skill>);

impl<'a> Iterator for PathIter<'a> {
	type Item = Skill;

	fn next(&mut self) -> Option<Skill> {
		self.0.next().copied()
	}
}

impl<'a> PathIter<'a> {
	/// Pair each level-up with the levels before it.
	pub fn levels(self) -> LevelsIter<'a> {
		LevelsIter {
			path: self.0,
			levels: Levels {
				attack: 1,
				strength: 1,
			},
		}
	}
}

/// An iterator over a path's level-ups, one (`Skill`, `Levels`) pair per
/// level-up: the skill leveled up and the levels before the level-up, created
/// by [`PathIter::levels`].
pub struct LevelsIter<'a> {
	path: core::slice::Iter<'a, Skill>,
	/// The path's current levels before the next level-up; each level-up
	/// raises one of them by one.
	levels: Levels,
}

impl<'a> Iterator for LevelsIter<'a> {
	type Item = (Skill, Levels);

	fn next(&mut self) -> Option<(Skill, Levels)> {
		let skill = *self.path.next()?;
		let levels = self.levels;
		match skill {
			Skill::Attack => self.levels.attack += 1,
			Skill::Strength => self.levels.strength += 1,
		}
		Some((skill, levels))
	}
}

impl<'a> LevelsIter<'a> {
	/// Price each level-up at the experience gained per second of `attacker`
	/// against `target` at the levels before it, in the skill's training
	/// style, yielding one [`Step`] per level-up. The same path can be priced
	/// with a different `attacker` or `target`.
	pub fn steps<'t, F, T>(self, attacker: F, target: &'t T) -> StepIter<'a, 't, F, T>
	where
		F: Fn(Levels, AttackStyle) -> Attacker,
		T: Target,
	{
		StepIter {
			path: self,
			attacker,
			target,
		}
	}
}

/// An iterator over a path's priced level-ups, one [`Step`] per level-up,
/// created by [`LevelsIter::steps`].
pub struct StepIter<'a, 't, F, T> {
	path: LevelsIter<'a>,
	attacker: F,
	target: &'t T,
}

impl<'a, 't, F, T> Iterator for StepIter<'a, 't, F, T>
where
	F: Fn(Levels, AttackStyle) -> Attacker,
	T: Target,
{
	type Item = Step;

	fn next(&mut self) -> Option<Step> {
		let (skill, levels) = self.path.next()?;
		let exp_per_second = exp_per_second_of(&self.attacker, self.target, levels, skill.style());
		Some(Step {
			skill,
			levels,
			exp_per_second,
		})
	}
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

/// The normal experience gained per damage in melee combat: 4 experience
/// per damage, so the experience gained per second is the DPS times this.
/// <https://oldschool.runescape.wiki/w/Combat#Experience_gain>
pub const EXP_PER_DAMAGE: f64 = 4.0;

/// The experience gained per second of the attacker at the given levels
/// and attack style, against the target: the DPS times the normal
/// experience gained per damage.
fn exp_per_second_of<T: Target, F: Fn(Levels, AttackStyle) -> Attacker>(
	attacker: &F,
	target: &T,
	levels: Levels,
	style: AttackStyle,
) -> f64 {
	MeleeDps::calculate(&attacker(levels, style), target).dps * EXP_PER_DAMAGE
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
	/// grid, as an independent verification of `Path::new`'s DP.
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
			// Mirror `Path::new`: attack level-ups are priced at the accurate style,
			// strength level-ups at the aggressive style.
			if a < max {
				let exp_per_second =
					exp_per_second_of(attacker, target, levels, Skill::Attack.style());
				rec(
					a + 1,
					s,
					max,
					acc + LEVEL_EXP_TABLE[a] as f64 / exp_per_second,
					attacker,
					target,
					best,
				);
			}
			if s < max {
				let exp_per_second =
					exp_per_second_of(attacker, target, levels, Skill::Strength.style());
				rec(
					a,
					s + 1,
					max,
					acc + LEVEL_EXP_TABLE[s] as f64 / exp_per_second,
					attacker,
					target,
					best,
				);
			}
		}
		let mut best = f64::INFINITY;
		rec(1, 1, MAX, 0.0, &test_attacker, &target, &mut best);

		let path = Path::new::<MAX, _, _>(test_attacker, &target);
		let total: f64 = path
			.iter()
			.levels()
			.steps(test_attacker, &target)
			.map(Step::time)
			.sum();
		assert!((total - best).abs() < 1e-9);

		// Replay the path from 1/1: each level-up is priced from the current
		// levels, and the path ends at MAX/MAX. No attacker or target needed
		// for this.
		let mut current = Levels {
			attack: 1,
			strength: 1,
		};
		for (skill, levels) in path.iter().levels() {
			assert_eq!(levels, current);
			match skill {
				Skill::Attack => current.attack += 1,
				Skill::Strength => current.strength += 1,
			}
		}
		assert_eq!(
			current,
			Levels {
				attack: MAX as u8,
				strength: MAX as u8,
			},
		);
	}
}
