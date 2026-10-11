package spirelab.protocol;

/** Pinned game identity for a trace (trace-local, never machine-specific). */
public final class GameIdentity {
    private final String id;
    private final String character;
    private final int ascension;

    public GameIdentity(String id, String character, int ascension) {
        if (id == null || id.isBlank()) throw new ProtocolException("game id must be non-blank");
        if (character == null || character.isBlank()) throw new ProtocolException("character must be non-blank");
        if (ascension < 0) throw new ProtocolException("ascension must be >= 0");
        this.id = id;
        this.character = character;
        this.ascension = ascension;
    }

    public String id() { return id; }
    public String character() { return character; }
    public int ascension() { return ascension; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof GameIdentity g)) return false;
        return ascension == g.ascension && id.equals(g.id) && character.equals(g.character);
    }

    @Override
    public int hashCode() {
        return 31 * (31 * id.hashCode() + character.hashCode()) + ascension;
    }

    @Override
    public String toString() {
        return GameIdentity.class.getSimpleName() + "{id='" + id + "', character='" + character
            + "', ascension=" + ascension + "}";
    }
}
