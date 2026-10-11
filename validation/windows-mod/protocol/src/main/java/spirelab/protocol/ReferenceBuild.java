package spirelab.protocol;

import java.util.Locale;

/** SHA-256 of the pinned game and loader binaries the trace was captured in. */
public final class ReferenceBuild {
    private final String gameSha256;
    private final String loaderSha256;

    public ReferenceBuild(String gameSha256, String loaderSha256) {
        this.gameSha256 = requireSha256("game_sha256", gameSha256);
        this.loaderSha256 = requireSha256("loader_sha256", loaderSha256);
    }

    private static String requireSha256(String field, String value) {
        if (value == null || !value.toLowerCase(Locale.ROOT).matches("[0-9a-f]{64}")) {
            throw new ProtocolException(field + " must be 64 lowercase hexadecimal characters");
        }
        return value.toLowerCase(Locale.ROOT);
    }

    public String gameSha256() { return gameSha256; }
    public String loaderSha256() { return loaderSha256; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof ReferenceBuild r)) return false;
        return gameSha256.equals(r.gameSha256) && loaderSha256.equals(r.loaderSha256);
    }

    @Override
    public int hashCode() {
        return 31 * gameSha256.hashCode() + loaderSha256.hashCode();
    }

    @Override
    public String toString() {
        return ReferenceBuild.class.getSimpleName() + "{gameSha256='" + gameSha256 + "'}";
    }
}
