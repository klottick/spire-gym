import groovy.json.JsonSlurper
import java.io.File
import java.security.MessageDigest
import org.gradle.api.plugins.JavaPluginExtension

// The real in-game capture mod is BLOCKED on user-provided evidence AND a real,
// reviewed, reference-dependent capture integration. Everything here is
// fail-closed and, until such an integration exists, BLOCKED UNCONDITIONALLY.
// A dummy/empty/unrelated .java file does NOT unblock anything (S3); the
// "reference-dependent capture integration is pending" gate is removed only as
// part of a future audited real-adapter implementation carrying actual reference
// evidence — never via a boolean, filename match, or source-count trick.
//
// R2: the mod build compiles with an EXPLICITly pinned build compiler (the
// local JDK 17) and a declared --release (bytecode/API level, NOT the build JDK).
// The current :protocol targets release 17; a runtime below 17 is a named
// protocol-compatibility blocker (it would require a later, evidence-backed
// protocol retarget). A valid synthetic target is NOT game evidence.
plugins {
    `java-library`
}

repositories {
    mavenCentral()
}

// --- R2: pinned build facts (not game evidence) ---
val PROTOCOL_RELEASE = 17            // current :protocol bytecode target
val BUILD_JDK_MAJOR = 17             // installed local build toolchain (tools/jdk17)
val SUPPORTED_RELEASES = 7..17       // tools/jdk17 `javac --help`: supported --release

// Bind the build compiler EXPLICITLY to the local JDK 17 (NOT the game runtime
// target, NOT a portable default inherited from protocol).
java {
    toolchain { languageVersion.set(JavaLanguageVersion.of(BUILD_JDK_MAJOR)) }
}

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

// Parse a canonical target (ASCII positive integer, no leading zero) from the
// reference; null on absent/non-string/non-numeric/leading-zero/overflow.
fun referenceInt(key: String): Int? {
    val ref = runCatching { JsonSlurper().parse(referenceBuildFile) }.getOrNull()
    if (ref !is Map<*, *>) return null
    val v = ref[key]
    if (v !is String) return null
    if (v.isEmpty() || v.any { it !in '0'..'9' }) return null
    if (v.length > 1 && v[0] == '0') return null
    val n = v.toIntOrNull()
    if (n == null || n <= 0) return null
    return n
}

// R2: bind JavaCompile.options.release to the declared compiler target when it
// is structurally valid. Unknown/invalid metadata is never silently defaulted to
// ambient bytecode: if it is invalid the readiness gate fails before any compile.
val declaredCompilerRelease = referenceInt("compiler_target")
if (declaredCompilerRelease != null) {
    tasks.withType<JavaCompile>().configureEach {
        options.release.set(declaredCompilerRelease)
    }
}

// R2: diagnostic (disposable) inspection of the target configuration. Runs
// WITHOUT removing the integration blocker and WITHOUT pretending a capture
// implementation exists; it only inspects the declared binding.
tasks.register("inspectCompileTarget") {
    doLast {
        val compile = tasks.named("compileJava").get() as JavaCompile
        val rel = compile.options.release.orNull
        val toolchainVer = runCatching {
            project.extensions.getByType<JavaPluginExtension>().toolchain.languageVersion.get().toString()
        }.getOrNull() ?: "unspecified"
        logger.lifecycle("inspectCompileTarget: options.release=$rel (declared=$declaredCompilerRelease)")
        logger.lifecycle("inspectCompileTarget: toolchain.languageVersion=$toolchainVer (pinned build JDK 17)")
    }
}

