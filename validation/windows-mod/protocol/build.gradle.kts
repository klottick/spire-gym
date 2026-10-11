plugins {
    `java-library`
}

repositories {
    mavenCentral()
}

dependencies {
    // Pinned exactly (lock): BOM pins every org.junit.* coordinate; exact
    // transitive checksums are verified against Maven Central metadata
    // (see the task-2 report) using a local worktree Gradle user home.
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
