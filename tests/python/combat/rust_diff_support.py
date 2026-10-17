"""Test-only authored interchange, complete checkpoints and fail-closed orchestration.

This is neither a snapshot wire format nor a product/capture transport.
"""
from copy import deepcopy
from dataclasses import asdict
from pathlib import Path
import json
import os
import re
import subprocess
import sys
import tempfile

from spire_lab.combat import engine
from spire_lab.combat.model import (CardInstance, EnemySetup, PlayerState, Piles,
    PowerState, RngState, Scenario, PlayCard, EndTurn, InvariantError,
    UnsupportedMechanicError, InvalidActionError, FaultedStateError)
from spire_lab.combat.rng import RNG_CONTRACT_ID, RandomStreamState, StreamId, make_stream
from spire_lab.combat.snapshot import snapshot, restore
from spire_lab.combat.view import public_view, legal_actions

SCHEMA="spire-lab-combat-diff-v1"
ROOT=Path(__file__).resolve().parents[3]
EVIDENCE=ROOT/".superpowers/sdd/2026-10-16-source-first-rust-plans/task-8"
PYTHON_CASE_WORKER=Path(__file__).with_name("rust_diff_python_child.py")


class DecodeError(Exception):
    def __init__(self,kind): self.kind=kind


def require(ok,kind="interchange"):
    if not ok: raise DecodeError(kind)


def fields(value,names):
    require(type(value) is dict and set(value)==set(names.split()))
    return value


def text(v):
    require(type(v) is str)
    require(not any(0xD800<=ord(c)<=0xDFFF for c in v))
    return v


def integer(v):
    require(type(v) is int and -(2**31)<=v<2**31)
    return v


def boolean(v):
    require(type(v) is bool)
    return v


def array(v):
    require(type(v) is list)
    return v


def finite(v,allowed):
    text(v); require(re.fullmatch(r"[a-z][a-z0-9]*(?:_[a-z0-9]+)*",v) is not None)
    require(v in allowed.split(),"unsupported")
    return v


def decimal(v,signed=True):
    text(v); require(re.fullmatch(r"0|-?[1-9][0-9]*" if signed else r"0|[1-9][0-9]*",v) is not None)
    # Keep adapter helpers exact even in the parent with CPython's decimal
    # conversion guard enabled. Every individual conversion is only nine digits.
    digits=v.removeprefix("-"); value=0
    for start in range(0,len(digits),9):
        chunk=digits[start:start+9]; value=value*10**len(chunk)+int(chunk)
    return -value if v.startswith("-") else value


def decimal_handle(value):
    require(type(value) is int)
    negative=value<0; value=abs(value); chunks=[]
    while value>=10**9:
        value,remainder=divmod(value,10**9); chunks.append(f"{remainder:09d}")
    return ("-" if negative else "")+str(value)+"".join(reversed(chunks))


def owner(v):
    text(v); require(v not in ("","player"),"invariant")
    return v


def decode_card(c):
    fields(c,"instance_id card_id upgraded")
    return CardInstance(decimal(c["instance_id"]),finite(c["card_id"],"strike defend bash"),boolean(c["upgraded"]))


def decode_powers(value):
    result={}
    for p in array(value):
        fields(p,"id amount skip_first monster_source_grace")
        name=finite(p["id"],"strength ritual vulnerable")
        require(name not in result,"invariant")
        result[name]=PowerState(integer(p["amount"]),boolean(p["skip_first"]),boolean(p["monster_source_grace"]))
    return result


def decode_stream(s):
    fields(s,"domain scope key_hex word_index")
    domain=text(s["domain"]); require(domain in ("combat.shuffle","combat.enemy.ai","combat.enemy.hp"))
    scope=tuple(text(x) for x in array(s["scope"]))
    key=text(s["key_hex"]); require(re.fullmatch("[0-9a-f]{64}",key) is not None)
    cursor=decimal(s["word_index"],False); require(cursor<=2**64)
    try: return RandomStreamState(StreamId(domain,scope),bytes.fromhex(key),cursor)
    except (ValueError,TypeError) as exc: raise DecodeError("invariant") from exc


