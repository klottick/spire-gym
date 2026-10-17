//! Finite content for the supplied-entry Ironclad A0 slice.
pub const CONTENT_ID: &str = "spire-lab-combat-reference-v2";

macro_rules! content_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }
        }
    };
}

content_enum!(CardKind { Strike => "strike", Defend => "defend", Bash => "bash" });
content_enum!(EnemyKind { Cultist => "cultist", JawWorm => "jaw_worm" });
content_enum!(MoveId { Incantation => "incantation", Attack => "attack", Chomp => "chomp",
    Thrash => "thrash", Bellow => "bellow" });
content_enum!(PowerId { Strength => "strength", Ritual => "ritual", Vulnerable => "vulnerable" });

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardStats {
    pub cost: i32,
    pub damage: i32,
    pub block: i32,
    pub vulnerable: i32,
}

pub fn card_stats(kind: CardKind, upgraded: bool) -> CardStats {
    let (cost, damage, block, vulnerable) = match (kind, upgraded) {
        (CardKind::Strike, false) => (1, 6, 0, 0),
        (CardKind::Strike, true) => (1, 9, 0, 0),
        (CardKind::Defend, false) => (1, 0, 5, 0),
        (CardKind::Defend, true) => (1, 0, 8, 0),
        (CardKind::Bash, false) => (2, 8, 0, 2),
        (CardKind::Bash, true) => (2, 10, 0, 3),
    };
    CardStats { cost, damage, block, vulnerable }
}