val referenceReadiness = tasks.register("referenceReadiness") {
    group = "verification"
    description =
        "Fail-closed check that a real capture mod can honestly be built. Every " +
        "unmet requirement is reported independently, and the build is " +
        "additionally blocked unconditionally while reference-dependent capture " +
        "integration is pending."
    doLast {
        val missing = mutableListOf<String>()
        fun fail(reason: String) { missing += "[MOD-READINESS] $reason" }

        // Canonical target shape: ASCII positive integer, no leading zero.
        fun targetInt(value: Any?, name: String): Int? {
            if (value !is String) {
                fail("$name must be a positive decimal integer string " +
                     "(got ${value?.let { it.javaClass.simpleName } ?: "null"})")
                return null
            }
            if (value.isEmpty() || value.any { it !in '0'..'9' }) {
                fail("$name = \"$value\" must be a positive decimal integer feature version " +
                     "(ASCII digits only, no fraction/sign/letters)")
                return null
            }
            if (value.length > 1 && value[0] == '0') {
                fail("$name = \"$value\" has a leading zero (ambiguous Java feature version)")
                return null
            }
            val v = value.toIntOrNull()
            if (v == null) {
                fail("$name = \"$value\" is not a valid positive integer (overflow?)")
                return null
            }
            if (v <= 0) {
                fail("$name = \"$value\" is zero or negative")
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

            // S1 — required game/loader objects (presence + shape).
            if (!("game" in ref) || ref["game"] == null) {
                fail("game object is required (a JSON object with version and sha256)")
            } else if (ref["game"] !is Map<*, *>) {
                fail("game must be a JSON object (got ${(ref["game"] as Any).javaClass.simpleName})")
            } else {
                val gv = (ref["game"] as Map<*, *>)["version"]
                if (gv !is String || gv.isBlank()) {
                    fail("game.version is required (a nonblank version ID)")
                }
                if (!isHex64((ref["game"] as Map<*, *>)["sha256"])) {
                    fail("game.sha256 is not a 64-char lowercase hex SHA-256")
                }
            }
            if (!("loader" in ref) || ref["loader"] == null) {
                fail("loader object is required (a JSON object with version and sha256)")
            } else if (ref["loader"] !is Map<*, *>) {
                fail("loader must be a JSON object (got ${(ref["loader"] as Any).javaClass.simpleName})")
            } else {
                val lv = (ref["loader"] as Map<*, *>)["version"]
                if (lv !is String || lv.isBlank()) {
                    fail("loader.version is required (a nonblank version ID)")
                }
                if (!isHex64((ref["loader"] as Map<*, *>)["sha256"])) {
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
            if (isHex64((ref["game"] as? Map<*, *>)?.get("sha256")) &&
                isHex64(ref["game_sha256"]) &&
                (ref["game"] as? Map<*, *>)?.get("sha256") != ref["game_sha256"]
            ) {
                fail("game.sha256 and game_sha256 disagree (nested/flat metadata contradiction)")
            }
            if (isHex64((ref["loader"] as? Map<*, *>)?.get("sha256")) &&
                isHex64(ref["loader_sha256"]) &&
                (ref["loader"] as? Map<*, *>)?.get("sha256") != ref["loader_sha256"]
            ) {
                fail("loader.sha256 and loader_sha256 disagree (nested/flat metadata contradiction)")
            }

            // R1 — optional flat version aliases: KEY-PRESENCE based (absent is
            // allowed); a supplied value (including null, wrong type, blank, or a
            // mismatch) is rejected with its own diagnostic.
            if ("game_version" in ref) {
                val alias = ref["game_version"]
                if (alias == null) {
                    fail("game_version is supplied as null (omit it, or supply a string matching game.version)")
                } else if (alias !is String) {
                    fail("game_version if supplied must be a string (got ${alias.javaClass.simpleName})")
                } else if (alias.isBlank()) {
                    fail("game_version if supplied must be a nonblank string")
                } else {
                    val gv = (ref["game"] as? Map<*, *>)?.get("version")
                    if (alias != gv) {
                        fail("game_version differs from game.version (optional alias must match the nested version)")
                    }
                }
            }
            if ("loader_version" in ref) {
                val alias = ref["loader_version"]
                if (alias == null) {
                    fail("loader_version is supplied as null (omit it, or supply a string matching loader.version)")
                } else if (alias !is String) {
                    fail("loader_version if supplied must be a string (got ${alias.javaClass.simpleName})")
                } else if (alias.isBlank()) {
                    fail("loader_version if supplied must be a nonblank string")
                } else {
                    val lv = (ref["loader"] as? Map<*, *>)?.get("version")
                    if (alias != lv) {
                        fail("loader_version differs from loader.version (optional alias must match the nested version)")
                    }
                }
            }

            // S2 — canonical target representation + constraints.
            val runtimeV = targetInt(ref["runtime_target"], "runtime_target")
            val compilerV = targetInt(ref["compiler_target"], "compiler_target")
            if (runtimeV != null && runtimeV < PROTOCOL_RELEASE) {
                fail("runtime_target ($runtimeV) is below the current protocol bytecode " +
                     "release ($PROTOCOL_RELEASE): the protocol is compiled to release $PROTOCOL_RELEASE, so a " +
                     "lower runtime requires a later evidence-backed protocol retarget, not a claim")
            }
            if (compilerV != null && compilerV !in SUPPORTED_RELEASES) {
                fail("compiler_target ($compilerV) is not a release supported by the pinned " +
                     "local build compiler (tools/jdk17 supports --release ${SUPPORTED_RELEASES.first()}-" +
                     "${SUPPORTED_RELEASES.last()})")
            }
            if (runtimeV != null && compilerV != null && compilerV > runtimeV) {
                fail("compiler_target ($compilerV) is incompatible with runtime_target ($runtimeV): " +
                     "bytecode cannot target a newer JVM than the game runs")
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
        // not a source-count check, not a flag. Removed only inside a future
        // audited real-adapter implementation with actual reference evidence.
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
                "consistent reference-build.json with real digests and compatible " +
                "toolchain-selectable targets, AND a real reference-dependent " +
                "capture implementation in this subproject before any :mod artifact " +
                "may be produced. See docs/validation/capture-guide.md."
        )
    }
}

// S2/S3 — gate the artifact AND the compile path. A reference-dependent
// integration must exist (and verified targets, later selected) before any :mod
// compile/jar/assemble/build can run. Valid synthetic metadata is not enough.
tasks.named("compileJava") { dependsOn(referenceReadiness) }
tasks.named("jar") { dependsOn(referenceReadiness) }
tasks.named("assemble") { dependsOn(referenceReadiness) }
tasks.named("build") { dependsOn(referenceReadiness) }
