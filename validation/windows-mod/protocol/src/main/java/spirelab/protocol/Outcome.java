package spirelab.protocol;

/** Combat outcome recorded by the end record. */
public enum Outcome {
    VICTORY("victory"),
    DEFEAT("defeat");

    private final String wire;

    Outcome(String wire) {
        this.wire = wire;
    }

    public String wire() {
        return wire;
    }

    public static Outcome fromWire(String wire) {
        for (Outcome o : values()) {
            if (o.wire.equals(wire)) return o;
        }
        throw new ProtocolException("outcome '" + wire + "' is not one of victory/defeat");
    }
}
