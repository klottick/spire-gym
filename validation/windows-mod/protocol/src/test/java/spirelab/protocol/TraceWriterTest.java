package spirelab.protocol;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Round-trip and fail-closed tests for the trace-v1 writer.
 *
 * <p>These are protocol tests (wire format only). Nothing here is a
 * game behavior oracle.
 */
class TraceWriterTest {

    /** Canonical state with values that would break if any numeric were widened or rounded. */
    private static Json.Node state() {
        return Json.parse(
            "{\"ruleset\":{\"character\":\"ironclad\",\"ascension\":0,\"mode\":\"standard\"},"
                + "\"player\":{\"hp\":70,\"max_hp\":70,\"block\":0,\"energy\":3,\"energy_per_turn\":3,"
                + "\"base_draw\":5,\"relics\":[\"champion-1\"],\"statuses\":[]},"
                + "\"enemies\":[{\"id\":\"cultist-1\",\"hp\":54,\"max_hp\":54,\"block\":0}],"
                + "\"cards\":[],\"hand\":[\"strike-1\"],\"draw\":[\"bash-1\"],\"discard\":[],"
                + "\"exhaust\":[],\"in_play\":[],"
                + "\"rng\":{\"stream_id\":\"main\",\"position\":9223372036854775807},"
                + "\"queue\":[],\"pending\":[],\"phase\":\"player_turn\",\"turn\":1,"
                + "\"outcome\":null,"
                + "\"public_knowledge\":{\"hand\":[\"strike-1\"],\"enemy_intents\":[],"
                + "\"deck_order\":null,\"relics\":[\"champion-1\"],"
                + "\"history\":[{\"turn\":1,\"seed_sample\":\"0000000000000010\"}]}}");
    }

    private static TraceRecord.Header header() {
        return new TraceRecord.Header(
            new GameIdentity("slay-the-spire", "ironclad", 0),
            List.of("0000000000000001", "0000ffff0000dead", "0000000000000010"),
            new Provenance("spire-lab-protocol-test", "1.0.0", null),
            new ReferenceBuild("0".repeat(64), "f".repeat(64)));
    }

    @Test
    void fullTrace_roundTrips_withoutNumericLoss(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("trace.jsonl"));
        Json.Node initial = state();
        Json.Node expected = state();
        writer.append(header());
        writer.append(new TraceRecord.Initial(initial));
        writer.append(new TraceRecord.Step(
            0, new SemanticAction.PlayCard("strike-1", "cultist-1"), expected));
        writer.append(new TraceRecord.End(Outcome.VICTORY));
        writer.close();

        List<String> lines = Files.readAllLines(dir.resolve("trace.jsonl"), StandardCharsets.UTF_8);
        assertEquals(4, lines.size());

        TraceRecord.Header parsedHeader =
            (TraceRecord.Header) TraceRecord.parse(lines.get(0));
        assertEquals(header().game(), parsedHeader.game());
        assertEquals(header().seedWords(), parsedHeader.seedWords());
        // hex-shaped 64-bit words stayed hex strings, verbatim (no numeric loss)
        assertEquals(List.of("0000000000000001", "0000ffff0000dead", "0000000000000010"),
            parsedHeader.seedWords());

        TraceRecord.Initial parsedInitial = (TraceRecord.Initial) TraceRecord.parse(lines.get(1));
        assertEquals(initial, parsedInitial.state(), "initial state must round-trip without loss");
        Json.Node parsedPos = parsedInitial.state().member("rng").member("position");
        assertEquals("9223372036854775807", parsedPos.number(), "64-bit position must stay verbatim");

        TraceRecord.Step parsedStep = (TraceRecord.Step) TraceRecord.parse(lines.get(2));
        assertEquals(0, parsedStep.index());
        assertEquals(new SemanticAction.PlayCard("strike-1", "cultist-1"), parsedStep.action());
        assertEquals(expected, parsedStep.expected(), "expected state must round-trip without loss");

        TraceRecord.End parsedEnd = (TraceRecord.End) TraceRecord.parse(lines.get(3));
        assertEquals(Outcome.VICTORY, parsedEnd.outcome());

        // fingerprint chain: each later line carries sha256(previous line)
        assertEquals(sha256Hex(lines.get(0)), parsedInitial.fingerprint());
        assertEquals(sha256Hex(lines.get(1)), parsedStep.fingerprint());
        assertEquals(sha256Hex(lines.get(2)), parsedEnd.fingerprint());
    }

    @Test
    void initial_before_header_is_rejected(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("t.jsonl"));
        ProtocolException e = assertThrows(ProtocolException.class,
            () -> writer.append(new TraceRecord.Initial(state())));
        assertTrue(e.getMessage().contains("header"), "got: " + e.getMessage());
    }

    @Test
    void step_out_of_sequence_is_rejected(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("t.jsonl"));
        writer.append(header());
        writer.append(new TraceRecord.Initial(state()));
        writer.append(new TraceRecord.Step(0, new SemanticAction.EndTurn(), state()));
        assertThrows(ProtocolException.class,
            () -> writer.append(new TraceRecord.Step(2, new SemanticAction.EndTurn(), state())));
        // replaying a used index is just as invalid
        assertThrows(ProtocolException.class,
            () -> writer.append(new TraceRecord.Step(0, new SemanticAction.EndTurn(), state())));
    }

    @Test
    void append_after_end_is_rejected(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("t.jsonl"));
        writer.append(header());
        writer.append(new TraceRecord.Initial(state()));
        writer.append(new TraceRecord.End(Outcome.DEFEAT));
        assertThrows(ProtocolException.class,
            () -> writer.append(new TraceRecord.Step(0, new SemanticAction.EndTurn(), state())));
        assertThrows(ProtocolException.class,
            () -> writer.append(new TraceRecord.End(Outcome.VICTORY)));
    }

    @Test
    void header_rejects_bad_seed_word(@TempDir Path dir) throws IOException {
        TraceWriter writer = new TraceWriter(dir.resolve("t.jsonl"));
        assertThrows(ProtocolException.class, () -> writer.append(new TraceRecord.Header(
            new GameIdentity("slay-the-spire", "ironclad", 0),
            List.of("000000000000001"), // 15 hex digits
            new Provenance("spire-lab-protocol-test", "1.0.0", null),
            new ReferenceBuild("0".repeat(64), "f".repeat(64)))));
    }

    @Test
    void state_must_be_object_with_supported_fields(@TempDir Path dir) {
        assertThrows(ProtocolException.class, () -> new TraceRecord.Initial(Json.parse("[1,2]")));
        assertThrows(ProtocolException.class,
            () -> new TraceRecord.Initial(Json.parse("{\"potions\":[\"ironclad-potion-1\"]}")));
    }

    @Test
    void integer_out_of_protocol_range_is_rejected() {
        // 2^64 — outside the trace-v1 integer domain (Rust reader matches)
        assertThrows(ProtocolException.class,
            () -> Json.parse("{\"x\":18446744073709551616}"));
        // NaN/Infinity never cross the wire
        assertThrows(ProtocolException.class, () -> Json.parse("{\"x\":NaN}"));
        assertThrows(ProtocolException.class, () -> Json.parse("{\"x\":Infinity}"));
    }

    private static String sha256Hex(String line) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] d = md.digest(line.getBytes(StandardCharsets.UTF_8));
            StringBuilder sb = new StringBuilder(64);
            for (byte b : d) sb.append(String.format("%02x", b));
            return sb.toString();
        } catch (Exception e) {
            throw new IllegalStateException(e);
        }
    }
}
