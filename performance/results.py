"""Strict, self-contained result contracts and recalculable summaries."""
import hashlib
import json
import math
import re
import statistics
from fractions import Fraction
from pathlib import Path

BOUNDARIES = {"native-transitions", "public-decision-loop"}


def canonical_bytes(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value):
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def integer(value, low, high, name):
    require(type(value) is int and low <= value <= high, f"invalid {name}")


def sha(value):
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value), "invalid SHA-256")


def validate_catalog(catalog):
    require(type(catalog["schema"]) is int and catalog["schema"] == 1, "unknown catalog schema")
    target = catalog["target"]
    require(target["characters"] == ["ironclad", "silent", "defect", "watcher"] and
            target["ascensions"] == [0, 20] and target["acts"] == [1, 4] and
            target["goal"] == "heart" and target["primary_ascension"] == 20, "invalid design target")
    require(isinstance(catalog["cases"], list) and catalog["cases"], "empty catalog")
    ids = set()
    for case in catalog["cases"]:
        require(isinstance(case["id"], str) and case["id"] and case["id"] not in ids, "duplicate/empty catalog ID")
        ids.add(case["id"])
        require(case["status"] in {"planned", "executable"}, "invalid case status")
        integer(case["version"], 1, 2**31 - 1, "version")
        if case["status"] == "executable":
            integer(case["ascension"], 0, 20, "ascension")
            integer(case["seed"], 0, 2**64 - 1, "seed")
            integer(case["expected_contract_version"], 1, 2**31 - 1, "expected contract")
            require(case["character"] in target["characters"], "unknown character")
            require(isinstance(case["profile"], str) and case["profile"], "missing profile")
            require(isinstance(case["creation"], dict) and isinstance(case["expected"], dict), "missing contract")
        require(isinstance(case["requires"], list), "invalid requirements")
    family_ids = set()
    for family in catalog["families"]:
        require(family["id"] not in family_ids and family["id"] not in ids, "duplicate family")
        family_ids.add(family["id"])
        require(family["status"] in {"planned", "partial", "executable"}, "invalid family status")
        require(family["requires"] and family["milestones"], "missing family capability/roadmap")
    canonical_bytes(catalog)
    return catalog


def load_catalog(path: Path):
    try:
        return validate_catalog(json.loads(path.read_text()))
    except (KeyError, TypeError, AttributeError, OverflowError) as error:
        raise ValueError(f"malformed catalog: {error}") from error


def validate_result(document):
    try:
        _validate_result(document)
    except (KeyError, TypeError, AttributeError, OverflowError) as error:
        raise ValueError(f"malformed result: {error}") from error
    return document


def _validate_result(doc):
    require(doc["schema"] in {"spire-gym-native-samples-v1", "spire-gym-performance-v1"}, "unknown result schema")
    catalog = validate_catalog(doc["catalog"])
    require(doc["catalog_sha256"] == digest(catalog), "catalog hash mismatch")
    contracts = {case["id"]: case for case in catalog["cases"]}
    sampling = doc["sampling"]
    for key, low, high in [("warmup", 0, 100), ("samples", 1, 100), ("iterations", 1, 1000)]:
        integer(sampling[key], low, high, key)
    require(isinstance(doc["cases"], list) and doc["cases"], "no measured cases")
    seen = set()
    for case in doc["cases"]:
        key = (case["id"], case["boundary"])
        require(key not in seen, "duplicate result row")
        seen.add(key)
        require(case["boundary"] in BOUNDARIES, "unknown timing boundary")
        contract = contracts[case["id"]]
        require(contract["status"] == "executable", "planned case cannot have samples")
        require(case["contract_sha256"] == digest(contract), "contract hash mismatch")
        for field in ["version", "profile", "character", "ascension", "seed", "expected_contract_version"]:
            require(canonical_bytes(case[field]) == canonical_bytes(contract[field]), f"contract mismatch: {field}")
        require(case["preflight_passed"] is True, "failed/missing preflight")
        for field in ["input_sha256", "actions_sha256"]:
            sha(case[field])
        integer(case["batch_size"], 1, 2**31 - 1, "batch size")
        integer(case["workers"], 1, case["batch_size"], "workers")
        integer(case["accepted_per_episode"], 1, 4096, "actions/episode")
        integer(case["preparation_ns"], 0, 2**64 - 1, "preparation duration")
        require(isinstance(case["endpoint"], dict), "invalid endpoint")
        for field, expected in contract["expected"].items():
            require(field in case["endpoint"] and canonical_bytes(case["endpoint"][field]) == canonical_bytes(expected),
                    f"behavioral endpoint mismatch: {field}")
        if "accepted_actions" in case["endpoint"]:
            require(case["endpoint"]["accepted_actions"] == case["accepted_per_episode"], "endpoint action mismatch")
        require(len(case["samples"]) == sampling["samples"], "sample count mismatch")
        for sample in case["samples"]:
            integer(sample["elapsed_ns"], 1, 2**64 - 1, "elapsed ns")
            integer(sample["episodes"], 1, 1000, "episodes")
            integer(sample["accepted_actions"], 1, 4096 * 1000, "accepted actions")
            require(sample["episodes"] == sampling["iterations"], "iteration count mismatch")
            require(sample["accepted_actions"] == sample["episodes"] * case["accepted_per_episode"], "action count mismatch")
    if doc["schema"] == "spire-gym-performance-v1":
        for field in ["runner_id", "cpu", "os", "arch"]:
            require(isinstance(doc["machine"][field], str), f"missing machine {field}")
        build = doc["build"]
        for field in ["rustc_vv", "target"]:
            require(isinstance(build[field], str), f"invalid build {field}")
        require(isinstance(build["profile"], dict) and isinstance(build["flags"], list), "invalid build settings")
        sha(build["config_sha256"])
        provenance = doc["provenance"]
        require(isinstance(provenance["git_revision"], str) and
                re.fullmatch(r"[0-9a-f]{40,64}", provenance["git_revision"]), "invalid revision")
        require(type(provenance["dirty"]) is bool, "invalid dirty status")
        sha(provenance["source_sha256"])
        seconds = provenance["build_seconds"]
        require(type(seconds) in {int, float} and math.isfinite(seconds) and seconds >= 0, "invalid build duration")
    canonical_bytes(doc)  # Also rejects nonfinite values nested in unknown/endpoint fields.


def median_ns_per_action(samples):
    return statistics.median(Fraction(s["elapsed_ns"], s["accepted_actions"]) for s in samples)


def summarize(samples):
    ratios = [Fraction(s["elapsed_ns"], s["accepted_actions"]) for s in samples]
    return {"median_elapsed_ns": float(statistics.median(s["elapsed_ns"] for s in samples)),
            "median_ns_per_action": float(statistics.median(ratios)),
            "median_actions_per_second": float(statistics.median(Fraction(10**9, 1) / r for r in ratios)),
            "min_ns_per_action": float(min(ratios)), "max_ns_per_action": float(max(ratios))}