def decode_request(case):
    # Scalar-text validity is an encoding property of the entire request, not
    # gameplay action decoding. Rust's JSON reader cannot represent surrogates.
    def scalar_text(value):
        if type(value) is str: text(value)
        elif type(value) is list:
            for item in value: scalar_text(item)
        elif type(value) is dict:
            for key,item in value.items(): scalar_text(key); scalar_text(item)
    scalar_text(case)
    fields(case,"schema case_id scenario actions")
    require(text(case["schema"])==SCHEMA and bool(text(case["case_id"])))
    actions=array(case["actions"]); s=fields(case["scenario"],"encounter_id player enemies piles rng")
    p=fields(s["player"],"hp max_hp block energy max_energy draw_per_turn powers")
    player=PlayerState(*(integer(p[n]) for n in ("hp","max_hp","block","energy","max_energy","draw_per_turn")),decode_powers(p["powers"]))
    enemies=[]
    for e in array(s["enemies"]):
        fields(e,"instance_id enemy_id hp max_hp block powers intent_id move_history")
        move=lambda v: finite(v,"incantation attack chomp thrash bellow")
        enemies.append(EnemySetup(owner(e["instance_id"]),finite(e["enemy_id"],"cultist jaw_worm"),
            integer(e["hp"]),integer(e["max_hp"]),integer(e["block"]),decode_powers(e["powers"]),
            move(e["intent_id"]),tuple(move(v) for v in array(e["move_history"]))))
    p=fields(s["piles"],"hand draw discard exhaust")
    piles=Piles(*([decode_card(c) for c in array(p[n])] for n in ("hand","draw","discard","exhaust")))
    r=fields(s["rng"],"contract_id enemy_instance_id shuffle enemy_ai")
    rng=RngState(text(r["contract_id"]),decode_stream(r["shuffle"]),decode_stream(r["enemy_ai"]),owner(r["enemy_instance_id"]))
    return Scenario(finite(s["encounter_id"],"cultist jaw_worm"),player,tuple(enemies),piles,rng),actions


def decode_action(a):
    require(type(a) is dict and "kind" in a)
    kind=finite(a["kind"],"play_card end_turn")
    if kind=="end_turn": fields(a,"kind"); return EndTurn()
    fields(a,"kind card_instance_id target_id")
    handle=decimal(a["card_instance_id"])
    target=a["target_id"]
    if target is not None:
        text(target); require(target not in ("","player"),"invalid_action")
    return PlayCard(handle,target)


def encode_action(a):
    return ({"kind":"end_turn"} if type(a) is EndTurn else
        {"kind":"play_card","card_instance_id":decimal_handle(a.card_instance_id),"target_id":a.target_id})


def encode_card(c):
    return {"instance_id":decimal_handle(c.instance_id),"card_id":c.card_id,"upgraded":c.upgraded}


def encode_stream(s):
    return {"domain":s.stream_id.domain,"scope":list(s.stream_id.scope),"key_hex":s.key.hex(),"word_index":str(s.word_index)}


def encode_powers(p):
    return [{"id":name,**asdict(state)} for name,state in p.items()]


def encode_effect(e):
    if e is None: return None
    source=(None if e.source_id is None else {"card":decimal_handle(e.source_id)} if type(e.source_id) is int else {"actor":e.source_id})
    return {"kind":e.kind,"source":source,"target":e.target_id,"payload":dict(e.payload)}


