use osrs_dps_calculator::{AttackStyle, Attacker, GearBonus, MeleeDps, Target};

fn main() {
    // High-level melee setup: 99/99, +19/+19 boosts, piety, aggressive
    // style, full melee void, ~105 str / ~80 atk equipment bonus,
    // 1.0s attack speed (fast weapon).
    let attacker = Attacker {
        strength: 99,
        attack: 99,
        strength_boost: 19,
        attack_boost: 19,
        prayer_strength_mult: 1.23, // piety
        prayer_attack_mult: 1.20,   // piety
        equipment_strength_bonus: 105,
        equipment_attack_bonus: 80,
        attack_style: AttackStyle::Aggressive,
        void: true,
        attack_speed_secs: 1.0,
    };

    // PvM: NPC with 87 def and 60 def bonus, slayer helm bonus active.
    let npc = Target {
        is_player: false,
        defence: 87,
        defence_boost: 0,
        prayer_defence_mult: 1.0,
        attack_style: AttackStyle::Aggressive,
        defence_bonus: 60,
        protect_from_melee: false,
    };
    println!("PvM (NPC 87 def / 60 def bonus, slayer helm):");
    let r = MeleeDps::calculate(&attacker, &npc, GearBonus::Slayer);
    println!("{r}");
    println!();

    // PvP: 99 def player, +15 boost, piety, defensive style,
    // 122 def bonus, Protect from Melee active.
    let player = Target {
        is_player: true,
        defence: 99,
        defence_boost: 15,
        prayer_defence_mult: 1.20, // piety
        attack_style: AttackStyle::Defensive,
        defence_bonus: 122,
        protect_from_melee: true,
    };
    println!("PvP (99 def player, defensive, PFM, no gear bonus):");
    let r = MeleeDps::calculate(&attacker, &player, GearBonus::None);
    println!("{r}");
}
