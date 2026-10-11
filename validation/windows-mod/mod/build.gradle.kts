import groovy.json.JsonSlurper
import java.io.File

// The real in-game capture mod is BLOCKED on user-provided evidence:
//   * a pinned Slay the Spire 1 game jar (-PgameJar, outside this repo),
//   * a pinned loader jar (-PmodTheSpireJar, outside this repo),
//   * a verified docs/validation/reference-build.json (status "verified"
//     with actual game/loader SHA-256s and runtime/compiler targets).
//
// Nothing here is a no-op adapter: the subproject contains no capture
// source, and every artifact path (jar/assemble/build) is fail-closed on
// the readiness check below. This is deliberate — a "successful" :mod:jar
// without the pinned reference data would be a fabricated artifact.
plugins {
    `java-library`
}

repositories {
    mavenCentral()
}

val referenceBuildFile =
    rootProject.projectDir
        .parentFile
        .parentFile
        .resolve("docs/validation/reference-build.json")

val referenceReadiness = tasks.register("referenceReadiness") {
    group = "verification"
    description =
        "Fail-closed check that the pinned reference build and external jars are available."
    doLast {
        val missing = mutableListOf<String>()

        val gameJar = providers.gradleProperty("gameJar").orNull
        if (gameJar.isNullOrBlank()) {
            missing += "user-provided -PgameJar is not set"
        } else if (!File(gameJar).isFile) {
            missing += "-PgameJar does not point to an existing file"
        }

        val modJar = providers.gradleProperty("modTheSpireJar").orNull
        if (modJar.isNullOrBlank()) {
            missing += "user-provided -PmodTheSpireJar is not set"
        } else if (!File(modJar).isFile) {
            missing += "-PmodTheSpireJar does not point to an existing file"
        }

        if (!referenceBuildFile.isFile) {
            missing += "docs/validation/reference-build.json is missing"
        } else {
            val parsed = runCatching { JsonSlurper().parse(referenceBuildFile) }.getOrNull()
            if (parsed !is Map<*, *>) {
                missing += "reference-build.json is not a parseable JSON object"
            } else {
                val ref = parsed
                if (ref["status"] != "verified") {
                    missing += "reference-build.json status is \"${ref["status"]}\" (need \"verified\")"
                }
                val gameSha = ref["game_sha256"]
                if (gameSha !is String || !gameSha.matches(Regex("^[0-9a-f]{64}$"))) {
                    missing += "reference-build.json game_sha256 is missing or not 64 lowercase hex"
                }
                val loaderSha = ref["loader_sha256"]
                if (loaderSha !is String || !loaderSha.matches(Regex("^[0-9a-f]{64}$"))) {
                    missing += "reference-build.json loader_sha256 is missing or not 64 lowercase hex"
                }
            }
        }

        if (missing.isEmpty()) return@doLast
        val detail = missing.joinToString("\n  - ")
        throw org.gradle.api.GradleException(
            "capture mod readiness FAILED (fail-closed):\n  - $detail\n" +
                "Provide the pinned game/loader jars and a verified reference-build.json. " +
                "See docs/validation/capture-guide.md."
        )
    }
}

// Fail-closed: no mod artifact may be produced without verified reference
// data and the user's external dependencies.
tasks.named("jar") {
    dependsOn(referenceReadiness)
}
tasks.named("assemble") {
    dependsOn(referenceReadiness)
}
tasks.named("build") {
    dependsOn(referenceReadiness)
}