def normalize_python_checkpoint(state):
    p=state.player
    return {
        "content_id":state.content_id,"phase":state.phase,"turn":state.turn,
        "player":{**asdict(p),"powers":encode_powers(p.powers)},
        "enemies":[{**asdict(e),"powers":encode_powers(e.powers),"move_history":list(e.move_history),
            "damage_entries":{k:asdict(v) for k,v in sorted(e.damage_entries.items())}} for e in state.enemies],
        "piles":{n:[encode_card(c) for c in getattr(state.piles,n)] for n in ("hand","draw","discard","exhaust")},
        "card_ids_issued":sorted(decimal_handle(i) for i in state.card_ids_issued),
        "rng":{"contract_id":state.rng.contract_id,"shuffle":encode_stream(state.rng.shuffle),
            "enemy_ai":encode_stream(state.rng.enemy_ai),"enemy_instance_id":state.rng.enemy_instance_id},
        "pending":[encode_effect(e) for e in state.pending],"current":encode_effect(state.current),
        "in_play":None if state.in_play is None else encode_card(state.in_play),
        "resolved_effects":state.resolved_effects,"public":{"revealed_order":list(state.public.revealed_order)},
        "outcome":None if state.outcome is None else state.outcome.kind,
        "fault":None if state.fault is None else {"kind":state.fault.kind.value,"resolved_effects":state.fault.resolved_effects},
        "hp_before_healing":state.hp_before_healing,"hp_after_cleanup":state.hp_after_cleanup,
    }


def record(state,index,rejection=None):
    view=legal=None
    if state.fault is None:
        v=public_view(state); view=asdict(v)
        view["player"]["powers"]=[list(p) for p in v.player.powers]
        view["enemies"]=[{**asdict(e),"powers":[list(p) for p in e.powers]} for e in v.enemies]
        for n in ("hand","discard","exhaust"): view[n]=[encode_card(c) for c in getattr(v,n)]
        view["revealed_order"]=list(v.revealed_order)
        legal=[encode_action(a) for a in legal_actions(state)]
    status="rejected" if rejection else "fault" if state.fault else "terminal" if state.outcome else "decision"
    return {"index":index,"status":status,"rejection":rejection,"state":normalize_python_checkpoint(state),"public":view,"legal":legal}


def submit(state,raw):
    try: action=decode_action(raw)
    except DecodeError as exc: return exc.kind
    if state.fault is not None: return "faulted"
    if state.phase=="terminal": return "terminal"
    if state.phase!="decision": return "not_ready"
    try: engine.step(state,action)
    except InvalidActionError: return "invalid_action"
    except FaultedStateError: return "faulted"
    return None


def run_python_case(case):
    # CPython permits a process decimal limit as low as640. Route longer
    # canonical decimal text regardless of this parent's current configuration.
    # The engine itself formats revealed handles, beyond adapter-only control.
    def needs_child(value):
        if type(value) is str:
            return len(value.removeprefix("-"))>640 and re.fullmatch(r"0|-?[1-9][0-9]*",value) is not None
        if type(value) is list: return any(needs_child(v) for v in value)
        if type(value) is dict: return any(needs_child(v) for v in value.values())
        return False
    if not needs_child(case): return _run_python_case(case)
    stage=fresh_stage("python-case")
    request=(stage/"request.json").resolve(); response=(stage/"response.json").resolve()
    save(request,json.dumps(case,ensure_ascii=True,indent=2)+"\n")
    # Startup flag affects only this disposable child: no sys setter, environment
    # mutation, ID remapping, int subclass, or gameplay monkeypatch in the parent.
    run_logged([sys.executable,"-B","-X","int_max_str_digits=0",str(PYTHON_CASE_WORKER),str(request),str(response)],
        stage,timeout=30,env=os.environ.copy())
    assert response.is_file(),f"Python comparison response missing; evidence {stage}"
    result=json.loads(response.read_text())
    assert set(result)=={"schema","case_id","entry_error","checkpoints"},f"bad Python response; {stage}"
    assert result["schema"]==SCHEMA and result["case_id"]==case.get("case_id",""),f"stale Python response; {stage}"
    return result


def _run_python_case(case):
    result={"schema":SCHEMA,"case_id":case.get("case_id","") if type(case) is dict else "","entry_error":None,"checkpoints":[]}
    try:
        scenario,actions=decode_request(case); state=engine.initial_combat(scenario)
    except (DecodeError,InvariantError,UnsupportedMechanicError) as exc:
        result["entry_error"]=exc.kind if type(exc) is DecodeError else "invariant" if type(exc) is InvariantError else "unsupported"
        return result
    result["checkpoints"].append(record(state,0))
    for index,raw in enumerate(actions,1):
        saved=snapshot(state); baseline=normalize_python_checkpoint(state)
        replay=restore(saved) if state.fault is None else None
        rejection=submit(state,raw)
        if replay is not None:
            assert rejection==submit(replay,raw)
            assert record(state,index,rejection)==record(replay,index,rejection)
            assert normalize_python_checkpoint(saved.state)==baseline
        result["checkpoints"].append(record(state,index,rejection))
    return result


