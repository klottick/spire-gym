import groovy.json.JsonSlurper
import java.io.File
import java.security.MessageDigest

// The real in-game capture mod is BLOCKED on user-provided evidence AND a real,
// reviewed, reference-dependent capture integration:
//   * a pinned Slay the Spire 1 game jar (-PgameJar, outside this repo),
//   * a pinned loader jar (-PmodTheSpireJar, outside this repo),
//   * a verified, internally consistent docs/validation/reference-build.json
//     (supported schema, status "verified", boolean verified:true, nonblank
//     game/loader version IDs, matching nested/flat SHA-256s, and compatible,
//     toolchain-selectable runtime/compiler targets),
//   * and an actual reference-dependent capture implementation in this
//     subproject.
//
// Everything here is deliberately fail-closed and, until a real capture
// integration exists, it is BLOCKED UNCONDITIONALLY. Adding a dummy/empty/unrelated
// .java file does NOT unblock it, and there is NO boolean, filename match, or
// source-count trick to bypass it: the "reference-dependent capture integration
// is pending" gate is the sole remaining gate and it is removed/replaced only
// inside a future audited real-adapter implementation carrying actual reference
// evidence. That is honest incomplete tooling, not a successful mod stub.
//
// :mod does NOT inherit the portable release-17 toolchain. If and when the
// integration gate is legitimately removed, its compiler target must be
// selected from the verified `compiler_target` in reference-build.json (not
// silently inherited from protocol). No portable release is applied here.
plugins {
    `java-library`
}

repositories {
    mavenCentral()
}

// NOTE: no toolchain/options.release is configured for :mod. Only :protocol
// carries the portable release-17 choice. The :mod game-runtime/compiler target
// (and the integration that would honour it) is pending.

val referenceBuildFile: File =
    providers.gradleProperty("referenceBuild").orNull
        ?.let { File(it) }
        ?: rootProject.projectDir.parentFile.parentFile
            .resolve("docs/validation/reference-build.json")

// Real SHA-256 of a file, lowercase hex.
fun fileSha256(path: String): String {
    val md = MessageDigest.getInstance("SHA-256")
    md.update(File(path).readBytes())
    return md.digest().joinToString("") { "%02x".format(it) }
}

