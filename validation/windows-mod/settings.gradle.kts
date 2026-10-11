// Spire Lab Windows capture project (portable, game-independent stage).
//
// Two subprojects:
//   :protocol — the trace-v1 wire protocol shared by the Rust parser
//     (crates/spire-core) and the (later) in-game capture mod. Portable and
//     tested on any JVM >= 17.
//   :mod      — the real in-game capture mod. Its build is FAIL-CLOSED:
//     :mod:jar cannot succeed until the user-provided pinned
//     game/loader artifacts and a verified reference-build.json exist.
//
// Game binaries, assets, and decompiled game source are NEVER committed.
// The user supplies -PgameJar and -PmodTheSpireJar at build time; both
// paths point outside this repository.
rootProject.name = "spire-lab-windows-mod"

include(":protocol")
include(":mod")