def fixed_case(enemy,*,death=False):
    hp,first=(48,"incantation") if enemy=="cultist" else (40,"chomp")
    stream=lambda domain,scope: encode_stream(make_stream(bytes(32),"test","combat-behavior-plan",StreamId(domain,scope)))
    cards=[{"instance_id":str(i+1),"card_id":name,"upgraded":True} for i,name in enumerate(("bash","strike","defend"))]
    if death: cards=[{"instance_id":"1","card_id":"defend","upgraded":False}]
    scenario={"encounter_id":enemy,"player":{"hp":(14 if enemy=="cultist" else 10) if death else 40,"max_hp":20 if death else 50,"block":0,"energy":3,"max_energy":3,"draw_per_turn":1 if death else 3,"powers":[]},
        "enemies":[{"instance_id":"enemy-1","enemy_id":enemy,"hp":hp,"max_hp":hp,"block":0,"powers":[],"intent_id":first,"move_history":[first]}],
        "piles":{"hand":cards,"draw":[],"discard":[],"exhaust":[]},
        "rng":{"contract_id":RNG_CONTRACT_ID,"enemy_instance_id":"enemy-1","shuffle":stream("combat.shuffle",("combat-plan-1",)),"enemy_ai":stream("combat.enemy.ai",("combat-plan-1","enemy-scope-1"))}}
    if death and enemy=="jaw_worm": scenario["rng"]["enemy_ai"]["word_index"]=str(2**64)
    play=lambda i:{"kind":"play_card","card_instance_id":str(i),"target_id":"enemy-1"}
    actions=([{"kind":"end_turn"}]*(3 if enemy=="cultist" else 1) if death else [play(1),play(2),{"kind":"end_turn"},play(1),play(2)])
    return {"schema":SCHEMA,"case_id":f"fixed-{enemy}-{'death' if death else 'victory'}","scenario":scenario,"actions":actions}


def walk_case(enemy,episode):
    case=fixed_case(enemy); case["case_id"]=f"walk-{enemy}-{episode}"; case["scenario"]["player"]["draw_per_turn"]=5
    for n in ("shuffle","enemy_ai"):
        old=case["scenario"]["rng"][n]
        case["scenario"]["rng"][n]=encode_stream(make_stream(bytes(32),"test",f"combat-vertical-{episode}",StreamId(old["domain"],tuple(old["scope"]))))
    cards=[{"instance_id":str(i+1),"card_id":("bash","strike","defend")[i%3],"upgraded":(i+episode)%2!=0} for i in range(9)]
    case["scenario"]["piles"]["hand"]=cards[:3];case["scenario"]["piles"]["draw"]=cards[3:]
    scenario,_=decode_request(case); state=engine.initial_combat(scenario);case["actions"]=[]
    for step in range(200):
        choices=legal_actions(state); action=choices[(episode+step)%len(choices)]
        case["actions"].append(encode_action(action));engine.step(state,action)
        require(state.fault is None,"invariant")
        cards_now=[c for n in ("hand","draw","discard","exhaust") for c in getattr(state.piles,n)]+([state.in_play] if state.in_play else [])
        assert len(cards_now)==9 and {c.instance_id for c in cards_now}==state.card_ids_issued==set(range(1,10))
        if state.outcome: break
    else: raise AssertionError("authored legal walk exceeded 200 actions")
    return case


def fresh_stage(prefix):
    EVIDENCE.mkdir(parents=True,exist_ok=True)
    return Path(tempfile.mkdtemp(prefix=prefix+"-",dir=EVIDENCE))


def save(path,text):
    with path.open("x",encoding="utf-8") as f: f.write(text)


