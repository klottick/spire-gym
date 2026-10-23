//! Manual single-thread recording-disabled workload; no wall-clock test gate.
//! The allocator wrapper lives only in this benchmark executable, never the gym.
use std::{alloc::{GlobalAlloc,Layout,System},sync::atomic::{AtomicBool,AtomicU64,Ordering},time::Instant,hint::black_box};
use spire_core::combat::*;
use spire_core::combat::rng::{make_stream,Partition,StreamId,Domain,RNG_CONTRACT_ID};
struct Counter;
static COUNT:AtomicU64=AtomicU64::new(0);
static MEASURE:AtomicBool=AtomicBool::new(false);
// SAFETY: forward each unchanged pointer/layout contract to the system allocator.
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {if MEASURE.load(Ordering::Relaxed){COUNT.fetch_add(1,Ordering::Relaxed);}unsafe{System.alloc(l)}}
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8 {if MEASURE.load(Ordering::Relaxed){COUNT.fetch_add(1,Ordering::Relaxed);}unsafe{System.alloc_zeroed(l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {if MEASURE.load(Ordering::Relaxed){COUNT.fetch_add(1,Ordering::Relaxed);}unsafe{System.realloc(p,l,n)}}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)}}
}
#[global_allocator] static ALLOCATOR:Counter=Counter;
fn scenario()->Scenario {
    let enemy=EnemyHandle::parse("enemy-0").unwrap();
    let stream=|domain,scope|make_stream(&[7;32],Partition::Test,"disabled-workload",&StreamId::new(domain,scope).unwrap()).unwrap();
    let cards=|offset:usize,kinds:&[CardKind]|kinds.iter().enumerate().map(|(i,kind)|CardInstance {instance_id:CardHandle::parse(&(i+offset).to_string()).unwrap(),card_id:*kind,upgraded:false}).collect();
    Scenario {encounter_id:EnemyKind::Cultist,player:PlayerState {hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        enemies:vec![EnemySetup {instance_id:enemy.clone(),enemy_id:EnemyKind::Cultist,hp:52,max_hp:52,block:0,powers:vec![],intent_id:MoveId::Incantation,move_history:vec![MoveId::Incantation]}],
        piles:Piles {hand:cards(0,&[CardKind::Strike,CardKind::Defend,CardKind::Strike,CardKind::Defend,CardKind::Bash]),draw:cards(5,&[CardKind::Strike,CardKind::Strike,CardKind::Strike,CardKind::Defend,CardKind::Defend]),discard:vec![],exhaust:vec![]},
        rng:CombatRng {contract_id:RNG_CONTRACT_ID.into(),shuffle:stream(Domain::CombatShuffle,vec!["disabled".into()]),enemy_ai:stream(Domain::CombatEnemyAi,vec!["disabled".into(),"enemy-0".into()]),enemy_instance_id:enemy}}
}
fn workload(setup:&Scenario,operations:usize)->u64 {
    let mut checksum=0u64;
    for _ in 0..operations {
        let mut combat=Combat::new(black_box(setup)).unwrap();
        for turn in 0..128 {
            let legal=combat.legal_actions().unwrap();if legal.is_empty(){break;}
            let action=if turn==0 {Action::EndTurn}else{legal[0].clone()};
            combat.step(black_box(&action)).unwrap();
        }
        let public=combat.public_view().unwrap();assert!(public.outcome.is_some());
        checksum=checksum.wrapping_add(public.player.hp as u64+public.turn as u64*1000+public.enemies[0].hp as u64*1000000);
    }checksum
}
fn main() {
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    let operations=args.first().map(|s|s.parse::<usize>().expect("operations")).unwrap_or(1000);
    let trials=args.get(1).map(|s|s.parse::<usize>().expect("trials")).unwrap_or(7);
    assert!(operations>0&&trials>0);let setup=scenario();black_box(workload(&setup,10));
    for trial in 0..trials {
        COUNT.store(0,Ordering::Relaxed);MEASURE.store(true,Ordering::Relaxed);let start=Instant::now();
        let checksum=black_box(workload(&setup,operations));let nanos=start.elapsed().as_nanos();MEASURE.store(false,Ordering::Relaxed);
        println!("trial={trial} operations={operations} ns={nanos} checksum={checksum} allocations={}",COUNT.load(Ordering::Relaxed));
    }
}
