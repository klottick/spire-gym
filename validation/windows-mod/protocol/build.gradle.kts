plugins {
    `java-library`
}

repositories {
    mavenCentral()
}

dependencies {
    // The BOM pins the org.junit.* versions; the *exact* resolved graph is
    // further enforced by dependency locking (see the committed
    // protocol/gradle.lockfile, generated with --write-locks) and the exact
    // artifact digests by gradle/verification-metadata.xml (SHA-256, generated
    // with --write-verification-metadata sha256). Both are build-enforced and
    // reconciled against Maven Central. See docs/validation/capture-guide.md (§5a).
    testImplementation(platform("org.junit:junit-bom:5.13.4"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

// The protocol library has zero compile-time dependencies: the wire JSON is
// handled by spirelab.protocol.Json, and SHA-256 fingerprints use the JDK
// MessageDigest. Keeping the portable stage dependency-free makes its
// artifact verifiable and its failure modes explicit.
java {
    withSourcesJar()
}
