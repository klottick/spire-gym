# M19 bounded Dark selection fact correction

Approved M19 Task0 requires resolving Dark's exact constructor and HalfDead
continuation chain before affected behavior. The primary conversation authorized
this correction before any M19 original outcome. Approved M16/M18/M19 planning
documents, historical reports, accepted source exports and receipts retain their
original bytes and revision-qualified evidence. This addendum records the newly
resolved fact; it does not recast those historical runs as corrected proof.

The older local-scan finding correctly identified a call to `isDeadOrEscaped`
and the absence of a separate HalfDead comparison inside Dark's constructor.
It omitted the inherited helper's semantics. Exact named-member inspection shows
that neither AbstractMonster nor Darkling overrides that helper; its
AbstractCreature declaration returns true for dying or HalfDead actors, then
checks escaping. Consequently a Dark evoke constructed while an actor is already
HalfDead excludes that actor. Equal eligible HP retains formation-list order,
without a random choice.

A target captured while eligible is a different boundary. Dark's queued update
retains its captured reference and does not select again. Its THORNS damage path
can reach that reference after the actor becomes HalfDead. Darkling's existing
half-death guard suppresses duplicate lifecycle callbacks. Selection eligibility
and accepted queued continuation must therefore be tested separately.

The shared `combat/characters.rs` selector now excludes already-HalfDead actors.
The source-disproven M16/M18 test expectations are replaced with stronger checks
of exact eligible-target damage, retained HalfDead state and unchanged lifecycle
counts. Their original full source preimages and incorrect expectations remain
independently retained. M19 additionally tests constructor exclusion, formation
ties, capture before HP changes and delivery after HalfDeath with no duplicate
callback. The existing damage/continuation owner is unchanged.

The private9401-byte named-chain report SHA256 is
`5787f494009e1cf9fb8e75f3da5c2f7da2e635de9121c7227219e7406ba84a00`;
its28-input and18-output manifests bind exact hashes and sizes. The inspected
owned archive SHA256 is
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`,
using javap21.0.12.1. This is bounded static evidence for those bytes. Upstream
build/channel identity and original-runtime fidelity remain unverified. No broad
audit, new admission, policy tuning or previous original-policy rerun follows.

The same report resolves the named TimeWarp/Echo/Amplify chain: the current use
finishes, queued autoplay siblings receive suppressed cleanup, and the common
end-turn handoff follows retained accepted parent work. M19 composition tests
exercise the existing shared cancellation and unwinding owner; no blue-specific
scheduler or new repeat rule is introduced.
