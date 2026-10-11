package spirelab.protocol;

import java.nio.file.Path;
import java.util.List;

/**
 * Test-scope helper: writes a small trace-v1 sample with the real writer
 * (header, initial, two steps, end) so the Rust reader can be pointed at
 * it (SPIRE_JAVA_TRACE interop test). Protocol-test only — not a reference
 * capture.
 */
public final class InteropSampleWriter {
    private InteropSampleWriter() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            System.err.println("usage: InteropSampleWriter <output.jsonl>");
            System.exit(2);
        }
        Path out = Path.of(args[0]);
        String stateJson =
            "{\"ruleset\":{\"character\":\"ironclad\",\"ascension\":0,\"mode\":\"standard\"},"
                + "\"player\":{\"hp\":70,\"max_hp\":70,\"block\":0,\"energy\":3,\"energy_per_turn\":3,"
                + "\"base_draw\":5,\"relics\":[\"champion-1\"],\"statuses\":[]},"
                + "\"enemies\":[{\"id\":\"cultist-1\",\"hp\":54,\"max_hp\":54,\"block\":0}],"
                + "\"cards\":[],\"hand\":[\"strike-1\"],\"draw\":[\"bash-1\"],\"discard\":[],"
                + "\"exhaust\":[],\"in_play\":[],"
                + "\"rng\":{\"stream_id\":\"main\",\"position\":0},\"queue\":[],\"pending\":[],"
                + "\"phase\":\"player_turn\",\"turn\":1,\"outcome\":null,"
                + "\"public_knowledge\":{\"hand\":[\"strike-1\"],\"enemy_intents\":[],"
                + "\"deck_order\":null,\"relics\":[\"champion-1\"],\"history\":[]}}";
        Json.Node state = Json.parse(stateJson);
        Json.Node expected0 = Json.parse(
            stateJson.replace("\"hp\":54", "\"hp\":48").replace("\"energy\":3", "\"energy\":2"));
        Json.Node expected1 = Json.parse(
            stateJson.replace("\"turn\":1", "\"turn\":2"));

        TraceWriter writer = new TraceWriter(out);
        writer.append(new TraceRecord.Header(
            new GameIdentity("slay-the-spire", "ironclad", 0),
            List.of("0000000000000001"),
            new Provenance("spire-lab-interop-sample", "1.0.0", null),
            new ReferenceBuild("0".repeat(64), "f".repeat(64))));
        writer.append(new TraceRecord.Initial(state));
        writer.append(new TraceRecord.Step(0, new SemanticAction.PlayCard("strike-1", "cultist-1"), expected0));
        writer.append(new TraceRecord.Step(1, new SemanticAction.EndTurn(), expected1));
        writer.append(new TraceRecord.End(Outcome.VICTORY));
        writer.close();
        System.out.println("wrote " + out);
    }
}
