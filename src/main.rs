use osrs_dps_calculator::{AttackPrayer, AttackStyle, Attacker, DefencePrayer, GameTicks, GearBonus, MeleeDps, StrengthPrayer, Target};

fn main() {
    // High-level melee setup: 99/99, +19/+19 boosts, piety, aggressive
    // style, full melee void, ~105 str / ~80 atk equipment bonus,
    // 2-tick (1.2s) attack speed.
    let attacker = Attacker {
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
        attack_speed: GameTicks(2),
    };

    // PvM: NPC with 87 def and 60 def bonus, slayer helm bonus active.
    let npc = Target {
        is_player: false,
        defence: 87,
        defence_boost: 0,
        defence_prayer: DefencePrayer::None,
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
        defence_prayer: DefencePrayer::Piety,
        attack_style: AttackStyle::Defensive,
        defence_bonus: 122,
        protect_from_melee: true,
    };
    println!("PvP (99 def player, defensive, PFM, no gear bonus):");
    let r = MeleeDps::calculate(&attacker, &player, GearBonus::None);
    println!("{r}");
}
