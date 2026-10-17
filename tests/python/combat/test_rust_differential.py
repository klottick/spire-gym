"""Authored semantic comparison, not original-runtime evidence."""
from copy import deepcopy
from pathlib import Path
import json
import subprocess
import os
import sys
import pytest
import rust_diff_support as support
from rust_diff_support import (assert_case_equal, build_rust_executable, fixed_case,
    walk_case, normalize_python_checkpoint, parse_test_executable, run_python_case, run_rust_case)

@pytest.fixture(scope="session")
def rust_combat_executable():
    return build_rust_executable()

def compare_real(executable, case, directory):
    left=run_python_case(case); right=run_rust_case(executable,case,directory)
    assert_case_equal(left,right)
    return right

@pytest.mark.parametrize("enemy",["cultist","jaw_worm"])
@pytest.mark.parametrize("death",[False,True])
def test_fixed_checkpoints_match(rust_combat_executable,tmp_path,enemy,death):
    result=compare_real(rust_combat_executable,fixed_case(enemy,death=death),tmp_path)
    end=result["checkpoints"][-1]; assert end["status"]=="terminal"
    assert end["state"]["outcome"]==("death" if death else "victory")
    expected=(0,0) if death else ((40,46) if enemy=="cultist" else (29,35))
    assert (end["state"]["hp_before_healing"],end["state"]["hp_after_cleanup"])==expected
    if not death:
        p=result["checkpoints"]
        assert p[2]["state"]["enemies"][0]["hp"]==(25 if enemy=="cultist" else 17)
        assert p[4]["state"]["enemies"][0]["hp"]==(10 if enemy=="cultist" else 2)
        assert [c["instance_id"] for c in p[3]["state"]["piles"]["hand"]]==["1","3","2"]

@pytest.mark.parametrize("enemy",["cultist","jaw_worm"])
@pytest.mark.parametrize("episode",range(20))
def test_legal_walk_checkpoints_match(rust_combat_executable,tmp_path,enemy,episode):
    assert compare_real(rust_combat_executable,walk_case(enemy,episode),tmp_path)["checkpoints"][-1]["status"]=="terminal"

@pytest.mark.parametrize("bad,reason",[
    ({"kind":"play_card","card_instance_id":True,"target_id":"enemy-1"},"interchange"),
    ({"kind":"play_card","card_instance_id":"01","target_id":"enemy-1"},"interchange"),
    ({"kind":"play_card","card_instance_id":"3","target_id":"player"},"invalid_action"),
    ({"kind":"play_card","card_instance_id":"3","target_id":""},"invalid_action"),
    ({"kind":"dance"},"unsupported")])
def test_late_rejection_preserves_checkpoint(rust_combat_executable,tmp_path,bad,reason):
    case=fixed_case("cultist"); case["actions"]=case["actions"][:1]+[bad]
    a,b=compare_real(rust_combat_executable,case,tmp_path)["checkpoints"][-2:]
    assert b["status"]=="rejected" and b["rejection"]==reason
    for key in ("state","public","legal"): assert a[key]==b[key]

