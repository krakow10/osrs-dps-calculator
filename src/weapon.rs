use crate::attacker::AttackStyle;

/// A number of game ticks. One tick is exactly 0.6 seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameTicks(pub u32);

impl GameTicks {
	/// Length of one tick in seconds.
	pub const SECONDS_PER_TICK: f64 = 0.6;

	/// Total time in seconds. Zero ticks is 0.0 seconds.
	pub fn as_seconds(self) -> f64 {
		f64::from(self.0) * Self::SECONDS_PER_TICK
	}
}

/// Bonuses provided by a melee weapon, as listed on the OSRS wiki.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponStats {
	/// Stab attack bonus.
	pub stab: i32,
	/// Slash attack bonus.
	pub slash: i32,
	/// Crush attack bonus.
	pub crush: i32,
	/// Strength bonus.
	pub strength: i32,
	/// Ticks per attack.
	pub attack_speed: GameTicks,
}

impl WeaponStats {
	/// Attack bonus for the given style.
	///
	/// One-handed weapons: only the controlled style uses the stab bonus;
	/// every other style uses the slash bonus.
	pub fn attack_bonus(self, style: AttackStyle) -> i32 {
		match style {
			AttackStyle::Controlled => self.stab,
			_ => self.slash,
		}
	}
}

/// Iron scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const IRON_SCIMITAR: WeaponStats = WeaponStats {
	stab: 2,
	slash: 10,
	crush: -2,
	strength: 9,
	attack_speed: GameTicks(4),
};

/// Steel scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const STEEL_SCIMITAR: WeaponStats = WeaponStats {
	stab: 3,
	slash: 15,
	crush: -2,
	strength: 14,
	attack_speed: GameTicks(4),
};

/// Black scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const BLACK_SCIMITAR: WeaponStats = WeaponStats {
	stab: 4,
	slash: 19,
	crush: -2,
	strength: 14,
	attack_speed: GameTicks(4),
};

/// Mithril scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const MITHRIL_SCIMITAR: WeaponStats = WeaponStats {
	stab: 5,
	slash: 21,
	crush: -2,
	strength: 20,
	attack_speed: GameTicks(4),
};

/// Adamant scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const ADAMANT_SCIMITAR: WeaponStats = WeaponStats {
	stab: 6,
	slash: 29,
	crush: -2,
	strength: 28,
	attack_speed: GameTicks(4),
};

/// Rune scimitar.
/// <https://oldschool.runescape.wiki/w/Rune_scimitar>
pub const RUNE_SCIMITAR: WeaponStats = WeaponStats {
	stab: 7,
	slash: 45,
	crush: -2,
	strength: 44,
	attack_speed: GameTicks(4),
};

/// Dragon scimitar.
/// <https://oldschool.runescape.wiki/w/Scimitar>
pub const DRAGON_SCIMITAR: WeaponStats = WeaponStats {
	stab: 8,
	slash: 67,
	crush: -2,
	strength: 66,
	attack_speed: GameTicks(4),
};

/// Abyssal whip.
/// <https://oldschool.runescape.wiki/w/Abyssal_whip>
pub const ABYSSAL_WHIP: WeaponStats = WeaponStats {
	stab: 0,
	slash: 82,
	crush: 0,
	strength: 82,
	attack_speed: GameTicks(4),
};

/// Blade of Saeldor, fully charged (the inactive blade has all bonuses +0).
/// <https://oldschool.runescape.wiki/w/Blade_of_Saeldor>
pub const BLADE_OF_SAELDOR: WeaponStats = WeaponStats {
	stab: 0,
	slash: 100,
	crush: 0,
	strength: 93,
	attack_speed: GameTicks(4),
};

/// The weapons the attacker progresses through, weakest to strongest, each
/// paired with the minimum attack level required to wield it.
///
/// The scimitars can be wielded at any level; the abyssal whip requires 70
/// attack and the blade of Saeldor 80.
pub const WEAPONS: &[(WeaponStats, u8)] = &[
	(IRON_SCIMITAR, 1),
	(STEEL_SCIMITAR, 5),
	(BLACK_SCIMITAR, 10),
	(MITHRIL_SCIMITAR, 20),
	(ADAMANT_SCIMITAR, 30),
	(RUNE_SCIMITAR, 40),
	(DRAGON_SCIMITAR, 60),
	(ABYSSAL_WHIP, 70),
	(BLADE_OF_SAELDOR, 80),
];
