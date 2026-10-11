package spirelab.protocol;

/**
 * Decides which boundary events become trace records.
 *
 * <p>Contract:
 * <ul>
 *   <li>first {@code afterGameUpdate} (boundary with no action) writes the
 *       header + one initial record;</li>
 *   <li>each subsequent <em>changed</em> boundary with an observed action
 *       writes exactly one step record;</li>
 *   <li>repeated observations at the <em>same unchanged</em> boundary are
 *       deduplicated: nothing is written and no RNG sampling happens;</li>
 *   <li>{@link #endCombat(Outcome)} writes the single end record.</li>
 * </ul>
 * The coordinator never calls {@link BoundaryEvent#rngOperationCount()}
 * (passive capture: it must not consume or even sample the live RNG).
 */
public final class CaptureCoordinator {

    private final TraceWriter writer;
    private final TraceRecord.Header header;

    private boolean started;
    private boolean finished;
    private String lastIdentity;
    private long stepsWritten;

    public CaptureCoordinator(TraceWriter writer, TraceRecord.Header header) {
        if (writer == null || header == null) {
            throw new ProtocolException("writer and header must not be null");
        }
        this.writer = writer;
        this.header = header;
    }

    /**
     * Observe one game update boundary. Writes at most one record:
     * the first clean boundary (no action) becomes the header + initial;
     * every later changed boundary with an observed action becomes one
     * step. Repeated observations at the same unchanged identity are
     * deduplicated silently. No RNG sampling ever happens here.
     */
    public void afterGameUpdate(GameView view) throws java.io.IOException {
        if (view == null) throw new ProtocolException("view must not be null");
        BoundaryEvent event = view.boundary();
        if (event == null) throw new ProtocolException("boundary event must not be null");
        String identity = event.identity();
        if (identity == null || identity.isBlank()) {
            throw new ProtocolException("boundary identity must be non-blank");
        }
        if (lastIdentity != null && lastIdentity.equals(identity)) {
            return; // unchanged boundary: deduplicate, write nothing, sample nothing
        }
        if (finished) {
            throw new ProtocolException("the trace is already ended; no further boundaries may be captured");
        }
        if (!event.captureEntry()) {
            throw new ProtocolException("boundary is not a capture entry "
                + "(requires an empty action/card queue, no pending screen selection/animation, "
                + "live monsters and player input available)");
        }
        lastIdentity = identity;
        if (!started) {
            started = true;
            if (event.action().isPresent()) {
                throw new ProtocolException("initial boundary must have no action");
            }
            writer.append(header);
            writer.append(new TraceRecord.Initial(event.state()));
            return;
        }
        if (event.action().isEmpty()) {
            throw new ProtocolException("boundary must carry the observed semantic action");
        }
        writer.append(new TraceRecord.Step(stepsWritten, event.action().get(), event.state()));
        stepsWritten++;
    }

    /** Write the single terminating end record. */
    public void endCombat(Outcome outcome) throws java.io.IOException {
        if (outcome == null) throw new ProtocolException("outcome must not be null");
        if (!started) {
            throw new ProtocolException("cannot end before the initial boundary was captured");
        }
        if (finished) {
            throw new ProtocolException("the trace is already ended");
        }
        writer.append(new TraceRecord.End(outcome));
        finished = true;
    }

    public long stepsWritten() {
        return stepsWritten;
    }

    public boolean hasEnded() {
        return finished;
    }
}
