package spirelab.protocol;

/**
 * Where a trace came from. Captured-at is ISO-8601 UTC or null when the
 * sample is synthetic (protocol-test fixtures use null).
 */
public final class Provenance {
    private final String tool;
    private final String toolVersion;
    private final String capturedAtUtc; // nullable by protocol design

    public Provenance(String tool, String toolVersion, String capturedAtUtc) {
        if (tool == null || tool.isBlank()) throw new ProtocolException("provenance tool must be non-blank");
        if (toolVersion == null || toolVersion.isBlank())
            throw new ProtocolException("provenance toolVersion must be non-blank");
        this.tool = tool;
        this.toolVersion = toolVersion;
        this.capturedAtUtc = capturedAtUtc;
    }

    public String tool() { return tool; }
    public String toolVersion() { return toolVersion; }
    public String capturedAtUtc() { return capturedAtUtc; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof Provenance p)) return false;
        return tool.equals(p.tool) && toolVersion.equals(p.toolVersion)
            && java.util.Objects.equals(capturedAtUtc, p.capturedAtUtc);
    }

    @Override
    public int hashCode() {
        return 31 * (31 * tool.hashCode() + toolVersion.hashCode())
            + java.util.Objects.hashCode(capturedAtUtc);
    }

    @Override
    public String toString() {
        return Provenance.class.getSimpleName() + "{tool='" + tool + "', toolVersion='" + toolVersion
            + "', capturedAtUtc=" + capturedAtUtc + "}";
    }
}
