package spirelab.protocol;

import java.util.Optional;

/**
 * One observed game boundary.
 *
 * <p>Passive capture contract:
 * <ul>
 *   <li>{@link #identity()} is stable within one logical boundary (the
 *       coordinator deduplicates repeated observations at the same
 *       boundary);</li>
 *   <li>{@link #state()} is the canonical state at the boundary;</li>
 *   <li>{@link #action()} is the semantic action observed at the boundary
 *       (empty at the initial boundary);</li>
 *   <li>{@link #captureEntry()} is true only when the game is at a valid
 *       capture entry: no game action/card queue, no pending screen
 *       selection/animation, live monsters and player input available;</li>
 *   <li>{@link #rngOperationCount()} exists ONLY so fake views can count
 *       accidental live-RNG sampling. Passive capture — including the
 *       coordinator — must never call it.</li>
 * </ul>
 */
public interface BoundaryEvent {

    String identity();

    Json.Node state();

    Optional<SemanticAction> action();

    int rngOperationCount();

    boolean captureEntry();
}
