//! Melee DPS calculator for Old School RuneScape.
//!
//! Implements the formulas from the OSRS wiki "Damage per second/Melee" page:
//! <https://oldschool.runescape.wiki/w/Damage_per_second/Melee>

/// Melee attack style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackStyle {
    /// +3 attack.
    Accurate,
    /// +3 strength.
    Aggressive,
    /// +1 attack and +1 strength.
    Controlled,
    /// +3 defence (defender only).
    Defensive,
}

/// Target-specific gear bonus.
///
/// Per the wiki, the slayer helm and salve amulet bonuses do NOT stack;
/// use the better of the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GearBonus {
    /// No gear bonus (1.0).
    #[default]
    None,
    /// 7/6 — slayer helm (on task) or salve amulet (undead).
    Slayer,
    /// 1.2 — salve amulet (e) or (ei).
    EnhancedSalve,
}

impl GearBonus {
    fn multiplier(self) -> f64 {
        match self {
            GearBonus::None => 1.0,
            GearBonus::Slayer => 7.0 / 6.0,
            GearBonus::EnhancedSalve => 1.2,
        }
    }
}

/// The player doing the attacking.
#[derive(Debug, Clone)]
pub struct Attacker {
    pub strength: u32,
    pub attack: u32,
    /// Temporary strength level boost (potion, cape, etc.).
    pub strength_boost: i32,
    /// Temporary attack level boost (potion, cape, etc.).
    pub attack_boost: i32,
    /// Strength prayer multiplier.
    /// Burst of Strength 1.05, Superhuman 1.10, Ultimate 1.15, Chivalry 1.18, Piety 1.23.
    pub prayer_strength_mult: f64,
    /// Attack prayer multiplier.
    /// Clarity 1.05, Improved Reflexes 1.10, Incredible Reflexes 1.15, Chivalry 1.15, Piety 1.20.
    pub prayer_attack_mult: f64,
    /// "Melee Strength" bonus from the equipment stats window.
    pub equipment_strength_bonus: i32,
    /// Stab/Slash/Crush bonus matching the weapon's attack type.
    pub equipment_attack_bonus: i32,
    pub attack_style: AttackStyle,
    /// Wearing full melee void.
    pub void: bool,
    /// Weapon attack speed in seconds per attack (e.g. 1.0 for a scimitar, 1.2 for an axe).
    pub attack_speed_secs: f64,
}

/// The target being attacked (an NPC or a player).
#[derive(Debug, Clone)]
pub struct Target {
    pub is_player: bool,
    /// Base defence level. For NPCs this is the shield-icon value on the wiki.
    pub defence: u32,
    /// Temporary defence boost (player targets only).
    pub defence_boost: i32,
    /// Defence prayer multiplier (player targets only; e.g. 1.15 incredible protection, 1.20 piety).
    pub prayer_defence_mult: f64,
    /// Player target's attack style (affects defence style bonus; player targets only).
    pub attack_style: AttackStyle,
    /// Target's stab/slash/crush defence bonus matching the attack type.
    pub defence_bonus: i32,
    /// Player target has Protect from Melee active.
    pub protect_from_melee: bool,
}

/// All intermediate values and the final DPS.
#[derive(Debug, Clone, PartialEq)]
pub struct MeleeDps {
    pub effective_strength: u32,
    pub max_hit: u32,
    pub effective_attack: u32,
    pub attack_roll: u32,
    /// `Some` for player targets, `None` for NPCs.
    pub effective_defence: Option<u32>,
    pub defence_roll: u32,
    pub hit_chance: f64,
    pub average_damage_per_attack: f64,
    pub dps: f64,
}