@pytest.mark.parametrize("owner",["enemy_ai","shuffle"])
def test_exhaustion_and_post_fault_match(rust_combat_executable,tmp_path,owner):
    case=fixed_case("jaw_worm" if owner=="enemy_ai" else "cultist")
    case["scenario"]["rng"][owner]["word_index"]=str(2**64 if owner=="enemy_ai" else 2**64-1)
    case["actions"]=[{"kind":"end_turn"}]*2
    # Independent dispatch accounting: end-player/discard/start/move/payload/roll,
    # then for Cultist end-round/draw/recycle. No partial permutation commits.
    expected=run_python_case(case)["checkpoints"][1]["state"]
    assert expected["resolved_effects"]==(6 if owner=="enemy_ai" else 9)
    assert (expected["phase"],expected["turn"],expected["player"]["hp"])==(
        ("enemy_turn",1,29) if owner=="enemy_ai" else ("player_start",2,40))
    assert expected["piles"]["hand"]==expected["piles"]["draw"]==[]
    assert [c["instance_id"] for c in expected["piles"]["discard"]]==["3","2","1"]
    assert [e["kind"] for e in expected["pending"]]==(["end_round"] if owner=="enemy_ai" else ["draw","decision"])
    point,rejected=compare_real(rust_combat_executable,case,tmp_path)["checkpoints"][1:]
    assert point["status"]=="fault" and point["public"] is point["legal"] is None
    s=point["state"]; assert s["fault"]=={"kind":"rng_exhausted","resolved_effects":s["resolved_effects"]}
    assert s["current"]["kind"]==("roll_move" if owner=="enemy_ai" else "recycle")
    assert s["pending"] and s["resolved_effects"]>0 and s["rng"][owner]["word_index"]==str(2**64)
    assert rejected["rejection"]=="faulted" and rejected["state"]==s

@pytest.mark.parametrize("enemy",["cultist","jaw_worm"])
def test_post_terminal_actions_preserve_checkpoint(rust_combat_executable,tmp_path,enemy):
    case=fixed_case(enemy); case["actions"] += [{"kind":"end_turn"}]*2
    points=compare_real(rust_combat_executable,case,tmp_path)["checkpoints"]
    for rejected in points[-2:]:
        assert rejected["rejection"]=="terminal" and rejected["state"]==points[-3]["state"]
        assert rejected["legal"]==[]

def test_large_negative_handles_round_trip(rust_combat_executable,tmp_path):
    case=fixed_case("jaw_worm"); mapping={"1":"-"+"9"*90,"2":"8"*100,"3":"0"}
    case["scenario"]["enemies"][0]["instance_id"]="123"; case["scenario"]["rng"]["enemy_instance_id"]="123"
    for c in case["scenario"]["piles"]["hand"]: c["instance_id"]=mapping[c["instance_id"]]
    for a in case["actions"]:
        if a["kind"]=="play_card": a["card_instance_id"]=mapping[a["card_instance_id"]]; a["target_id"]="123"
    compare_real(rust_combat_executable,case,tmp_path)

def unbounded_handle_case(enemy,negative=False):
    case=fixed_case(enemy)
    mapping={"1":("-" if negative else "")+"9"*5000,"2":"8"*5001,"3":"0"}
    case["case_id"]=f"unbounded-{enemy}-{'negative' if negative else 'positive'}"
    for card in case["scenario"]["piles"]["hand"]:
        card["instance_id"]=mapping[card["instance_id"]]
    for action in case["actions"]:
        if action["kind"]=="play_card": action["card_instance_id"]=mapping[action["card_instance_id"]]
    return case,mapping

@pytest.mark.parametrize("enemy",["cultist","jaw_worm"])
@pytest.mark.parametrize("negative",[False,True])
def test_unbounded_handles_play_snapshot_and_recycle_match(rust_combat_executable,tmp_path,enemy,negative):
    before=sys.get_int_max_str_digits(); environment=dict(os.environ)
    case,mapping=unbounded_handle_case(enemy,negative)
    try:
        points=compare_real(rust_combat_executable,case,tmp_path)["checkpoints"]
        assert points[-1]["status"]=="terminal"
        assert points[0]["state"]["card_ids_issued"]==sorted(mapping.values())
        assert points[3]["state"]["public"]["revealed_order"]==[mapping["1"],mapping["3"],mapping["2"]]
        assert points[3]["public"]["revealed_order"]==points[3]["state"]["public"]["revealed_order"]
    finally:
        assert sys.get_int_max_str_digits()==before
        assert dict(os.environ)==environment

