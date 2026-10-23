# Pre-M8 public repository snapshot

**Request:** commit the current public work and make the new reference resources
easy for AI agents to find before M8. This authorizes local commits, not a push,
merge to main, cleanup, dependency installation or M8 implementation.

## Checklist

- [x] Commit the accepted M7 source, tests, example and milestone documentation
  on the existing `feat/run-act2-prefix` branch. Verify against its reviewed
  snapshot and rerun focused acceptance checks; preserve the seven historical
  core failures rather than claiming a green workspace.
- [x] Materialize the existing main-checkout `data/{cards,relics,enemies,powers}`
  and five reference test files unchanged on this branch. Retain newer branch
  versions of overlapping project documents; do not overwrite main-checkout work.
  Add `data/README.md` and prominent `AGENTS.md`/root README discovery links,
  explaining source grades, uncertainties, licenses and offline checks.
- [x] Run the standard-library reference tests, offline regeneration checks and
  bounded Rust checks. Inspect the exact staged file list and diff, then make a
  separate reference/docs commit. Confirm local commit contents and clean branch
  status. Exclude `.superpowers/`, tracker state/locks, control logs, private
  artifacts and build outputs. Leave original checkouts and worktrees intact.

Reference integrity is not simulator behavior acceptance. Reuse the completed M7
milestone review; this snapshot does not authorize another implementation/review
cycle or broaden the accepted profile. M8 remains planning-only afterward.

## Verification and disposition

- Accepted M7 commit: `2d9a47e`; reviewed crate bytes unchanged.
- Reference snapshot: 31 unchanged public data/builder/test files from main,
  plus the new index; source hashes matched in both checkouts before committing.
- Reference tests: 51 pass; card370/power133 regeneration checks match exactly.
- Rust: public50/0, M7internal16/0, M6native90/0, consumers43/0, other21/0,
  reporting1/0, seven examples and the preselected seed11 paths pass. Budget0/1
  correctly fail as unfinished. Workspace core534/7 retains exactly the same
  underlying seven historical failures; this is not a whole-workspace-green claim.
- Discovery links resolve; whitespace checks pass. The separate reference/docs
  commit records these files without changing the original main checkout.
- Older main project documents were already present on this branch, either
  identical or superseded by later accepted milestone updates. Do not replace
  those updates with stale main snapshots just to make documents byte-identical.
- Local tracker/evidence/lock/control/build files remain uncommitted and intact.
  Nothing is pushed or merged; no M8 implementation was started.
