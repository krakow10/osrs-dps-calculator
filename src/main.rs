use osrs_dps_calculator::{
	AttackPrayer, AttackStyle, Attacker, GameTicks, GearBonus, MeleeDps, StrengthPrayer, Target,
};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

/// Highest level we care about for attack and strength.
const MAX_LEVEL: u32 = 99;

fn level_exp(level: u8) -> u32 {
	(1..level)
		.map(|l| {
			let l = l as f64;
			(l + 300.0 * 2.0f64.powf(l / 7.0)) as u32
		})
		.sum::<u32>()
		/ 4
}

/// Exp needed to raise a skill from `level` to `level + 1`.
fn exp_to_gain(level: u32) -> u32 {
	level_exp((level + 1) as u8) - level_exp(level as u8)
}

/// Flatten an (attack, strength) pair into a node index, and back.
fn node_index(attack: u32, strength: u32) -> usize {
	(attack - 1) as usize * (MAX_LEVEL as usize) + (strength - 1) as usize
}

fn node_coords(idx: usize) -> (u32, u32) {
	let attack = idx as u32 / MAX_LEVEL + 1;
	let strength = idx as u32 % MAX_LEVEL + 1;
	(attack, strength)
}

/// DPS at a given (attack, strength), keeping the rest of the attacker
/// setup and target fixed.
fn dps_of(attacker: &Attacker, target: &Target, attack: u32, strength: u32) -> f64 {
	let mut atk = attacker.clone();
	atk.attack = attack;
	atk.strength = strength;
	MeleeDps::calculate(&atk, target).dps
}

/// Entry in the Dijkstra priority queue. Ordered so the smallest
/// (distance, node) pops first.
#[derive(Debug, Clone, Copy)]
struct HeapItem {
	dist: f64,
	node: usize,
}

impl PartialEq for HeapItem {
	fn eq(&self, other: &Self) -> bool {
		self.dist.total_cmp(&other.dist) == Ordering::Equal && self.node == other.node
	}
}

impl Eq for HeapItem {}

