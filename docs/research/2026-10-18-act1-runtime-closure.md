# Act 1 runtime probe capability: bounded negative finding

**Status: original-class execution remains unapproved and unestablished.** This
research found no installed, reviewed combination that presently satisfies the
required filesystem, network, process, native-loading, profile, and per-target
initialization/dependency boundaries. This is a capability limit, not evidence
about game mechanics. No game class, initializer, launcher, native game library,
save, Steam/cloud integration, or network-enabled probe was run.

## What is established

Prior reviewed evidence supports only a narrow trusted non-game canary. Installed
`setpriv` Landlock filesystem restrictions combined with a pinned JDK startup
permission guard passed the recorded scratch/outside-path controls. A JVM-only
negative control allowed both controlled symlink read and write escapes; the
outside fixture mutation was retained. The independent review reproduced the
bounded canary outcomes and accepted the negative capability report, not a game
runner. These checks do not establish a general sandbox.

The current installed util-linux `setpriv` help and man page expose
`--seccomp-filter <file>` for loading raw BPF filters. A narrowly scoped setup
check using an empty filter and a missing filter both returned 127 without
executing the requested harmless `/usr/bin/true` target. This verifies
fail-closed behavior for those setup errors only. It does **not** test a
functioning filter, prove syscall coverage, or demonstrate
network/process/native confinement. The current process observation reported
`Seccomp: 0` and `Seccomp_filters: 0`; this is an
observation, not proof that the kernel lacks seccomp support. The prior observed
AppArmor label was `unconfined`; no profile was established or changed. Namespace
creation was already recorded as unavailable and was not retried.

## Why the available pieces do not compose into approval

- Landlock is a useful inherited filesystem layer, but the tested rules and broad
  read allowances are not an exact target dependency closure. They do not filter
  network, process, or native syscalls. JVM permission checks are not a kernel
  network barrier; the earlier “network” canary checked a permission gate before
  socket use and made no connection.
- A raw seccomp filter is present as a facility, but no reviewed filter was
  supplied or exercised. A filter must be designed for the pinned architecture,
  JVM startup/runtime, and exact target; accidental denials can break startup,
  while permissive holes defeat the intended boundary. It cannot be credited as
  confinement merely because `setpriv` advertises the option. No syscall profile
  was authored in this spike.
- The JDK SecurityManager canary is deprecated, trusts platform/native code, and
  only covered finite application permission checks. Existing process/native
  checks were pre-API denials, not syscall-level tests. No actual socket, native
  library, or external process was used by those checks.
- No reviewed AppArmor confinement profile, network namespace, process-tree
  supervisor, or hostile-code boundary is established. Direct-child timeout is
  not process-tree termination or resource isolation. HOME/user.home redirection
  is hygiene, not confinement; inherited descriptors and other kernel interfaces
  remain relevant.
- No per-target class-initialization or dependency/permission closure exists.
  Before any candidate, an independent reviewer would need an exact pinned target
  and classpath inventory, unique archive-entry checks, transitive class and
  native dependency accounting, initializer/path/permission analysis, explicit
  profile/output roots, and a guard applied before target loading. Dynamic
  reflection, service loading, native code, and environment-sensitive behavior
  must be handled or excluded by the bounded target contract. Never widen a
  sandbox ad hoc to make initialization succeed.

## Decision and handoff boundary

Fail closed: continue static and model work; do not launch original or reconstructed
game code, even if a new helper or candidate file appears. A future proposal must
be freshly reviewed for the combined process-local filesystem, network, syscall,
process/native, profile, descriptor, descendant-timeout, and target-closure
properties, then explicitly handed off by main before any game execution. This
document is not that review, runner, handoff, approval, or security assurance.

Private evidence for this bounded investigation is retained under
`verification/2026-10-18-research-pr-wave-v2/runtime/` in the private verification
workspace: `evidence/seccomp-setup-controls.json` and
`evidence/environment-observation.json`. Prior immutable canary and independent
review evidence remains under
`verification/2026-10-18-act1-followup-v1/{isolation,isolation-review}/`; the
accepted disposition is in `verification/2026-10-18-act1-followup-v1/controller/isolation-disposition.md`.
No private source, helper, fixture, raw game artifact, or detailed log is included
here. No public test suite was run because no product code changed.