val referenceReadiness = tasks.register("referenceReadiness") {
    group = "verification"
    description =
        "Fail-closed check that a real capture mod can honestly be built. " +
        "Every unmet requirement is reported independently, and the build is " +
        "additionally blocked unconditionally while reference-dependent capture " +
        "integration is pending."
    doLast {
        val missing = mutableListOf<String>()
        fun fail(reason: String) { missing += "[MOD-READINESS] $reason" }

        // Canonical target shape: a strict positive decimal integer (Java
        // feature version) — no letters, no fraction, no sign, no leading zero.
        fun targetInt(value: Any?, name: String): Int? {
            if (value !is String) {
                fail("$name must be a positive integer feature version (a string) — got ${value?.let { it.javaClass.simpleName } ?: "null"}")
                return null
            }
            if (value.isEmpty() || value.any { !it.isDigit() }) {
                fail("$name must be a positive integer feature version (digits only, no fraction/sign) — got \"$value\"")
                return null
            }
            if (value.length > 1 && value[0] == '0') {
                fail("$name has a leading zero (ambiguous Java feature version) — got \"$value\"")
                return null
            }
            val v = value.toIntOrNull()
            if (v == null || v <= 0) {
                fail("$name is zero or negative — got \"$value\"")
                return null
            }
            return v
        }

        // --- pinned jars present (their real digests are compared below) ---
        val gameJar = providers.gradleProperty("gameJar").orNull
        if (gameJar.isNullOrBlank()) {
            fail("game jar is not provided (-PgameJar not set)")
        } else if (!File(gameJar).isFile) {
            fail("game jar is not provided (-PgameJar does not point to an existing file)")
        }

        val modJar = providers.gradleProperty("modTheSpireJar").orNull
        if (modJar.isNullOrBlank()) {
            fail("loader jar is not provided (-PmodTheSpireJar not set)")
        } else if (!File(modJar).isFile) {
            fail("loader jar is not provided (-PmodTheSpireJar does not point to an existing file)")
        }

        val ref: Any? = if (referenceBuildFile.isFile) {
            runCatching { JsonSlurper().parse(referenceBuildFile) }.getOrNull()
        } else {
            fail("reference-build.json is missing (docs/validation/reference-build.json)")
            null
        }

        if (ref is Map<*, *>) {
            val hex = Regex("^[0-9a-f]{64}$")
            fun isHex64(v: Any?): Boolean = v is String && hex.matches(v)

            // S1 — supported schema, verified status, boolean verified:true.
            if (ref["schema"] != "spire-lab/reference-build/1") {
                fail("schema is not supported: expected \"spire-lab/reference-build/1\", got \"${ref["schema"]}\"")
            }
            if (ref["status"] != "verified") {
                fail("status is \"${ref["status"]}\" (need \"verified\")")
            }
            if (ref["verified"] != true) {
                fail("verified must be boolean true (got ${ref["verified"]})")
            }

            // S1 — required game/loader objects with nonblank version IDs and
            // valid nested SHA-256s (each an independent blocker).
            val game = ref["game"]
            if (game !is Map<*, *>) {
                fail("game object is required (game.version, game.sha256)")
            } else {
                val gv = game["version"]
                if (gv !is String || gv.isBlank()) {
                    fail("game.version is required (a nonblank version ID)")
                }
                if (!isHex64(game["sha256"])) {
                    fail("game.sha256 is not a 64-char lowercase hex SHA-256")
                }
            }
            val loader = ref["loader"]
            if (loader !is Map<*, *>) {
                fail("loader object is required (loader.version, loader.sha256)")
            } else {
                val lv = loader["version"]
                if (lv !is String || lv.isBlank()) {
                    fail("loader.version is required (a nonblank version ID)")
                }
                if (!isHex64(loader["sha256"])) {
                    fail("loader.sha256 is not a 64-char lowercase hex SHA-256")
                }
            }

            // S1 — flat required hashes + nested/flat agreement (independent).
            if (!isHex64(ref["game_sha256"])) {
                fail("game_sha256 is not a 64-char lowercase hex SHA-256")
            }
            if (!isHex64(ref["loader_sha256"])) {
                fail("loader_sha256 is not a 64-char lowercase hex SHA-256")
            }
            val gNest = (ref["game"] as? Map<*, *>)?.get("sha256")
            val lNest = (ref["loader"] as? Map<*, *>)?.get("sha256")
            if (isHex64(gNest) && isHex64(ref["game_sha256"]) && gNest != ref["game_sha256"]) {
                fail("game.sha256 and game_sha256 disagree (nested/flat metadata contradiction)")
            }
            if (isHex64(lNest) && isHex64(ref["loader_sha256"]) && lNest != ref["loader_sha256"]) {
                fail("loader.sha256 and loader_sha256 disagree (nested/flat metadata contradiction)")
            }

            // S1 — optional flat version aliases: if supplied, typed and matching.
            val gameVerAlias = ref["game_version"]
            if (gameVerAlias != null) {
                val gv = (ref["game"] as? Map<*, *>)?.get("version")
                if (gameVerAlias !is String) {
                    fail("game_version if supplied must be a string")
                } else if (gv != gameVerAlias) {
                    fail("game_version differs from game.version (optional alias must match the nested version)")
                }
            }
            val loaderVerAlias = ref["loader_version"]
            if (loaderVerAlias != null) {
                val lv = (ref["loader"] as? Map<*, *>)?.get("version")
                if (loaderVerAlias !is String) {
                    fail("loader_version if supplied must be a string")
                } else if (lv != loaderVerAlias) {
                    fail("loader_version differs from loader.version (optional alias must match the nested version)")
                }
            }

            // S2 — strict compatible target representation.
            val runtimeV = targetInt(ref["runtime_target"], "runtime_target")
            val compilerV = targetInt(ref["compiler_target"], "compiler_target")
            if (runtimeV != null && compilerV != null && compilerV > runtimeV) {
                fail("compiler_target is incompatible with runtime_target " +
                     "(compiler=$compilerV > runtime=$runtimeV: bytecode cannot target a newer JVM than the game runs)")
            }
            // S2 — target must be honouable by the selected local build toolchain.
            val localMajor = org.gradle.api.JavaVersion.current().majorVersion.toIntOrNull()
            if (compilerV != null && localMajor != null && compilerV > localMajor) {
                fail("compiler_target is higher than the local JDK " +
                     "(target=$compilerV > local=$localMajor): the selected local build " +
                     "toolchain cannot target it — a real game target requires a matching " +
                     "JDK/toolchain during integration")
            }

            // S1 — retain actual-file digest comparisons.
            if (gameJar != null && File(gameJar).isFile && isHex64(ref["game_sha256"])) {
                val actual = fileSha256(gameJar)
                if (actual != ref["game_sha256"]) {
                    fail("game jar SHA-256 does not match the supplied file " +
                         "(claimed ${ref["game_sha256"].toString().take(16)}…, actual ${actual.take(16)}…)")
                }
            }
            if (modJar != null && File(modJar).isFile && isHex64(ref["loader_sha256"])) {
                val actual = fileSha256(modJar)
                if (actual != ref["loader_sha256"]) {
                    fail("loader jar SHA-256 does not match the supplied file " +
                         "(claimed ${ref["loader_sha256"].toString().take(16)}…, actual ${actual.take(16)}…)")
                }
            }
        } else if (referenceBuildFile.isFile) {
            fail("reference-build.json is not a parseable JSON object")
        }

        // S3 — UNCONDITIONAL integration-pending gate. Not a filename heuristic,
        // not a source-count check, not a flag. Removing this gate is part of a
        // future audited real-adapter implementation with actual reference
        // evidence only.
        fail("reference-dependent capture integration is pending — this subproject " +
             "has NO real, reviewed, reference-dependent capture implementation, so " +
             "no :mod artifact (jar/assemble/build) is produced. A real adapter " +
             "implementation with actual reference evidence must remove this gate " +
             "(not a filename match, boolean, or source-count heuristic)")

        if (missing.isEmpty()) return@doLast
        val detail = missing.joinToString("\n  ")
        throw org.gradle.api.GradleException(
            "capture mod readiness FAILED (fail-closed):\n  $detail\n" +
                "Provide the pinned game/loader jars, a verified and internally " +
                "consistent reference-build.json, compatible toolchain-selectable " +
                "targets, AND a real reference-dependent capture implementation in " +
                "this subproject before any :mod artifact may be produced. " +
                "See docs/validation/capture-guide.md."
        )
    }
}

// S2/S3 — gate the artifact AND the compile path before compilation: a
// reference-dependent integration must exist (and, later, verified targets
// selected) before any :mod compile/jar/assemble/build can run.
tasks.named("compileJava") { dependsOn(referenceReadiness) }
tasks.named("jar") { dependsOn(referenceReadiness) }
tasks.named("assemble") { dependsOn(referenceReadiness) }
tasks.named("build") { dependsOn(referenceReadiness) }
