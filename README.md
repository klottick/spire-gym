# Spire Lab

A Slay the Spire 1 simulation and machine-learning project, with a Rust
simulator and Python training code in one repository.

## Status

Design phase. No simulator or training implementation exists yet.

The [written design](docs/superpowers/specs/2026-10-11-spire-lab-ml-first-design.md)
is ready for review; it is not yet approved for implementation planning.

## Confirmed goals

- Prioritize efficient ML training: learning progress on held-out scenarios per
  unit of wall-clock time, backed by game-correct simulation.
- Build the simulator in Rust; learning Rust remains a secondary benefit.
- Use Python for model training.
- Develop a simulation tool for learning to play Slay the Spire.
- Make Heart victory the ultimate agent goal; report Act 3 completion separately.
- Plan the design and implementation before building the application.

## Confirmed roadmap

1. Act 1-ready Ironclad combat simulation and training (the starting milestone).
2. Act 1 Ironclad runs.
3. Full-game Ironclad runs including keys, Act 4, and the Heart.
4. Full-game Ironclad across Ascension 0–20, targeting Heart victory.
5. Repeat the preceding progression for Silent.
6. Extend the same capabilities to the remaining characters, Defect and Watcher.
7. Support Daily run modifiers under their original mode rules, without
   requiring Heart access where the mode does not permit it.

Detailed content coverage and acceptance criteria will be designed per phase.
The initial implementation plan will target slice 1a (starter cards versus
Cultist/Jaw Worm), not the entire roadmap.
Phase 1 will support explicitly scoped Act 1 combat scenarios; complete
Ironclad combat through the Heart is not a prerequisite for phase 2.

## Approved architectural direction

- Rust-native game-specific core with a batched numeric interface to Python.
- Train and evaluate agents using player-visible information, not hidden RNG
  state or unseen future outcomes.
- Start with a small, validated end-to-end training slice, then expand content.
- Use existing simulators as references, not unquestioned correctness oracles.
- Keep game fidelity, deterministic replay, and measurable training throughput
  central to the design.

These are architectural principles, not an approved complete spec or
implementation plan. Concrete contracts, pilot defaults, and acceptance procedures
are proposed in the written design for review.

See [the design checklist](docs/design-process.md) for progress and decisions.
