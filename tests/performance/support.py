"""Authored result fixtures; intentionally independent of the native runner."""
import copy
import hashlib
import json


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, ensure_ascii=False,
                                     separators=(",", ":")).encode()).hexdigest()


def result():
    contract = {"id": "test.combat", "version": 1, "status": "executable",
                "profile": "test", "character": "ironclad", "ascension": 0,
                "seed": 7, "expected_contract_version": 1, "creation": {},
                "expected": {"outcome": "victory"}, "requires": []}
    catalog = {"schema": 1, "target": {"characters": ["ironclad", "silent", "defect", "watcher"],
                "ascensions": [0, 20], "acts": [1, 4], "goal": "heart", "primary_ascension": 20},
               "cases": [contract], "families": []}
    return {"schema": "spire-gym-performance-v1", "catalog": catalog,
            "catalog_sha256": digest(catalog), "sampling": {"warmup": 1, "samples": 2, "iterations": 1},
            "provenance": {"git_revision": "a" * 40, "dirty": False, "source_sha256": "b" * 64, "build_seconds": 0.1},
            "machine": {"runner_id": "test-host", "cpu": "test-cpu", "os": "linux", "arch": "x86_64"},
            "build": {"rustc_vv": "rustc test", "target": "x86_64-linux", "profile": {"opt_level": "3"},
                      "flags": [], "config_sha256": "c" * 64},
            "cases": [{"id": "test.combat", "version": 1, "contract_sha256": digest(contract),
                       "boundary": "native-transitions", "profile": "test", "character": "ironclad",
                       "ascension": 0, "seed": 7, "batch_size": 1, "workers": 1,
                       "input_sha256": "d" * 64, "actions_sha256": "e" * 64, "accepted_per_episode": 10,
                       "expected_contract_version": 1, "endpoint": {"outcome": "victory"},
                       "preflight_passed": True, "preparation_ns": 1,
                       "samples": [{"elapsed_ns": 1000, "episodes": 1, "accepted_actions": 10},
                                   {"elapsed_ns": 2000, "episodes": 1, "accepted_actions": 10}]}]}


def changed(document=None):
    return copy.deepcopy(result() if document is None else document)
