"""Optional Gymnasium adapters for the native combat simulator."""

from .single import CombatEnv
from .vector import CombatVectorEnv

__all__ = ["CombatEnv", "CombatVectorEnv"]