def run_logged(argv,stage,*,timeout,env):
    save(stage/"command.json",json.dumps(argv)+"\n")
    try: result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,timeout=timeout)
    except (OSError,subprocess.TimeoutExpired) as exc:
        save(stage/"subprocess-error.txt",repr(exc)+"\n"+str(getattr(exc,"stdout",None))+"\n"+str(getattr(exc,"stderr",None)))
        raise AssertionError(f"native subprocess failed; evidence {stage}") from exc
    save(stage/"stdout.log",result.stdout);save(stage/"stderr.log",result.stderr);save(stage/"exit.txt",str(result.returncode)+"\n")
    assert result.returncode==0,f"native subprocess exit {result.returncode}; evidence {stage}"
    return result


def parse_test_executable(stdout,manifest):
    matches=[]
    for line in stdout.splitlines():
        if not line.startswith("{"): continue
        try: a=json.loads(line)
        except json.JSONDecodeError as exc: raise AssertionError("invalid Cargo JSON metadata") from exc
        if (a.get("reason")=="compiler-artifact" and a.get("manifest_path")==str(manifest.resolve())
            and a.get("target",{}).get("name")=="spire_core" and a.get("target",{}).get("kind")==["lib"]
            and a.get("profile",{}).get("test") is True and a.get("executable") is not None):
            matches.append(Path(a["executable"]))
    assert len(matches)==1,f"expected exactly one matching native test artifact, got {matches}"
    assert matches[0].is_absolute() and matches[0].is_file(),"native artifact executable missing"
    return matches[0]


def build_rust_executable():
    stage=fresh_stage("build")
    argv=["cargo","test","-p","spire-core","--lib","--no-run","--locked","--offline","--message-format=json"]
    r=run_logged(argv,stage,timeout=180,env={**os.environ,"CARGO_NET_OFFLINE":"true"})
    return parse_test_executable(r.stdout,ROOT/"crates/spire-core/Cargo.toml")


def run_rust_case(executable,case,directory):
    # Persist even failed real comparisons outside pytest's disposable scratch tree.
    stage=fresh_stage("case-"+re.sub(r"[^A-Za-z0-9_-]","_",directory.name)[:60])
    request=(stage/"request.json").resolve(); response=(stage/"response.json").resolve()
    save(request,json.dumps(case,ensure_ascii=True,indent=2)+"\n")
    save(stage/"python-response.json",json.dumps(run_python_case(case),ensure_ascii=True,indent=2)+"\n")
    env={**os.environ,"CARGO_NET_OFFLINE":"true","SPIRE_COMBAT_DIFF_REQUEST":str(request),"SPIRE_COMBAT_DIFF_RESPONSE":str(response)}
    r=run_logged([str(executable),"--exact","combat::tests::differential::protocol","--nocapture"],stage,timeout=30,env=env)
    summaries=re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",r.stdout)
    assert summaries==[("1","0","0")],f"not exactly one passing/zero ignored native tests; evidence {stage}"
    assert response.is_file(),f"native response missing; evidence {stage}"
    try: result=json.loads(response.read_text())
    except (ValueError,OSError) as exc: raise AssertionError(f"invalid native response; evidence {stage}") from exc
    assert set(result)=={"schema","case_id","entry_error","checkpoints"},f"bad response fields; {stage}"
    assert result["schema"]==SCHEMA and result["case_id"]==case["case_id"],f"stale response identity; {stage}"
    expected=[] if result["entry_error"] else list(range(len(case["actions"])+1))
    assert [p["index"] for p in result["checkpoints"]]==expected,f"incomplete native checkpoint indices; {stage}"
    return result


def assert_case_equal(left,right):
    def first(a,b,path):
        if type(a) is not type(b): return path,a,b
        if type(a) is dict:
            if set(a)!=set(b): return path+".fields",list(a),list(b)
            for k in a:
                diff=first(a[k],b[k],path+"."+k)
                if diff: return diff
        elif type(a) is list:
            if len(a)!=len(b): return path+".length",len(a),len(b)
            for i,(x,y) in enumerate(zip(a,b)):
                diff=first(x,y,f"{path}[{i}]")
                if diff:return diff
        elif a!=b:return path,a,b
        return None
    diff=first(left,right,"")
    assert diff is None,f"case={left.get('case_id')} first divergence {diff}"
