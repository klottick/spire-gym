//! Finite Ironclad work carried by the existing resolver.
use crate::combat::{CardHandle,CardInstanceId,EnemyHandle};

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum IroncladEffect {
    UpgradeHand{all:bool},
    UpgradeCard{instance:CardInstanceId},
    SwordBoomerangHit{damage:i32},
    DoubleCurrentBlock,
    SpotWeakness{amount:i32},
    TrueGritRandomExhaust,
    ChooseTrueGritCard,
    ChooseHeadbuttCard,
    ChooseExhumeCard,
    ChooseDualWieldCard,
    WhirlwindRound{remaining:u32,hits:Vec<(EnemyHandle,i32)>,spend_current:bool},
    WhirlwindSpend,
    HavocTop{target:Option<EnemyHandle>},
    HavocExtract{target:Option<EnemyHandle>},
    DoubleTapRepeat{child:CardHandle,target:Option<crate::combat::model::Actor>,energy_on_use:i32,cancelled:bool},
    HavocQueued{child:CardHandle,target:Option<EnemyHandle>,energy_on_use:i32,cancelled:bool},
    FinishSuppressed,
    LimitBreakDispatch,
    DropkickDispatch{damage:i32},
    RampageGrow{family:u64,amount:i32},
    ReaperResolve{targets:Vec<(EnemyHandle,i32)>,next:usize,healed:i32},
    InfernalBladeGenerate,
}
