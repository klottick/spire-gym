"""M36 freeze (design §3.8, I5): the frozen Python names. Names only, no signatures. Removing or renaming a frozen name
needs user approval; an additive name updates these sets with a CHANGES.md line (docs/delivery.md, change protocol)."""

import ast
from pathlib import Path

import spire_lab.gymnasium as gymnasium
from spire_lab import _native

GYMNASIUM_EXPORTS = {
    "CombatEnv", "CombatVectorEnv", "IroncladCombatEnv", "IroncladCombatVectorEnv", "PublicBranch", "PublicSearch",
    "RunEnv", "RunVectorEnv", "StandardCombatEnv", "StandardCombatVectorEnv",
}
NATIVE_CALLABLES = {
    "BatchFaultError", "CombatFaultError", "CombatInputError", "CombatSetupError", "CombatStateError", "ExactCheckpoint",
    "NativeCombatPool", "NativeCombatTrainingBatch", "NativeIroncladCombatSession", "NativePublicSearch",
    "NativeRunBatch", "NativeRunSession", "NativeStandardCombatSession", "RunBudgetError", "RunFaultError",
    "RunInputError", "RunStateError", "RunUnsupportedBoundaryError", "build_info", "card_metadata",
}
BUILD_INFO_KEYS = {"dirty", "extension", "features", "file_digest", "profile", "revision", "rust_version", "source_digest"}


def _stub_callables():
    """Public classes and functions of `_native.pyi`, excluding TypedDict payload types (directly or by inheritance)."""
    tree = ast.parse(Path(_native.__file__).with_name("_native.pyi").read_text(encoding="utf-8"))
    typed_dicts, names = {"TypedDict"}, set()
    for node in tree.body:
        if isinstance(node, ast.ClassDef):
            if {ast.unparse(base).split(".")[-1] for base in node.bases} & typed_dicts:
                typed_dicts.add(node.name)
            elif not node.name.startswith("_"):
                names.add(node.name)
        elif isinstance(node, ast.FunctionDef) and not node.name.startswith("_"):
            names.add(node.name)
    return names


def test_gymnasium_exports_are_frozen():
    assert set(gymnasium.__all__) == GYMNASIUM_EXPORTS


def test_native_callables_are_frozen_and_the_stub_agrees():
    runtime = {name for name in dir(_native) if not name.startswith("_") and callable(getattr(_native, name))}
    assert runtime == NATIVE_CALLABLES
    assert _stub_callables() == NATIVE_CALLABLES


def test_build_info_keys_are_frozen():
    assert set(_native.build_info()) == BUILD_INFO_KEYS
