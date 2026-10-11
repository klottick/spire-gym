package spirelab.protocol;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;

/**
 * The four trace-v1 records. Serialization order and field sets are strict
 * (deny-unknown semantics, matching the Rust reader): any field outside the
 * wire schema is a protocol error, never silently dropped.
 */
public sealed interface TraceRecord
        permits TraceRecord.Header, TraceRecord.Initial, TraceRecord.Step, TraceRecord.End {

    String RECORD_TAG = "record";

    String kind();

    /**
     * Serialize this record. The header is the only record without a
     * fingerprint (pass {@code null}); every other record requires the
     * SHA-256 fingerprint of the preceding line, precomputed by the writer.
     */
    String toJsonLine(String fingerprint);

    /** Parse one wire record line (any of the four kinds). */
    static TraceRecord parse(String line) {
        Json.Node node = Json.parse(line);
        if (!node.isObject()) throw new ProtocolException("record line must be a JSON object");
        Json.Node tag = node.member(RECORD_TAG);
        if (tag == null || !tag.isString()) {
            throw new ProtocolException("record line is missing its '" + RECORD_TAG + "' tag");
        }
        switch (tag.string()) {
            case "header": return Header.parse(node);
            case "initial": return Initial.parse(node);
            case "step": return Step.parse(node);
            case "end": return End.parse(node);
            default: throw new ProtocolException("unknown record kind '" + tag.string() + "'");
        }
    }

    // ---- shared strict helpers ----

    static void requireKeys(Json.Node node, Set<String> expected, String what) {
        Map<String, Json.Node> actual = node.members();
        if (!actual.keySet().equals(expected)) {
            throw new ProtocolException(what + " field set " + actual.keySet()
                + " does not match the trace-v1 schema " + expected);
        }
    }

    static void requireFingerprint(String fp) {
        if (fp == null || !fp.matches("[0-9a-f]{64}")) {
            throw new ProtocolException("fingerprint must be 64 lowercase hexadecimal characters");
        }
    }

    // ---- records ----

    final class Header implements TraceRecord {
        public static final String VERSION = "1";

        private final GameIdentity game;
        private final List<String> seedWords;
        private final Provenance provenance;
        private final ReferenceBuild referenceBuild;

        public Header(GameIdentity game, List<String> seedWords, Provenance provenance,
                      ReferenceBuild referenceBuild) {
            if (seedWords == null || seedWords.isEmpty()) {
                throw new ProtocolException("rng.seed_words must be non-empty");
            }
            for (String w : seedWords) {
                if (!w.matches("[0-9a-f]{16}")) {
                    throw new ProtocolException("seed word '" + w + "' is not 16 lowercase hex digits");
                }
            }
            this.game = Objects.requireNonNull(game, "game");
            this.seedWords = List.copyOf(seedWords);
            this.provenance = Objects.requireNonNull(provenance, "provenance");
            this.referenceBuild = Objects.requireNonNull(referenceBuild, "referenceBuild");
        }

        @Override public String kind() { return "header"; }
        public GameIdentity game() { return game; }
        public List<String> seedWords() { return seedWords; }
        public Provenance provenance() { return provenance; }
        public ReferenceBuild referenceBuild() { return referenceBuild; }

        @Override
        public String toJsonLine(String fingerprint) {
            if (fingerprint != null) throw new ProtocolException("header records carry no fingerprint");
            Map<String, Json.Node> rng = new LinkedHashMap<>();
            rng.put("seed_words", Json.array(seedWords.stream().map(Json::string).collect(Collectors.toList())));
            Json.Node capt = provenance.capturedAtUtc() == null
                ? Json.nullValue()
                : Json.string(provenance.capturedAtUtc());
            Map<String, Json.Node> prov = new LinkedHashMap<>();
            prov.put("tool", Json.string(provenance.tool()));
            prov.put("tool_version", Json.string(provenance.toolVersion()));
            prov.put("captured_at_utc", capt);
            Map<String, Json.Node> game = new LinkedHashMap<>();
            game.put("id", Json.string(this.game.id()));
            game.put("character", Json.string(this.game.character()));
            game.put("ascension", Json.number(this.game.ascension()));
            Map<String, Json.Node> rb = new LinkedHashMap<>();
            rb.put("game_sha256", Json.string(referenceBuild.gameSha256()));
            rb.put("loader_sha256", Json.string(referenceBuild.loaderSha256()));
            Map<String, Json.Node> root = new LinkedHashMap<>();
            root.put(RECORD_TAG, Json.string("header"));
            root.put("version", Json.string(VERSION));
            root.put("game", Json.object(game));
            root.put("rng", Json.object(rng));
            root.put("provenance", Json.object(prov));
            root.put("reference_build", Json.object(rb));
            return Json.serialize(Json.object(root));
        }

        static Header parse(Json.Node node) {
            requireKeys(node, Set.of(RECORD_TAG, "version", "game", "rng", "provenance", "reference_build"), "header");
            Json.Node version = node.member("version");
            if (version == null || !version.isString() || !VERSION.equals(version.string())) {
                throw new ProtocolException("unsupported trace version " + version);
            }
            Json.Node gameNode = node.member("game");
            requireKeys(gameNode, Set.of("id", "character", "ascension"), "game");
            Json.Node gameId = gameNode.member("id");
            Json.Node character = gameNode.member("character");
            Json.Node ascension = gameNode.member("ascension");
            if (gameId == null || !gameId.isString() || gameId.string().isBlank()
                || character == null || !character.isString() || character.string().isBlank()
                || ascension == null || !ascension.isNumber()) {
                throw new ProtocolException("game identity is malformed");
            }
            int asc;
            try {
                asc = Integer.parseInt(ascension.number());
            } catch (NumberFormatException e) {
                throw new ProtocolException("ascension " + ascension + " is not an integer");
            }
            if (asc < 0) throw new ProtocolException("ascension must be >= 0");
            GameIdentity game = new GameIdentity(gameId.string(), character.string(), asc);

            Json.Node rngNode = node.member("rng");
            requireKeys(rngNode, Set.of("seed_words"), "rng");
            Json.Node words = rngNode.member("seed_words");
            if (words == null || !words.isArray() || words.elements().isEmpty()) {
                throw new ProtocolException("rng.seed_words must be a non-empty array");
            }
            List<String> seedWords = words.elements().stream()
                .map(w -> {
                    if (w == null || !w.isString() || !w.string().matches("[0-9a-f]{16}")) {
                        throw new ProtocolException("rng seed word must be a 16-lowercase-hex string");
                    }
                    return w.string();
                })
                .collect(Collectors.toList());

            Json.Node provNode = node.member("provenance");
            requireKeys(provNode, Set.of("tool", "tool_version", "captured_at_utc"), "provenance");
            Json.Node tool = provNode.member("tool");
            Json.Node toolVersion = provNode.member("tool_version");
            Json.Node captured = provNode.member("captured_at_utc");
            if (tool == null || !tool.isString() || tool.string().isBlank()
                || toolVersion == null || !toolVersion.isString() || toolVersion.string().isBlank()) {
                throw new ProtocolException("provenance tool/tool_version must be non-blank");
            }
            Provenance provenance = new Provenance(
                tool.string(),
                toolVersion.string(),
                captured == null || captured.isNull() ? null : requireString(captured, "captured_at_utc"));

            Json.Node rbNode = node.member("reference_build");
            requireKeys(rbNode, Set.of("game_sha256", "loader_sha256"), "reference_build");
            Json.Node gSha = rbNode.member("game_sha256");
            Json.Node lSha = rbNode.member("loader_sha256");
            if (gSha == null || !gSha.isString() || lSha == null || !lSha.isString()) {
                throw new ProtocolException("reference_build hashes must be strings");
            }
            ReferenceBuild rb = new ReferenceBuild(gSha.string(), lSha.string());
            return new Header(game, seedWords, provenance, rb);
        }
    }

    final class Initial implements TraceRecord {
        private final Json.Node state;
        private String fingerprint; // set on parse; the writer binds its own at write time

        public Initial(Json.Node state) {
            requireState(state, "initial");
            this.state = state;
        }

        @Override public String kind() { return "initial"; }
        public Json.Node state() { return state; }
        /** Fingerprint as written on the wire (64 lowercase hex), or null before serializing. */
        public String fingerprint() { return fingerprint; }

        @Override
        public String toJsonLine(String fingerprint) {
            requireFingerprint(fingerprint);
            Map<String, Json.Node> root = new LinkedHashMap<>();
            root.put(RECORD_TAG, Json.string("initial"));
            root.put("fingerprint", Json.string(fingerprint));
            root.put("state", state);
            return Json.serialize(Json.object(root));
        }

        static Initial parse(Json.Node node) {
            requireKeys(node, Set.of(RECORD_TAG, "fingerprint", "state"), "initial");
            Json.Node fpNode = node.member("fingerprint");
            String fp = requireString(fpNode, "fingerprint");
            requireFingerprint(fp);
            Initial initial = new Initial(node.member("state"));
            initial.fingerprint = fp;
            return initial;
        }
    }

    final class Step implements TraceRecord {
        private final long index;
        private final SemanticAction action;
        private final Json.Node expected;
        private String fingerprint;

        public Step(long index, SemanticAction action, Json.Node expected) {
            if (index < 0) throw new ProtocolException("step index must be >= 0");
            if (action == null) throw new ProtocolException("step action must not be null");
            requireState(expected, "step expected state");
            this.index = index;
            this.action = action;
            this.expected = expected;
        }

        @Override public String kind() { return "step"; }
        public long index() { return index; }
        public SemanticAction action() { return action; }
        public Json.Node expected() { return expected; }
        public String fingerprint() { return fingerprint; }

        @Override
        public String toJsonLine(String fingerprint) {
            requireFingerprint(fingerprint);
            Map<String, Json.Node> root = new LinkedHashMap<>();
            root.put(RECORD_TAG, Json.string("step"));
            root.put("fingerprint", Json.string(fingerprint));
            root.put("index", Json.number(index));
            root.put("action", SemanticAction.toJson(action));
            root.put("expected", expected);
            return Json.serialize(Json.object(root));
        }

        static Step parse(Json.Node node) {
            requireKeys(node, Set.of(RECORD_TAG, "fingerprint", "index", "action", "expected"), "step");
            Json.Node fpNode = node.member("fingerprint");
            String fp = requireString(fpNode, "fingerprint");
            requireFingerprint(fp);
            Json.Node idx = node.member("index");
            if (idx == null || !idx.isNumber()) throw new ProtocolException("step index must be an integer");
            long index = Long.parseLong(idx.number());
            SemanticAction action = SemanticAction.parse(node.member("action"));
            Step step = new Step(index, action, node.member("expected"));
            step.fingerprint = fp;
            return step;
        }
    }

    final class End implements TraceRecord {
        private final Outcome outcome;
        private String fingerprint;

        public End(Outcome outcome) {
            this.outcome = Objects.requireNonNull(outcome, "outcome");
        }

        @Override public String kind() { return "end"; }
        public Outcome outcome() { return outcome; }
        public String fingerprint() { return fingerprint; }

        @Override
        public String toJsonLine(String fingerprint) {
            requireFingerprint(fingerprint);
            Map<String, Json.Node> root = new LinkedHashMap<>();
            root.put(RECORD_TAG, Json.string("end"));
            root.put("fingerprint", Json.string(fingerprint));
            root.put("outcome", Json.string(outcome.wire()));
            return Json.serialize(Json.object(root));
        }

        static End parse(Json.Node node) {
            requireKeys(node, Set.of(RECORD_TAG, "fingerprint", "outcome"), "end");
            Json.Node fpNode = node.member("fingerprint");
            String fp = requireString(fpNode, "fingerprint");
            requireFingerprint(fp);
            Json.Node outcomeNode = node.member("outcome");
            String wire = requireString(outcomeNode, "outcome");
            End end = new End(Outcome.fromWire(wire));
            end.fingerprint = fp;
            return end;
        }
    }

    // ---- vocabulary shared with the Rust reader (Task 4 field agreement) ----

    Set<String> SUPPORTED_STATE_FIELDS = Set.of(
        "player", "enemies", "cards", "hand", "draw", "discard", "exhaust", "in_play",
        "rng", "queue", "pending", "phase", "turn", "outcome", "public_knowledge", "ruleset");

    static void requireState(Json.Node state, String what) {
        if (state == null || !state.isObject()) {
            throw new ProtocolException(what + " must be a JSON object");
        }
        for (String key : state.members().keySet()) {
            if (!stateKeyAllowed(key)) {
                throw new ProtocolException(what + " field '" + key
                    + "' is not part of the trace-v1 supported vocabulary");
            }
        }
        Json.Node player = state.member("player");
        if (player != null) {
            if (!player.isObject()) throw new ProtocolException("state.player must be a JSON object");
            for (String key : player.members().keySet()) {
                if (!SUPPORTED_PLAYER_FIELDS.contains(key)) {
                    throw new ProtocolException("state.player field '" + key
                        + "' is not part of the trace-v1 supported vocabulary");
                }
            }
        }
    }

    static boolean stateKeyAllowed(String key) {
        return SUPPORTED_STATE_FIELDS.contains(key);
    }

    Set<String> SUPPORTED_PLAYER_FIELDS = Set.of(
        "hp", "max_hp", "block", "energy", "energy_per_turn", "base_draw", "relics", "statuses");

    private static String requireString(Json.Node node, String what) {
        if (node == null || !node.isString()) {
            throw new ProtocolException(what + " must be a string");
        }
        return node.string();
    }
}