@pytest.mark.parametrize("negative",[False,True])
def test_unbounded_hidden_draw_then_recycle_reveals_match(rust_combat_executable,tmp_path,negative):
    before=sys.get_int_max_str_digits(); environment=dict(os.environ)
    case,mapping=unbounded_handle_case("cultist",negative)
    piles=case["scenario"]["piles"]
    bash,strike,defend=piles["hand"]
    piles["hand"]=[defend]; piles["draw"]=[strike,bash]
    case["actions"]=[{"kind":"end_turn"}]+case["actions"]
    try:
        points=compare_real(rust_combat_executable,case,tmp_path)["checkpoints"]
        assert points[0]["public"]["revealed_order"]==[]
        assert mapping["1"] not in json.dumps(points[0]["public"])
        assert points[1]["public"]["revealed_order"]==[mapping["1"],mapping["2"],mapping["3"]]
        assert points[1]["state"]["rng"]["shuffle"]["word_index"]=="0"
        assert points[4]["state"]["rng"]["shuffle"]["word_index"]=="2"
        assert points[-1]["status"]=="terminal"
        assert points[4]["public"]["revealed_order"][-3:]==[mapping["1"],mapping["3"],mapping["2"]]
    finally:
        assert sys.get_int_max_str_digits()==before
        assert dict(os.environ)==environment

@pytest.mark.parametrize("negative",[False,True])
def test_unbounded_adapter_conversions_and_snapshot_projection_preserve_parent(negative):
    from spire_lab.combat import engine
    from spire_lab.combat.snapshot import snapshot,restore
    before=sys.get_int_max_str_digits(); environment=dict(os.environ)
    case,mapping=unbounded_handle_case("cultist",negative)
    try:
        scenario,_=support.decode_request(case)
        state=engine.initial_combat(scenario); saved=snapshot(state)
        projected=normalize_python_checkpoint(state)
        assert projected==normalize_python_checkpoint(restore(saved))
        assert projected["card_ids_issued"]==sorted(mapping.values())
        action=support.decode_action(case["actions"][0])
        assert support.encode_action(action)==case["actions"][0]
        # Prove the actual Python identities are exact arbitrary integers, not
        # aliases or int subclasses with a special string representation.
        handle=scenario.piles.hand[0].instance_id
        assert type(handle) is int and abs(handle)%10**9==999999999
        assert abs(handle).bit_length()>16000
    finally:
        assert sys.get_int_max_str_digits()==before
        assert dict(os.environ)==environment

def test_unbounded_comparison_child_error_preserves_parent(monkeypatch,tmp_path):
    before=sys.get_int_max_str_digits(); environment=dict(os.environ)
    case,_=unbounded_handle_case("cultist")
    # Actual interpreter startup failure, not a mocked process result. Its
    # decimal-limit flag must never change this pytest process, even on error.
    monkeypatch.setattr(support,"PYTHON_CASE_WORKER",tmp_path/"absent-comparison-worker.py",raising=False)
    try:
        with pytest.raises(AssertionError,match="subprocess exit"):
            run_python_case(case)
    finally:
        assert sys.get_int_max_str_digits()==before
        assert dict(os.environ)==environment

def test_non_scalar_request_text_is_entry_interchange(rust_combat_executable,tmp_path):
    # A lone surrogate is not Unicode scalar interchange text at all, even if
    # located in a later action. Do not replace it with a different target ID.
    case=fixed_case("cultist")
    case["actions"][3]["target_id"]="\ud800"
    result=compare_real(rust_combat_executable,case,tmp_path)
    assert result["entry_error"]=="interchange" and result["checkpoints"]==[]

@pytest.mark.parametrize("path,value,error",[
    (("player","hp"),True,"interchange"),(("player","hp"),40.0,"interchange"),
    (("player","hp"),2**31,"interchange"),(("player","hp"),0,"invariant"),
    (("player","extra"),1,"interchange"),(("encounter_id",),"slime","unsupported"),
    (("enemies",0,"intent_id"),"sleep","unsupported"),
    (("piles","hand",0,"card_id"),"feed","unsupported"),
    (("piles","hand",0,"upgraded"),1,"interchange"),
    (("piles","hand",0,"instance_id"),"-0","interchange"),
    (("rng","shuffle","word_index"),str(2**64+1),"interchange"),
    (("rng","shuffle","word_index"),"00","interchange"),
    (("rng","shuffle","key_hex"),"AB"*32,"interchange"),
    (("rng","shuffle","domain"),"unknown","interchange"),
    (("rng","shuffle","scope"),[],"invariant"),
    (("enemies",0,"instance_id"),"\ud800","interchange")])
