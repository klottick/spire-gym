// Root conventions. Subprojects apply `java` themselves.
allprojects {
    group = "spirelab"
    version = "0.1.0"
}

subprojects {
    plugins.withId("java") {
        // Portable toolchain for this stage (JVM >= 17). The real in-game
        // mod subproject must additionally target the pinned game's JVM
        // (pending user evidence in reference-build.json); this portable
        // build is NOT a verified game/runtime pair.
        extensions.configure<org.gradle.api.plugins.JavaPluginExtension>("java") {
            toolchain.languageVersion.set(JavaLanguageVersion.of(17))
        }
        tasks.withType<JavaCompile>().configureEach {
            options.release.set(17)
            options.encoding = "UTF-8"
        }
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
