use osrs_dps_calculator::{
	ABYSSAL_WHIP, AttackPrayer, AttackStyle, Attacker, GearBonus, MeleeDps, NpcTarget,
	StrengthPrayer, Target,
};

/// Highest level we care about for attack and strength.
const MAX_LEVEL: u32 = 99;

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

/// DPS at a given (attack, strength), keeping the rest of the attacker
/// setup and target fixed.
fn dps_of<T: Target>(attacker: &Attacker, target: &T, attack: u32, strength: u32) -> f64 {
	let mut atk = *attacker;
	atk.attack = attack;
	atk.strength = strength;
	MeleeDps::calculate(&atk, target).dps
}

/// Find the attack/strength leveling path from 1/1 to max/max that
/// minimizes the total time, where the time spent on each level-up is
/// proportional to the exp needed for that level divided by the DPS at
/// the state you're in while leveling it up.
///
/// The leveling graph is a DAG (edges only increase attack or strength),
/// so a topological-order DP finds the optimum exactly.
///
/// Returns the path (start to goal) with the cumulative time at each node.
fn solve<T: Target>(attacker: &Attacker, target: &T, max: u32) -> Vec<(u32, u32, f64)> {
	let n = max as usize;
	let idx = |a: u32, s: u32| (a - 1) as usize * n + (s - 1) as usize;
	let level_exp = level_exp_table();
	let exp_gain = |l: u32| (level_exp[(l + 1) as usize] - level_exp[l as usize]) as f64;

	let mut dist = vec![f64::INFINITY; n * n];
	// How each node was reached: 0 = attack level-up, 1 = strength level-up.
	let mut came = vec![0u8; n * n];
	dist[idx(1, 1)] = 0.0;

	for a in 1..=max {
		for s in 1..=max {
			let i = idx(a, s);
			let d = dist[i];
			let dps = dps_of(attacker, target, a, s);
			if a < max {
				let j = idx(a + 1, s);
				let nd = d + exp_gain(a) / dps;
				if nd < dist[j] {
					dist[j] = nd;
					came[j] = 0;
				}
			}
			if s < max {
				let j = idx(a, s + 1);
				let nd = d + exp_gain(s) / dps;
				if nd < dist[j] {
					dist[j] = nd;
					came[j] = 1;
				}
			}
		}
	}

	// Reconstruct the path from goal back to start. Its length is known:
	// 2 * (max - 1) level-ups plus the start node.
	let mut path = Vec::with_capacity(2 * (n - 1) + 1);
	let (mut a, mut s) = (max, max);
	loop {
		let i = idx(a, s);
		path.push((a, s, dist[i]));
		if (a, s) == (1, 1) {
			break;
		}
		if came[i] == 0 {
			a -= 1;
		} else {
			s -= 1;
		}
	}
	path.reverse();
	path
}

fn base_attacker() -> Attacker {
	// High-level melee setup: 99/99, aggressive style, full melee void,
	// wielding an abyssal whip (82 str / 82 atk, 2.4s attack speed).
	Attacker {
		strength: 99,
		attack: 99,
		strength_boost: 0,
		attack_boost: 0,
		strength_prayer: StrengthPrayer::None,
		attack_prayer: AttackPrayer::None,
		weapon: ABYSSAL_WHIP,
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

	println!("Optimal leveling path 1/1 -> 99/99 (minimizes sum of exp / dps):");
	let mut prev = 0.0;
	for &(attack, strength, total) in &path {
		let step = total - prev;
		prev = total;
		let dps = dps_of(&attacker, &target, attack, strength);
		println!(
			"att={attack:02} str={strength:02}  step={step:>12.4}  total={total:>12.4}  dps={dps:.4}"
		);
	}
	println!(
		"Total time: {:.4}",
		path.last().expect("path is non-empty").2
	);
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
		let total = path.last().expect("path is non-empty").2;

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
			let dps = dps_of(attacker, target, a, s);
			let cost = |l: u32| (level_exp[(l + 1) as usize] - level_exp[l as usize]) as f64 / dps;
			if a < max {
				rec(
					a + 1,
					s,
					max,
					acc + cost(a),
					level_exp,
					attacker,
					target,
					best,
				);
			}
			if s < max {
				rec(
					a,
					s + 1,
					max,
					acc + cost(s),
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

		// The path must be monotone: each step raises exactly one skill by one.
		for w in path.windows(2) {
			let (pa, ps) = (w[0].0, w[0].1);
			let (ca, cs) = (w[1].0, w[1].1);
			assert!(
				(ca != pa) as usize + (cs != ps) as usize == 1 && ca >= pa && cs >= ps,
				"step from ({pa}, {ps}) to ({ca}, {cs}) is not a single level-up"
			);
		}
		assert_eq!(path.first().expect("path is non-empty"), &(1, 1, 0.0));
		assert_eq!((path.last().unwrap().0, path.last().unwrap().1), (MAX, MAX));
	}
}
