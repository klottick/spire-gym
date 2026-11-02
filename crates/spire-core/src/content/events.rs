use super::EventId;

/// The standard pool an event belongs to (design §4, §5.2): Neow is the start event; ordinary and
/// shrine lists are rebuilt per act; one-time rows are built once per Run and never replenished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPool { Start, Ordinary, Shrine, OneTime }

/// `acts` is a bit mask (bit `n` = act `n`). Rows of one pool appear in source order, which the
/// standard pool owner keeps when it builds each act's lists.
#[derive(Clone, Copy, Debug)]
pub struct EventDefinition { pub id: EventId, pub acts: u8, pub pool: EventPool, pub definition_only: bool }
const fn row(id: EventId, acts: u8, pool: EventPool) -> EventDefinition { EventDefinition { id, acts, pool, definition_only: false } }
const A0: u8 = 1; const A1: u8 = 1 << 1; const A2: u8 = 1 << 2; const A3: u8 = 1 << 3;
pub(crate) const EVENTS: &[EventDefinition] = {
    use EventId::*; use EventPool::*;
    const ANY: u8 = A1 | A2 | A3;
    &[
        // The accepted seven keep their place; shrines Upgrade Shrine and Purifier join every act's
        // shrine list and Joust the one-time list (design §4, §5.5).
        row(Falling, A3, Ordinary), row(TombOfLordRedMask, A3, Ordinary), row(Neow, A0, Start), row(UpgradeShrine, ANY, Shrine),
        row(Purifier, ANY, Shrine), row(AncientWriting, A2, Ordinary), row(Joust, A2, OneTime),
        // Act 1 ordinary.
        row(BigFish, A1, Ordinary), row(DeadAdventurer, A1, Ordinary), row(GoldenIdol, A1, Ordinary), row(Mushrooms, A1, Ordinary),
        row(LivingWall, A1, Ordinary), row(ScrapOoze, A1, Ordinary), row(ShiningLight, A1, Ordinary), row(Cleric, A1, Ordinary),
        row(Ssssserpent, A1, Ordinary), row(WingStatue, A1, Ordinary), row(WorldOfGoop, A1, Ordinary),
        // Act 2 ordinary.
        row(Addict, A2, Ordinary), row(Beggar, A2, Ordinary), row(Colosseum, A2, Ordinary), row(CursedTome, A2, Ordinary),
        row(DrugDealer, A2, Ordinary), row(ForgottenAltar, A2, Ordinary), row(Ghosts, A2, Ordinary), row(MaskedBandits, A2, Ordinary),
        row(Nest, A2, Ordinary), row(Library, A2, Ordinary), row(Mausoleum, A2, Ordinary), row(Vampires, A2, Ordinary),
        // Act 3 ordinary.
        row(MindBloom, A3, Ordinary), row(MoaiHead, A3, Ordinary), row(MysteriousSphere, A3, Ordinary), row(SensoryStone, A3, Ordinary),
        row(WindingHalls, A3, Ordinary),
        // Shrines.
        row(GoldenShrine, ANY, Shrine), row(MatchAndKeep, ANY, Shrine), row(Transmogrifier, ANY, Shrine), row(WheelOfChange, ANY, Shrine),
        // One-time; the act mask is each row's act eligibility (design §5.2).
        row(AccursedBlacksmith, ANY, OneTime), row(BonfireSpirits, ANY, OneTime), row(FaceTrader, A1 | A2, OneTime),
        row(FountainOfCleansing, ANY, OneTime), row(Lab, ANY, OneTime), row(NoteForYourself, ANY, OneTime), row(WeMeetAgain, ANY, OneTime),
        row(WomanInBlue, ANY, OneTime), row(Designer, A2 | A3, OneTime), row(Duplicator, A2 | A3, OneTime), row(KnowingSkull, A2, OneTime),
        row(Nloth, A2, OneTime), row(SecretPortal, A3, OneTime),
    ]
};
