// Root conventions.
//
// Portable-stage scope: only the protocol subproject inherits the
// "portable" JVM toolchain (release 17). This is a **portable** choice
// that has no claim of compatibility with the actual pinned Slay the Spire 1
// runtime; the game-runtime / compiler target for the :mod subproject stays
// **pending**, carried by the (unverified) `runtime_target` and
// `compiler_target` fields in docs/validation/reference-build.json and
// enforced by :mod's fail-closed referenceReadiness predicate.
//
// We do NOT silently apply a toolchain to :mod here: an unverified portable
// release must not be read as a "verified game target".
allprojects {
    group = "spirelab"
    version = "0.1.0"

    // I8: strict, build-enforced resolved-graph locks for every subproject.
    // Every configuration is locked and STRICT enforcement means an unlocked
    // (or changed) configuration fails resolution instead of silently
    // re-resolving. Combined with gradle/verification-metadata.xml (SHA-256)
    // this is what makes a fresh build enforce both the reviewed dependency
    // graph and the exact artifact digests. Generated/refreshed deliberately
    // with --write-locks / --write-verification-metadata (see
    // docs/validation/capture-guide.md); never rewritten silently.
    dependencyLocking {
        lockAllConfigurations()
        lockMode.set(LockMode.STRICT)
    }
}

subprojects {
    plugins.withId("java") {
        tasks.withType<Test>().configureEach {
            useJUnitPlatform()
            testLogging {
                events("failed", "skipped")
                showStandardStreams = false
                exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
            }
        }
    }
}

// Portable release-17 toolchain, scoped to :protocol only (see header).
subprojects {
    if (name == "protocol") {
        plugins.withId("java") {
            extensions.configure<
                    org.gradle.api.plugins.JavaPluginExtension>(
                "java"
            ) { toolchain.languageVersion.set(JavaLanguageVersion.of(17)) }
            tasks.withType<JavaCompile>().configureEach {
                options.release.set(17)
                options.encoding = "UTF-8"
            }
        }
    }
}
