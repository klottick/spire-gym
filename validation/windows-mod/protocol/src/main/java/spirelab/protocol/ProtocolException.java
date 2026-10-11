package spirelab.protocol;

/** Fail-closed protocol violations (malformed wire data, misuse, out-of-range values). */
public final class ProtocolException extends RuntimeException {
    public ProtocolException(String message) {
        super(message);
    }
}