def test_malformed_interchange_never_coerces_values(rust_combat_executable,tmp_path,path,value,error):
    case=fixed_case("cultist"); obj=case["scenario"]
    for key in path[:-1]: obj=obj[key]
    obj[path[-1]]=value
    r=compare_real(rust_combat_executable,case,tmp_path)
    assert r["entry_error"]==error and r["checkpoints"]==[]

def test_normalizer_has_every_component():
    from dataclasses import fields
    from spire_lab.combat.model import CombatState
    r=run_python_case(fixed_case("cultist")); s=r["checkpoints"][1]["state"]
    assert list(s)==[f.name for f in fields(CombatState)]
    assert set(s["enemies"][0])=={"instance_id","enemy_id","hp","max_hp","block","powers","intent_id","move_history","selected_move","damage_entries","intent_damage"}
    assert set(s["enemies"][0]["powers"][0])=={"id","amount","skip_first","monster_source_grace"}

def test_missing_ambiguous_or_stale_artifact_fails(tmp_path):
    exe=tmp_path/"native"; exe.write_text("authored executable"); manifest=tmp_path/"Cargo.toml"
    a={"reason":"compiler-artifact","manifest_path":str(manifest.resolve()),"target":{"name":"spire_core","kind":["lib"]},"profile":{"test":True},"executable":str(exe)}
    line=json.dumps(a); assert parse_test_executable(line,manifest)==exe
    for text in ("",line+"\n"+line,line.replace(str(manifest),str(tmp_path/"other.toml")),line.replace('"test": true','"test": false'),line.replace(str(exe),str(tmp_path/"missing"))):
        with pytest.raises(AssertionError): parse_test_executable(text,manifest)

@pytest.mark.parametrize("mode",["zero","missing","stale","indices","ignored","exit"])
def test_zero_tests_or_missing_response_fails(monkeypatch,tmp_path,mode):
    case=fixed_case("cultist")
    def fake_run(argv,**kw):
        assert argv[1:]==["--exact","combat::tests::differential::protocol","--nocapture"]
        r=run_python_case(case)
        if mode=="stale": r["case_id"]="stale"
        if mode=="indices": r["checkpoints"].pop()
        if mode!="missing": Path(kw["env"]["SPIRE_COMBAT_DIFF_RESPONSE"]).write_text(json.dumps(r))
        out="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out"
        if mode=="zero": out=out.replace("1 passed","0 passed")
        if mode=="ignored": out=out.replace("0 ignored","1 ignored")
        return subprocess.CompletedProcess(argv,1 if mode=="exit" else 0,out,"authored control")
    monkeypatch.setattr(support.subprocess,"run",fake_run)
    with pytest.raises(AssertionError): run_rust_case(Path("/authored/native"),case,tmp_path)

@pytest.mark.parametrize("component",["flag","rng","pile","order","public","legal","count"])
def test_one_field_corruption_is_detected(component):
    left=run_python_case(fixed_case("cultist")); right=deepcopy(left); p=right["checkpoints"][3]
    if component=="flag": p["state"]["enemies"][0]["powers"][0]["skip_first"]=True
    if component=="rng": p["state"]["rng"]["shuffle"]["word_index"]="99"
    if component=="pile": p["state"]["piles"]["hand"].pop()
    if component=="order": p["state"]["piles"]["hand"].reverse()
    if component=="public": p["public"]["player"]["hp"]=1
    if component=="legal": p["legal"].reverse()
    if component=="count": p["state"]["resolved_effects"]+=1
    with pytest.raises(AssertionError,match=r"case=.*checkpoints\[3\]"): assert_case_equal(left,right)
