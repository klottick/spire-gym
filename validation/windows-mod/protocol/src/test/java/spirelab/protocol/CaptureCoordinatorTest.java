package spirelab.protocol;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Passive-capture coordinator tests with a fake view.
 *
 * <p>Key brief assertion: calling {@code afterGameUpdate} twice at one
 * unchanged boundary must produce exactly one initial record and zero RNG
 * sampling calls — the coordinator is observe-only.
 */
class CaptureCoordinatorTest {

    /** Counts any live-RNG sampling so tests can assert passive capture. */
    static final class FakeBoundary implements BoundaryEvent {
        private final String identity;
        private final Json.Node state;
        private final Optional<SemanticAction> action;
        private final boolean captureEntry;
        int rngCalls;

        FakeBoundary(String identity, Json.Node state, Optional<SemanticAction> action, boolean captureEntry) {
            this.identity = identity;
            this.state = state;
            this.action = action;
            this.captureEntry = captureEntry;
        }

        @Override public String identity() { return identity; }
        @Override public Json.Node state() { return state; }
        @Override public Optional<SemanticAction> action() { return action; }

        @Override
        public int rngOperationCount() {
            rngCalls++;
            return 0;
        }

        @Override public boolean captureEntry() { return captureEntry; }
    }

    static final class FakeView implements GameView {
        private final BoundaryEvent event;
        FakeView(BoundaryEvent event) { this.event = event; }
        @Override public BoundaryEvent boundary() { return event; }
    }

    private static Json.Node state() {
        return Json.parse(
            "{\"player\":{\"hp\":70,\"max_hp\":70,\"block\":0,\"energy\":3,\"energy_per_turn\":3,"
                + "\"base_draw\":5,\"relics\":[\"champion-1\"],\"statuses\":[]},\"enemies\":[],"
                + "\"cards\":[],\"hand\":[],\"draw\":[],\"discard\":[],\"exhaust\":[],\"in_play\":[],"
                + "\"rng\":{\"stream_id\":\"main\",\"position\":0},\"queue\":[],\"pending\":[],"
                + "\"phase\":\"player_turn\",\"turn\":1,\"outcome\":null,"
                + "\"public_knowledge\":{\"hand\":[],\"enemy_intents\":[],\"deck_order\":null,"
                + "\"relics\":[\"champion-1\"],\"history\":[]},\"ruleset\":{\"character\":\"ironclad\","
                + "\"ascension\":0,\"mode\":\"standard\"}}");
    }

    private static TraceRecord.Header header() {
        return new TraceRecord.Header(
            new GameIdentity("slay-the-spire", "ironclad", 0),
            List.of("0000000000000001"),
            new Provenance("spire-lab-protocol-test", "1.0.0", null),
            new ReferenceBuild("0".repeat(64), "f".repeat(64)));
    }

    @Test
    void twoCallsAtUnchangedBoundary_writeOneInitial_and_zeroRngCalls(@TempDir Path dir) throws IOException {
        Path file = dir.resolve("dedup.jsonl");
        TraceWriter writer = new TraceWriter(file);
        CaptureCoordinator coordinator = new CaptureCoordinator(writer, header());
        FakeBoundary boundary = new FakeBoundary("boundary-0", state(), Optional.empty(), true);
        FakeView view = new FakeView(boundary);

        coordinator.afterGameUpdate(view);
        coordinator.afterGameUpdate(view); // same unchanged boundary: no-op

        writer.close();
        List<String> lines = Files.readAllLines(file, StandardCharsets.UTF_8);
        assertEquals(2, lines.size(), "exactly one header + one initial record: " + lines);
        assertEquals("header", ((TraceRecord.Header) TraceRecord.parse(lines.get(0))).kind());
        assertEquals("initial", ((TraceRecord.Initial) TraceRecord.parse(lines.get(1))).kind());
        assertEquals(0, boundary.rngCalls, "passive capture must never sample the live RNG");
        assertEquals(0, coordinator.stepsWritten());
        assertTrue(!coordinator.hasEnded());
    }

    @Test
    void initial_thenTwoSteps_thenEnd_roundTrips(@TempDir Path dir) throws IOException {
        Path file = dir.resolve("full.jsonl");
        TraceWriter writer = new TraceWriter(file);
        CaptureCoordinator coordinator = new CaptureCoordinator(writer, header());

        coordinator.afterGameUpdate(new FakeView(
            new FakeBoundary("boundary-0", state(), Optional.empty(), true)));
        coordinator.afterGameUpdate(new FakeView(
            new FakeBoundary("boundary-1", state(),
                Optional.of(new SemanticAction.PlayCard("strike-1", "cultist-1")), true)));
        coordinator.afterGameUpdate(new FakeView(
            new FakeBoundary("boundary-2", state(),
                Optional.of(new SemanticAction.EndTurn()), true)));
        coordinator.endCombat(Outcome.VICTORY);
        writer.close();

        List<String> lines = Files.readAllLines(file, StandardCharsets.UTF_8);
        assertEquals(5, lines.size());
        assertEquals("header", ((TraceRecord) TraceRecord.parse(lines.get(0))).kind());
        assertEquals("initial", ((TraceRecord) TraceRecord.parse(lines.get(1))).kind());
        TraceRecord.Step step0 = (TraceRecord.Step) TraceRecord.parse(lines.get(2));
        assertEquals(0, step0.index());
        assertEquals(new SemanticAction.PlayCard("strike-1", "cultist-1"), step0.action());
        TraceRecord.Step step1 = (TraceRecord.Step) TraceRecord.parse(lines.get(3));
        assertEquals(1, step1.index());
        assertEquals(Outcome.VICTORY, ((TraceRecord.End) TraceRecord.parse(lines.get(4))).outcome());
        assertEquals(2, coordinator.stepsWritten());
        assertTrue(coordinator.hasEnded());
    }

    @Test
    void changedBoundary_withoutAction_isRejected(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("t.jsonl"));
        CaptureCoordinator coordinator = new CaptureCoordinator(writer, header());
        coordinator.afterGameUpdate(new FakeView(
            new FakeBoundary("boundary-0", state(), Optional.empty(), true)));
        ProtocolException e = assertThrows(ProtocolException.class, () ->
            coordinator.afterGameUpdate(new FakeView(
                new FakeBoundary("boundary-1", state(), Optional.empty(), true))));
        assertTrue(e.getMessage().contains("action"), "got: " + e.getMessage());
    }

    @Test
    void nonCaptureEntry_boundary_is_rejected(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("t.jsonl"));
        CaptureCoordinator coordinator = new CaptureCoordinator(writer, header());
        ProtocolException e = assertThrows(ProtocolException.class, () ->
            coordinator.afterGameUpdate(new FakeView(
                new FakeBoundary("boundary-0", state(), Optional.empty(), false))));
        assertTrue(e.getMessage().contains("capture entry"), "got: " + e.getMessage());
    }

    @Test
    void update_afterEnd_isRejected(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("t.jsonl"));
        CaptureCoordinator coordinator = new CaptureCoordinator(writer, header());
        coordinator.afterGameUpdate(new FakeView(
            new FakeBoundary("boundary-0", state(), Optional.empty(), true)));
        coordinator.endCombat(Outcome.DEFEAT);
        assertThrows(ProtocolException.class, () ->
            coordinator.afterGameUpdate(new FakeView(
                new FakeBoundary("boundary-1", state(),
                    Optional.of(new SemanticAction.EndTurn()), true))));
    }
}
