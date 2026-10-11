package spirelab.protocol;

/**
 * A passive view of the live game at one update boundary. Implementations
 * must NEVER issue game actions, consume RNG, or mutate the game to answer
 * {@link #boundary()} — capture is observe-only.
 */
public interface GameView {

    /** The boundary event observed at the current game update. */
    BoundaryEvent boundary();
}