impl PartialOrd for HeapItem {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for HeapItem {
	fn cmp(&self, other: &Self) -> Ordering {
		self.dist
			.total_cmp(&other.dist)
			.then_with(|| self.node.cmp(&other.node))
	}
}

/// Find the attack/strength leveling path from 1/1 to 99/99 that
/// minimizes the total time, where the time spent on each level-up is
/// proportional to the exp needed for that level divided by the DPS at
/// the state you're in while leveling it up.
///
/// Returns the path (start to goal) and the cumulative time to reach
/// each point on the path.
fn solve(attacker: &Attacker, target: &Target) -> (Vec<(u32, u32)>, Vec<f64>) {
	let n = (MAX_LEVEL * MAX_LEVEL) as usize;
	let start = node_index(1, 1);
	let goal = node_index(MAX_LEVEL, MAX_LEVEL);

	let mut dist = vec![f64::INFINITY; n];
	let mut prev = vec![None; n];
	dist[start] = 0.0;

	let mut heap = BinaryHeap::new();
	heap.push(Reverse(HeapItem {
		dist: 0.0,
		node: start,
	}));

	while let Some(Reverse(HeapItem { dist: d, node: cur })) = heap.pop() {
		// Skip stale entries.
		if d > dist[cur] {
			continue;
		}
		if cur == goal {
			break;
		}
		let (attack, strength) = node_coords(cur);
		let current_dps = dps_of(attacker, target, attack, strength);

		// Level up attack: (a, s) -> (a + 1, s).
		if attack < MAX_LEVEL {
			let cost = exp_to_gain(attack) as f64 / current_dps;
			let next = node_index(attack + 1, strength);
			let nd = d + cost;
			if nd < dist[next] {
				dist[next] = nd;
				prev[next] = Some(cur);
				heap.push(Reverse(HeapItem {
					dist: nd,
					node: next,
				}));
			}
		}
		// Level up strength: (a, s) -> (a, s + 1).
		if strength < MAX_LEVEL {
			let cost = exp_to_gain(strength) as f64 / current_dps;
			let next = node_index(attack, strength + 1);
			let nd = d + cost;
			if nd < dist[next] {
				dist[next] = nd;
				prev[next] = Some(cur);
				heap.push(Reverse(HeapItem {
					dist: nd,
					node: next,
				}));
			}
		}
	}

	// Reconstruct the path from goal back to start.
	let mut path = Vec::new();
	let mut cur = goal;
	loop {
		path.push(node_coords(cur));
		if cur == start {
			break;
		}
		cur = match prev[cur] {
			Some(p) => p,
			None => break,
		};
	}
	path.reverse();

	let times: Vec<f64> = path
		.iter()
		.map(|&(a, s)| dist[node_index(a, s)])
		.collect();

	(path, times)
}

fn base_attacker() -> Attacker {
	// High-level melee setup: 99/99, aggressive style, full melee void,
	// 40 str / 40 atk equipment bonus, 4-tick (2.4s) attack speed.
	Attacker {
		strength: 99,
		attack: 99,
		strength_boost: 0,
		attack_boost: 0,
		strength_prayer: StrengthPrayer::None,
		attack_prayer: AttackPrayer::None,
		equipment_strength_bonus: 40,
		equipment_attack_bonus: 40,
		attack_style: AttackStyle::Aggressive,
		void: true,
		gear_bonus: GearBonus::None,
		attack_speed: GameTicks(4),
	}
}

fn test_target() -> Target {
	// PvM: NPC with 1 def and 0 def bonus.
	Target::Npc {
		defence: 1,
		defence_bonus: 0,
	}
}

fn main() {
	let attacker = base_attacker();
	let target = test_target();

	let (path, times) = solve(&attacker, &target);

	println!("Optimal leveling path 1/1 -> 99/99 (minimizes sum of exp / dps):");
	for i in 0..path.len() {
		let (attack, strength) = path[i];
		let step = if i == 0 {
			0.0
		} else {
			times[i] - times[i - 1]
		};
		let total = times[i];
		let dps = dps_of(&attacker, &target, attack, strength);
		println!(
			"att={attack:02} str={strength:02}  step={step:>12.4}  total={total:>12.4}  dps={dps:.4}"
		);
	}
	println!(
		"Total time: {:.4}",
		*times.last().expect("path is non-empty")
	);
}

#[cfg(test)]
mod tests {
	use super::*;

	/// The leveling graph is a DAG (edges only increase attack/strength), so
	/// the optimum can also be found by a topological-order DP. Use that as
	/// an independent check of the Dijkstra implementation.
	#[test]
	fn dijkstra_matches_dp() {
		let attacker = base_attacker();
		let target = test_target();

		let (path, times) = solve(&attacker, &target);

		let mut dp = vec![vec![f64::INFINITY; MAX_LEVEL as usize + 1]; MAX_LEVEL as usize + 1];
		dp[1][1] = 0.0;
		for a in 1..=MAX_LEVEL {
			for s in 1..=MAX_LEVEL {
				let d = dp[a as usize][s as usize];
				if d.is_infinite() {
					continue;
				}
				let cost = |level: u32| exp_to_gain(level) as f64 / dps_of(&attacker, &target, a, s);
				if a < MAX_LEVEL {
					dp[(a + 1) as usize][s as usize] =
						dp[(a + 1) as usize][s as usize].min(d + cost(a));
				}
				if s < MAX_LEVEL {
					dp[a as usize][(s + 1) as usize] =
						dp[a as usize][(s + 1) as usize].min(d + cost(s));
				}
			}
		}
		assert!((dp[MAX_LEVEL as usize][MAX_LEVEL as usize] - times[times.len() - 1]).abs() < 1e-6);

		// The path must be monotone: each step raises exactly one skill by one.
		for i in 1..path.len() {
			let (pa, ps) = path[i - 1];
			let (ca, cs) = path[i];
			let da = (ca as i32 - pa as i32).abs();
			let ds = (cs as i32 - ps as i32).abs();
			assert!(da + ds == 1, "step from {:?} to {:?} is not a single level-up", path[i - 1], path[i]);
		}
		assert_eq!(path[0], (1, 1));
		assert_eq!(path[path.len() - 1], (MAX_LEVEL, MAX_LEVEL));
	}
}
