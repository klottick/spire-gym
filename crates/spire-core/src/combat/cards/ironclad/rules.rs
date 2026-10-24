//! Card-specific upgrade law. Pile membership and permanent ownership stay elsewhere.
use crate::{combat::{LiveCard,CardPublicValues,CardCostKind},content::{CardId,CardType,Catalog},run::RunError};

pub(crate) fn can_upgrade(id:CardId,count:u32)->bool {
    id==CardId::SearingBlow || count==0 && Catalog::standard().card(id)
        .is_some_and(|c|matches!(c.card_type,CardType::Attack|CardType::Skill|CardType::Power))
}

/// Permanent descriptors reconstruct prior actual upgrades, unlike the original
/// count-taking fresh-kind constructor. Wide arithmetic avoids an artificial
/// iteration limit; narrowing retains the source-width damage result.
pub(crate) fn reconstructed_searing_damage(count:u32)->i32 {
    let n=u128::from(count);
    (12+n*(n+7)/2) as i32
}

pub(crate) fn searing_damage(card:&LiveCard)->i32 {
    card.local.base_damage.unwrap_or_else(||reconstructed_searing_damage(card.upgrades))
}

pub(crate) fn upgrade(card:&mut LiveCard)->Result<(),RunError> {
    if !can_upgrade(card.id,card.upgrades){return Err(RunError::InvalidAction)}
    let next=card.upgrades.checked_add(1).ok_or(RunError::Unsupported("native upgrade count exhausted"))?;
    let temporary=card.local.temporary_upgrades.checked_add(1).ok_or(RunError::Unsupported("native combat upgrade count exhausted"))?;
    if card.id==CardId::BloodForBlood {
        crate::combat::card_state::upgrade_blood_for_blood_cost(card);
    }
    if card.id==CardId::SearingBlow {
        card.local.base_damage=Some(searing_damage(card).wrapping_add(4_i32.wrapping_add(card.upgrades as i32)));
    }
    if card.id==CardId::LimitBreak{card.local.exhaust_on_finish=false;}
    if card.id==CardId::Brutality{card.local.innate=true;}
    card.upgrades=next;card.local.temporary_upgrades=temporary;Ok(())
}

pub(crate) fn public_values(card:&LiveCard)->Option<CardPublicValues> {
    let up=card.upgrades>0;
    let (damage,block,magic)=match card.id {
        CardId::BloodForBlood=>(Some(if up{22}else{18}),None,None),
        CardId::Feed=>(Some(if up{12}else{10}),None,Some(if up{4}else{3})),
        CardId::Reaper=>(Some(if up{5}else{4}),None,None),
        CardId::Dropkick=>(Some(if up{8}else{5}),None,None),
        CardId::Rampage=>(Some(card.local.base_damage.unwrap_or(8)),None,Some(if up{8}else{5})),
        CardId::Barricade|CardId::Brutality=>(None,None,None),
        CardId::InfernalBlade=>(None,None,None),
        CardId::Berserk=>(None,None,Some(if up{1}else{2})),
        CardId::DemonForm=>(None,None,Some(if up{3}else{2})),
        CardId::Metallicize=>(None,None,Some(if up{4}else{3})),
        CardId::Combust=>(None,None,Some(if up{7}else{5})),
        CardId::Flex=>(None,None,Some(if up{4}else{2})),
        CardId::LimitBreak=>(None,None,None),
        CardId::SearingBlow=>(Some(searing_damage(card)),None,None),
        CardId::Entrench=>(None,None,None),
        CardId::GhostlyArmor=>(None,Some(if up{13}else{10}),None),
        CardId::Intimidate=>(None,None,Some(if up{2}else{1})),
        CardId::PowerThrough=>(None,Some(if up{20}else{15}),None),
        CardId::RecklessCharge=>(Some(if up{10}else{7}),None,None),
        CardId::SeeingRed=>(None,None,Some(2)),
        CardId::Shockwave=>(None,None,Some(if up{5}else{3})),
        CardId::SpotWeakness=>(None,None,Some(if up{4}else{3})),
        CardId::Evolve=>(None,None,Some(if up{2}else{1})),
        CardId::FireBreathing=>(None,None,Some(if up{10}else{6})),
        CardId::FlameBarrier=>(None,Some(if up{16}else{12}),Some(if up{6}else{4})),
        CardId::Juggernaut=>(None,None,Some(if up{7}else{5})),
        CardId::Rage=>(None,None,Some(if up{5}else{3})),
        CardId::Rupture=>(None,None,Some(if up{2}else{1})),
        CardId::TrueGrit=>(None,Some(if up{9}else{7}),Some(1)),
        CardId::Headbutt=>(Some(if up{12}else{9}),None,None),
        CardId::Warcry=>(None,None,Some(if up{2}else{1})),
        CardId::BattleTrance=>(None,None,Some(if up{4}else{3})),
        CardId::Exhume=>(None,None,None),
        CardId::DualWield=>(None,None,Some(if up{2}else{1})),
        CardId::Havoc=>(None,None,None),
        CardId::DoubleTap=>(None,None,Some(if up{2}else{1})),
        CardId::Whirlwind=>(Some(if up{8}else{5}),None,None),
        CardId::BodySlam=>(None,None,None),
        CardId::Carnage=>(Some(if up{28}else{20}),None,None),
        CardId::Clash=>(Some(if up{18}else{14}),None,None),
        CardId::Hemokinesis=>(Some(if up{20}else{15}),None,Some(2)),
        CardId::Pummel=>(Some(2),None,Some(if up{5}else{4})),
        CardId::SwordBoomerang=>(Some(3),None,Some(if up{4}else{3})),
        CardId::WildStrike=>(Some(if up{17}else{12}),None,None),
        CardId::Uppercut=>(Some(13),None,Some(if up{2}else{1})),
        CardId::Immolate=>(Some(if up{28}else{21}),None,None),
        CardId::Armaments=>(None,Some(5),None),
        CardId::Corruption|CardId::DarkEmbrace=>(None,None,None),
        CardId::FeelNoPain=>(None,None,Some(if up{4}else{3})),
        CardId::Sentinel=>(None,Some(if up{8}else{5}),Some(if up{3}else{2})),
        CardId::SecondWind=>(None,Some(if up{7}else{5}),None),
        CardId::SeverSoul=>(Some(if up{22}else{16}),None,None),
        CardId::FiendFire=>(Some(if up{10}else{7}),None,None),
        CardId::Strike|CardId::StrikeGreen|CardId::StrikeBlue|CardId::StrikePurple=>(Some(if up{9}else{6}),None,None),
        CardId::Defend|CardId::DefendGreen|CardId::DefendBlue|CardId::DefendPurple=>(None,Some(if up{8}else{5}),None),
        CardId::Bash=>(Some(if up{10}else{8}),None,Some(if up{3}else{2})),
        _=>return None,
    };
    Some(CardPublicValues{base_damage:damage,base_block:block,magic_number:magic,damage_bonus:None,
        cost_kind:if card.local.x_cost{CardCostKind::X}else{CardCostKind::Fixed},free_play:card.local.free_play,
        exhaust_on_finish:card.local.exhaust_on_finish,ethereal:card.local.ethereal,innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}