impl MeleeDps {
    /// Calculate melee DPS using the formulas from the OSRS wiki.
    pub fn calculate(attacker: &Attacker, target: &Target, gear: GearBonus) -> MeleeDps {
        // --- Step one: effective strength level -------------------------------
        let strength_style_bonus = match attacker.attack_style {
            AttackStyle::Aggressive => 3,
            AttackStyle::Controlled => 1,
            _ => 0,
        };
        let effective_strength = effective_level(
            attacker.strength,
            attacker.strength_boost,
            attacker.prayer_strength_mult,
            strength_style_bonus,
            attacker.void,
        );

        // --- Step two: max hit ------------------------------------------------
        // eff_strength * (str bonus + 64) + 320, / 640, floor, then gear bonus, floor.
        let max_hit_base =
            ((effective_strength as f64 * (attacker.equipment_strength_bonus as f64 + 64.0) + 320.0)
                / 640.0)
                .floor() as u64;
        let mut max_hit = (max_hit_base as f64 * gear.multiplier()).floor() as u64;
        if target.protect_from_melee {
            max_hit = (max_hit as f64 * 0.6).floor() as u64;
        }
        let max_hit = max_hit as u32;

        // --- Step three: effective attack level -------------------------------
        let attack_style_bonus = match attacker.attack_style {
            AttackStyle::Accurate => 3,
            AttackStyle::Controlled => 1,
            _ => 0,
        };
        let effective_attack = effective_level(
            attacker.attack,
            attacker.attack_boost,
            attacker.prayer_attack_mult,
            attack_style_bonus,
            attacker.void,
        );

        // --- Step four: attack roll --------------------------------------------
        let attack_roll = (effective_attack as f64
            * (attacker.equipment_attack_bonus as f64 + 64.0)
            * gear.multiplier())
        .floor() as u32;

        // --- Steps five & six: defence roll ------------------------------------
        let (effective_defence, defence_roll) = if target.is_player {
            let def_style_bonus = match target.attack_style {
                AttackStyle::Defensive => 3,
                AttackStyle::Controlled => 1,
                _ => 0,
            };
            let eff = effective_level(
                target.defence,
                target.defence_boost,
                target.prayer_defence_mult,
                def_style_bonus,
                false,
            );
            (
                Some(eff),
                eff as u64 * (target.defence_bonus as u64 + 64),
            )
        } else {
            // NPC: (defence level + 9) * (defence bonus + 64)
            (
                None,
                (target.defence as u64 + 9) * (target.defence_bonus as u64 + 64),
            )
        };
        let defence_roll = defence_roll as u32;

        // --- Step seven: hit chance --------------------------------------------
        let (a, d) = (attack_roll as f64, defence_roll as f64);
        let hit_chance = if a > d {
            1.0 - (d + 2.0) / (2.0 * (a + 1.0))
        } else {
            a / (2.0 * (d + 1.0))
        };

        // --- Step eight: damage output ------------------------------------------
        // Hit chance * (max_hit / 2 + 1 / max_hit + 1). The +1/max_hit term
        // accounts for a 0 roll on a successful hit being bumped up to 1.
        // (If max hit is 0, every successful hit deals exactly 1.)
        let average_damage_per_attack = if max_hit == 0 {
            hit_chance
        } else {
            hit_chance * (max_hit as f64 / 2.0 + 1.0 / max_hit as f64 + 1.0)
        };
        let dps = average_damage_per_attack / attacker.attack_speed_secs;

        MeleeDps {
            effective_strength,
            max_hit,
            effective_attack,
            attack_roll,
            effective_defence,
            defence_roll,
            hit_chance,
            average_damage_per_attack,
            dps,
        }
    }
}

/// Steps one/three/five: (level + boost) * prayer, floor, + style bonus, +8,
/// optional *1.1 void, floor.
fn effective_level(
    level: u32,
    boost: i32,
    prayer_mult: f64,
    style_bonus: u32,
    void: bool,
) -> u32 {
    let mut lvl = (level as f64 + boost as f64) * prayer_mult;
    lvl = lvl.floor() + style_bonus as f64 + 8.0;
    if void {
        lvl *= 1.1;
    }
    lvl.floor() as u32
}

