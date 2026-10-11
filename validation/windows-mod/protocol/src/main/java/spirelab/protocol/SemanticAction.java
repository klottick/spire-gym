package spirelab.protocol;

/**
 * Observed semantic action at a step boundary, identified by trace-local
 * card/enemy identities (never heap addresses or private RNG state).
 */
public sealed interface SemanticAction permits SemanticAction.PlayCard, SemanticAction.EndTurn {

    String WIRE_KIND = "type";
    String PLAY_CARD = "play_card";
    String END_TURN = "end_turn";

    record PlayCard(String card, String target) implements SemanticAction {
        public PlayCard {
            if (card == null || card.isBlank())
                throw new ProtocolException("play_card card identity must be non-blank");
            if (target == null || target.isBlank())
                throw new ProtocolException("play_card target identity must be non-blank");
        }
    }

    record EndTurn() implements SemanticAction {
    }

    /** Parse the wire action object. Strict: exact field sets per kind. */
    static SemanticAction parse(Json.Node action) {
        if (action == null || !action.isObject()) {
            throw new ProtocolException("action must be a JSON object");
        }
        Json.Node kindNode = action.member(WIRE_KIND);
        if (kindNode == null || !kindNode.isString()) {
            throw new ProtocolException("action is missing its '" + WIRE_KIND + "' kind");
        }
        String kind = kindNode.string();
        if (PLAY_CARD.equals(kind)) {
            if (action.members().size() != 3) {
                throw new ProtocolException("play_card action must carry exactly type/card/target");
            }
            Json.Node card = action.member("card");
            Json.Node target = action.member("target");
            if (card == null || !card.isString() || card.string().isBlank()) {
                throw new ProtocolException("play_card action is missing a non-blank card identity");
            }
            if (target == null || !target.isString() || target.string().isBlank()) {
                throw new ProtocolException("play_card action is missing a non-blank target identity");
            }
            return new PlayCard(card.string(), target.string());
        }
        if (END_TURN.equals(kind)) {
            if (action.members().size() != 1) {
                throw new ProtocolException("end_turn action must carry only its '" + WIRE_KIND + "' kind");
            }
            return new EndTurn();
        }
        throw new ProtocolException("action kind '" + kind + "' is not supported by trace-v1");
    }

    static Json.Node toJson(SemanticAction action) {
        if (action instanceof PlayCard p) {
            java.util.Map<String, Json.Node> m = new java.util.LinkedHashMap<>();
            m.put(WIRE_KIND, Json.string(PLAY_CARD));
            m.put("card", Json.string(p.card()));
            m.put("target", Json.string(p.target()));
            return Json.object(m);
        }
        if (action instanceof EndTurn) {
            java.util.Map<String, Json.Node> m = new java.util.LinkedHashMap<>();
            m.put(WIRE_KIND, Json.string(END_TURN));
            return Json.object(m);
        }
        throw new ProtocolException("unknown semantic action: " + action);
    }
}
