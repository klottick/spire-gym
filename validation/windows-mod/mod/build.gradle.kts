import groovy.json.JsonSlurper
import java.io.File
import java.security.MessageDigest

// The real in-game capture mod is BLOCKED on user-provided evidence and a real
// capture implementation:
//   * a pinned Slay the Spire 1 game jar (-PgameJar, outside this repo),
//   * a pinned loader jar (-PmodTheSpireJar, outside this repo),
//   * a verified docs/validation/reference-build.json (status "verified",
//     consistent nested/flat hashes, and real runtime/compiler targets),
//   * actual capture source living in this subproject.
//
// Everything here is deliberately fail-closed. This subproject currently has
// NO capture source, so even a fully-populated, internally-consistent,
// status-"verified" reference-build.json with present, correctly-hashed jars
// CANNOT produce a `:mod` artifact: a "successful" empty jar without a real
// capture implementation would be a fabricated artifact. The readiness gate
// lists every unmet requirement; it has NO boolean an implementer can flip to
// bypass the source requirement.
//
// It also does NOT inherit the portable release-17 toolchain: its real JVM /
// compiler target is whatever the pinned game requires (the
// `runtime_target`/`compiler_target` in reference-build.json), which is
// pending. No portable release is silently applied to it here.
plugins {
    `java-library`
}

repositories {
    mavenCentral()
}

// NOTE: no JavaExtension.toolchain / options.release is configured for :mod.
// The portable release-17 choice is a property of the :protocol subproject
// only (see protocol/build.gradle.kts). The game-runtime target for :mod stays
// pending until reference-build.json carries verified, real targets.

// Default location is the repo's docs/validation/reference-build.json (two
// levels above the root project dir). The focused isolated test harness
// overrides it per-case via -PreferenceBuild so it never mutates the committed
// pending file; both resolve to the same logical path here.
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
        "Fail-closed check that a real capture mod can honestly be built: " +
        "pinned jars present with matching digests, a verified and internally " +
        "consistent reference-build.json, verified runtime/compiler targets, and " +
        "an actual capture source. Any unmet requirement blocks the build."
    doLast {
        val missing = mutableListOf<String>()
        fun fail(reason: String) { missing += "[MOD-READINESS] $reason" }

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

        // --- reference-build.json: verified status, internally consistent ---
        val ref = if (referenceBuildFile.isFile) {
            runCatching { JsonSlurper().parse(referenceBuildFile) }.getOrNull()
        } else {
            fail("reference-build.json is missing (docs/validation/reference-build.json)")
            null
        }
        if (ref is Map<*, *>) {
            // Schema + status gate.
            if (ref["status"] != "verified") {
                fail("reference-build.json status is \"${ref["status"]}\" (need \"verified\")")
            }

            // A verified reference must carry real, matching, non-null hashes,
            // never a hash-shaped empty string.
            val topGame = ref["game_sha256"]
            val topLoader = ref["loader_sha256"]
            val nest = ref["game"] as? Map<*, *>
            val nestLoader = ref["loader"] as? Map<*, *>
            val nestGameSha = nest?.get("sha256")
            val nestLoaderSha = nestLoader?.get("sha256")

            val hex = Regex("^[0-9a-f]{64}$")
            if (topGame is String && hex.matches(topGame) &&
                nestGameSha is String && hex.matches(nestGameSha) &&
                nestGameSha != topGame
            ) {
                fail("game.sha256 and game_sha256 disagree (nested/flat metadata contradiction)")
            }
            if (topLoader is String && hex.matches(topLoader) &&
                nestLoaderSha is String && hex.matches(nestLoaderSha) &&
                nestLoaderSha != topLoader
            ) {
                fail("loader.sha256 and loader_sha256 disagree (nested/flat metadata contradiction)")
            }

            if (topGame !is String || !hex.matches(topGame)) {
                fail("game_sha256 is not a 64-char lowercase hex SHA-256")
            }
            if (topLoader !is String || !hex.matches(topLoader)) {
                fail("loader_sha256 is not a 64-char lowercase hex SHA-256")
            }

            // Real runtime/compiler targets must be present (a verified
            // reference implies a real game whose runtime target is known).
            val runtimeTarget = ref["runtime_target"]
            val compilerTarget = ref["compiler_target"]
            if (runtimeTarget !is String || runtimeTarget.isBlank() ||
                compilerTarget !is String || compilerTarget.isBlank()
            ) {
                fail("runtime_target and compiler_target are both required; the pending " +
                    "portable release-17 toolchain is not a verified game target and must not " +
                    "be inherited")
            }

            // The claimed hashes must match the ACTUAL digest of the supplied jars.
            if (gameJar != null && File(gameJar).isFile && topGame is String && hex.matches(topGame)) {
                val actual = fileSha256(gameJar)
                if (actual != topGame) {
                    fail("game jar SHA-256 does not match the supplied file " +
                        "(claimed $topGame, actual ${actual.take(16)}…)")
                }
            }
            if (modJar != null && File(modJar).isFile && topLoader is String && hex.matches(topLoader)) {
                val actual = fileSha256(modJar)
                if (actual != topLoader) {
                    fail("loader jar SHA-256 does not match the supplied file " +
                        "(claimed $topLoader, actual ${actual.take(16)}…)")
                }
            }
        } else if (referenceBuildFile.isFile) {
            fail("reference-build.json is not a parseable JSON object")
        }

        // --- the hard, non-bypassable gate: an actual capture source must exist ---
        val srcRoot = projectDir.resolve("src/main/java")
        val captureSources = if (srcRoot.isDirectory) {
            srcRoot.walkTopDown().filter { it.isFile && it.extension == "java" }.toList()
        } else {
            emptyList()
        }
        if (captureSources.isEmpty()) {
            fail("capture source (implemented capture mod) is not present; a source-less " +
                "mod artifact is not produced — this subproject contains no capture " +
                "implementation yet, so a `:mod` jar cannot be a real capture mod")
        }

        if (missing.isEmpty()) return@doLast
        val detail = missing.joinToString("\n  ")
        throw org.gradle.api.GradleException(
            "capture mod readiness FAILED (fail-closed):\n  $detail\n" +
                "Provide the pinned game/loader jars, a verified and internally consistent " +
                "reference-build.json with real digests and real runtime/compiler targets, " +
                "AND an actual capture implementation in this subproject before a :mod jar " +
                "may be produced. See docs/validation/capture-guide.md."
        )
    }
}

// Fail-closed: no mod artifact may be produced without satisfied readiness.
tasks.named("jar") {
    dependsOn(referenceReadiness)
}
tasks.named("assemble") {
    dependsOn(referenceReadiness)
}
tasks.named("build") {
    dependsOn(referenceReadiness)
}