impl std::fmt::Display for MeleeDps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Effective strength: {}", self.effective_strength)?;
        writeln!(f, "  Max hit:            {}", self.max_hit)?;
        writeln!(f, "  Effective attack:   {}", self.effective_attack)?;
        writeln!(f, "  Attack roll:        {}", self.attack_roll)?;
        if let Some(eff_def) = self.effective_defence {
            writeln!(f, "  Effective defence:  {eff_def}")?;
        }
        writeln!(f, "  Defence roll:       {}", self.defence_roll)?;
        writeln!(f, "  Hit chance:         {:.4} ({:.2}%)", self.hit_chance, self.hit_chance * 100.0)?;
        writeln!(f, "  Avg damage/hit:     {:.4}", self.average_damage_per_attack)?;
        write!(f, "  DPS:              {:.4}", self.dps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_target_matches_hand_calculation() {
        // 99/99 with +10/+10, piety (1.23 str / 1.20 atk), aggressive style,
        // no void, str bonus 60, atk bonus 80, 1.0s weapon.
        let attacker = Attacker {
            strength: 99,
            attack: 99,
            strength_boost: 10,
            attack_boost: 10,
            prayer_strength_mult: 1.23,
            prayer_attack_mult: 1.20,
            equipment_strength_bonus: 60,
            equipment_attack_bonus: 80,
            attack_style: AttackStyle::Aggressive,
            void: false,
            attack_speed_secs: 1.0,
        };
        // NPC with 50 def, 40 def bonus.
        let target = Target {
            is_player: false,
            defence: 50,
            defence_boost: 0,
            prayer_defence_mult: 1.0,
            attack_style: AttackStyle::Aggressive,
            defence_bonus: 40,
            protect_from_melee: false,
        };

        let r = MeleeDps::calculate(&attacker, &target, GearBonus::None);

        // eff strength: floor(109 * 1.23) = 134; 134 + 3 (aggressive) + 8 = 145
        assert_eq!(r.effective_strength, 145);
        // max hit: floor((145 * 124 + 320) / 640) = floor(28.59) = 28
        assert_eq!(r.max_hit, 28);
        // eff attack: floor(109 * 1.20) = 130; 130 + 3 + 8 = 141
        assert_eq!(r.effective_attack, 141);
        // attack roll: 141 * 144 = 20304
        assert_eq!(r.attack_roll, 20_304);
        // def roll: (50 + 9) * (40 + 64) = 6136
        assert_eq!(r.defence_roll, 6_136);
        assert_eq!(r.effective_defence, None);

        let expected_hit_chance = 1.0 - 6138.0 / (2.0 * 20_305.0);
        assert!((r.hit_chance - expected_hit_chance).abs() < 1e-12);

        let expected_dps = expected_hit_chance * (14.0 + 1.0 / 28.0 + 1.0);
        assert!((r.dps - expected_dps).abs() < 1e-9);
        assert!((r.dps - 12.7631).abs() < 1e-3);
    }

    #[test]
    fn player_target_with_gear_and_pfm() {
        let attacker = Attacker {
            strength: 99,
            attack: 99,
            strength_boost: 10,
            attack_boost: 10,
            prayer_strength_mult: 1.23,
            prayer_attack_mult: 1.20,
            equipment_strength_bonus: 60,
            equipment_attack_bonus: 80,
            attack_style: AttackStyle::Aggressive,
            void: false,
            attack_speed_secs: 1.0,
        };
        // Player target: 99 def +15, piety (1.20), defensive style, 100 def bonus, PFM.
        let target = Target {
            is_player: true,
            defence: 99,
            defence_boost: 15,
            prayer_defence_mult: 1.20,
            attack_style: AttackStyle::Defensive,
            defence_bonus: 100,
            protect_from_melee: true,
        };

        let r = MeleeDps::calculate(&attacker, &target, GearBonus::EnhancedSalve);

        // max hit: floor(28 * 1.2) = 33, then PFM: floor(33 * 0.6) = 19
        assert_eq!(r.max_hit, 19);
        // attack roll includes the salve (e) gear bonus: floor(141 * 144 * 1.2) = 24364
        assert_eq!(r.attack_roll, 24_364);
        // eff def: floor(114 * 1.20) = 136; 136 + 3 (defensive) + 8 = 147
        assert_eq!(r.effective_defence, Some(147));
        // def roll: 147 * 164 = 24108
        assert_eq!(r.defence_roll, 24_108);
    }

    #[test]
    fn zero_max_hit_is_handled() {
        let attacker = Attacker {
            strength: 1,
            attack: 1,
            strength_boost: 0,
            attack_boost: 0,
            prayer_strength_mult: 1.0,
            prayer_attack_mult: 1.0,
            // Bonus of -64 zeroes the strength term: (eff_str * 0 + 320) / 640 = 0.5 -> 0
            equipment_strength_bonus: -64,
            equipment_attack_bonus: 0,
            attack_style: AttackStyle::Aggressive,
            void: false,
            attack_speed_secs: 1.0,
        };
        let target = Target {
            is_player: false,
            defence: 1,
            defence_boost: 0,
            prayer_defence_mult: 1.0,
            attack_style: AttackStyle::Aggressive,
            defence_bonus: 0,
            protect_from_melee: false,
        };

        let r = MeleeDps::calculate(&attacker, &target, GearBonus::None);
        assert_eq!(r.max_hit, 0);
        // Every successful hit rolls 0 and is bumped up to 1.
        assert!((r.average_damage_per_attack - r.hit_chance).abs() < 1e-12);
    }
}
