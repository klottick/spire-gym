package spirelab.protocol;

import java.util.AbstractMap;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

/**
 * Minimal strict JSON parser/serializer used by the trace-v1 wire protocol.
 *
 * <p>Strictness (fail-closed, no silent coercion):
 * <ul>
 *   <li>no NaN/Infinity literals, no hex numbers, no leading zeros;</li>
 *   <li>integers are bounded to the signed/unsigned 64-bit range (the
 *       trace-v1 protocol domain, matching the Rust reader);</li>
 *   <li>duplicate object keys are rejected;</li>
 *   <li>objects keep insertion order; serialization is deterministic and
 *       whitespace-free, and numbers stay verbatim (no numeric loss);</li>
 *   <li>string values escape as required; control characters are rejected.</li>
 * </ul>
 */
public final class Json {
    private Json() {}

    private static final int MAX_DEPTH = 512;
    private static final String U64_MAX = "18446744073709551615";

    // --- factory methods (validated, fail-closed) ---

    public static Node string(String value) {
        if (value == null) throw new ProtocolException("string value must not be null");
        return new Node(Node.Kind.STRING, null, null, value, null, null);
    }

    public static Node number(long value) {
        return number(Long.toString(value));
    }

    public static Node number(String digits) {
        if (digits == null || digits.isEmpty()) throw new ProtocolException("numeric digits must not be null/empty");
        String bare = digits.startsWith("-") ? digits.substring(1) : digits;
        if (!bare.matches("[0-9]+")) throw new ProtocolException("number '" + digits + "' is not an integer");
        if (bare.length() > 20 || (bare.length() == 20 && bare.compareTo(U64_MAX) > 0)) {
            throw new ProtocolException("integer " + digits + " exceeds the protocol 64-bit range");
        }
        if (bare.length() > 1 && bare.startsWith("0")) throw new ProtocolException("number '" + digits + "' has a leading zero");
        return new Node(Node.Kind.NUMBER, null, null, null, digits, null);
    }

    public static Node booleanValue(boolean b) {
        return new Node(Node.Kind.BOOLEAN, null, null, null, null, b);
    }

    public static Node nullValue() {
        return new Node(Node.Kind.NULL, null, null, null, null, null);
    }

    public static Node object() {
        return new Node(Node.Kind.OBJECT, new LinkedHashMap<>(), null, null, null, null);
    }

    public static Node object(Map<String, Node> members) {
        if (members == null) throw new ProtocolException("object members must not be null");
        for (Map.Entry<String, Node> e : members.entrySet()) {
            if (e.getKey() == null || e.getValue() == null)
                throw new ProtocolException("object member key/value must not be null");
        }
        return new Node(Node.Kind.OBJECT, new LinkedHashMap<>(members), null, null, null, null);
    }

    public static Node array() {
        return new Node(Node.Kind.ARRAY, null, new ArrayList<>(), null, null, null);
    }

    public static Node array(List<Node> elements) {
        if (elements == null) throw new ProtocolException("array elements must not be null");
        for (Node e : elements) if (e == null) throw new ProtocolException("array element must not be null");
        return new Node(Node.Kind.ARRAY, null, new ArrayList<>(elements), null, null, null);
    }

    public static Node parse(String json) {
        Parser p = new Parser(json);
        Node node = p.parseValue(0);
        p.skipWs();
        if (!p.eof()) throw new ProtocolException("trailing characters after JSON value");
        return node;
    }

    public static String serialize(Node node) {
        StringBuilder sb = new StringBuilder();
        write(node, sb, 0);
        return sb.toString();
    }

    private static void write(Node node, StringBuilder sb, int depth) {
        if (depth > MAX_DEPTH) throw new ProtocolException("value nesting exceeds protocol depth");
        switch (node.kind) {
            case OBJECT -> {
                sb.append('{');
                boolean first = true;
                for (Map.Entry<String, Node> e : node.members.entrySet()) {
                    if (!first) sb.append(',');
                    first = false;
                    writeString(e.getKey(), sb);
                    sb.append(':');
                    write(e.getValue(), sb, depth + 1);
                }
                sb.append('}');
            }
            case ARRAY -> {
                sb.append('[');
                boolean first = true;
                for (Node e : node.elements) {
                    if (!first) sb.append(',');
                    first = false;
                    write(e, sb, depth + 1);
                }
                sb.append(']');
            }
            case STRING -> writeString(node.string, sb);
            case NUMBER -> sb.append(node.number);
            case BOOLEAN -> sb.append(node.bool ? "true" : "false");
            case NULL -> sb.append("null");
        }
    }

    private static void writeString(String s, StringBuilder sb) {
        sb.append('"');
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '"' -> sb.append("\\\"");
                case '\\' -> sb.append("\\\\");
                case '\b' -> sb.append("\\b");
                case '\f' -> sb.append("\\f");
                case '\n' -> sb.append("\\n");
                case '\r' -> sb.append("\\r");
                case '\t' -> sb.append("\\t");
                default -> {
                    if (c < 0x20) {
                        sb.append(String.format("\\u%04x", (int) c));
                    } else {
                        sb.append(c);
                    }
                }
            }
        }
        sb.append('"');
    }

    /** One JSON value with its exact numeric text (numbers are never widened or rounded). */
    public static final class Node {
        public enum Kind { OBJECT, ARRAY, STRING, NUMBER, BOOLEAN, NULL }

        private final Kind kind;
        private final LinkedHashMap<String, Node> members;
        private final ArrayList<Node> elements;
        private final String string;
        private final String number;
        private final Boolean bool;

        private Node(Kind kind,
                     LinkedHashMap<String, Node> members,
                     ArrayList<Node> elements,
                     String string,
                     String number,
                     Boolean bool) {
            this.kind = kind;
            this.members = members;
            this.elements = elements;
            this.string = string;
            this.number = number;
            this.bool = bool;
        }

        public Kind kind() { return kind; }

        public boolean isObject() { return kind == Kind.OBJECT; }
        public boolean isArray() { return kind == Kind.ARRAY; }
        public boolean isString() { return kind == Kind.STRING; }
        public boolean isNumber() { return kind == Kind.NUMBER; }
        public boolean isBoolean() { return kind == Kind.BOOLEAN; }
        public boolean isNull() { return kind == Kind.NULL; }

        public Map<String, Node> members() {
            require(Kind.OBJECT);
            return Collections.unmodifiableMap(members);
        }

        public Node put(String key, Node value) {
            require(Kind.OBJECT);
            if (key == null || value == null) throw new ProtocolException("member key/value must not be null");
            if (members.containsKey(key)) throw new ProtocolException("duplicate object key '" + key + "'");
            members.put(key, value);
            return value;
        }

        public List<Node> elements() {
            require(Kind.ARRAY);
            return Collections.unmodifiableList(elements);
        }

        public String string() {
            if (kind == Kind.STRING) return string;
            return null;
        }

        public Node member(String key) {
            require(Kind.OBJECT);
            return members.get(key);
        }

        /** Verbatim numeric digits as written (lossless). */
        public String number() {
            return kind == Kind.NUMBER ? number : null;
        }

        public boolean booleanValue() {
            if (kind != Kind.BOOLEAN) throw new ProtocolException("node is not a boolean");
            return bool;
        }

        private void require(Kind expected) {
            if (kind != expected)
                throw new ProtocolException("node kind is " + kind + ", expected " + expected);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (!(o instanceof Node n)) return false;
            return kind == n.kind
                && Objects.equals(members, n.members)
                && Objects.equals(elements, n.elements)
                && Objects.equals(string, n.string)
                && Objects.equals(number, n.number)
                && Objects.equals(bool, n.bool);
        }

        @Override
        public int hashCode() {
            int h = 17;
            h = 31 * h + kind.hashCode();
            h = 31 * h + Objects.hashCode(members);
            h = 31 * h + Objects.hashCode(elements);
            h = 31 * h + Objects.hashCode(string);
            h = 31 * h + Objects.hashCode(number);
            h = 31 * h + Objects.hashCode(bool);
            return h;
        }

        @Override
        public String toString() { return Json.serialize(this); }
    }

    private static final class Parser {
        final String s;
        int pos;

        Parser(String s) { this.s = Objects.requireNonNull(s, "json"); }

        boolean eof() { return pos >= s.length(); }
        char ch() { return s.charAt(pos); }

        void skipWs() {
            while (!eof() && (ch() == ' ' || ch() == '\t' || ch() == '\n' || ch() == '\r')) pos++;
        }

        String peek() {
            if (eof()) throw new ProtocolException("unexpected end of JSON input");
            return String.valueOf(ch());
        }

        Node parseValue(int depth) {
            skipWs();
            if (depth > MAX_DEPTH) throw new ProtocolException("value nesting exceeds protocol depth");
            String t = peek();
            return switch (t) {
                case "{" -> parseObject(depth);
                case "[" -> parseArray(depth);
                case "\"" -> new Node(Node.Kind.STRING, null, null, parseString(), null, null);
                case "t" -> expectLiteral("true", Node.Kind.BOOLEAN, Boolean.TRUE);
                case "f" -> expectLiteral("false", Node.Kind.BOOLEAN, Boolean.FALSE);
                case "n" -> expectLiteral("null", Node.Kind.NULL, null);
                default -> parseNumber();
            };
        }

        private Node expectLiteral(String lit, Node.Kind kind, Boolean b) {
            if (!s.startsWith(lit, pos)) throw new ProtocolException("garbage literal at " + pos + ": " + peek());
            pos += lit.length();
            return new Node(kind, null, null, null, null, b);
        }

        private Node parseObject(int depth) {
            pos++; // '{'
            LinkedHashMap<String, Node> members = new LinkedHashMap<>();
            skipWs();
            if (!eof() && ch() == '}') { pos++; return new Node(Node.Kind.OBJECT, members, null, null, null, null); }
            while (true) {
                skipWs();
                if (!eof() && ch() == '"') {
                    String key = parseString();
                    skipWs();
                    if (eof() || ch() != ':') throw new ProtocolException("expected ':' after object key at " + pos);
                    pos++;
                    Node v = parseValue(depth + 1);
                    if (members.put(key, v) != null) throw new ProtocolException("duplicate object key '" + key + "' at " + pos);
                } else {
                    throw new ProtocolException("expected object key string at " + pos + ": " + peek());
                }
                skipWs();
                if (eof()) throw new ProtocolException("unterminated object at " + pos);
                char c = ch();
                if (c == ',') { pos++; continue; }
                if (c == '}') { pos++; break; }
                throw new ProtocolException("expected ',' or '}' in object at " + pos + ": " + c);
            }
            return new Node(Node.Kind.OBJECT, members, null, null, null, null);
        }

        private Node parseArray(int depth) {
            pos++; // '['
            ArrayList<Node> elements = new ArrayList<>();
            skipWs();
            if (!eof() && ch() == ']') { pos++; return new Node(Node.Kind.ARRAY, null, elements, null, null, null); }
            while (true) {
                elements.add(parseValue(depth + 1));
                skipWs();
                if (eof()) throw new ProtocolException("unterminated array at " + pos);
                char c = ch();
                if (c == ',') { pos++; continue; }
                if (c == ']') { pos++; break; }
                throw new ProtocolException("expected ',' or ']' in array at " + pos + ": " + c);
            }
            return new Node(Node.Kind.ARRAY, null, elements, null, null, null);
        }

        private String parseString() {
            pos++; // '"'
            StringBuilder sb = new StringBuilder();
            while (true) {
                if (eof()) throw new ProtocolException("unterminated string at " + pos);
                char c = ch();
                if (c == '"') { pos++; return sb.toString(); }
                if (c == '\\') {
                    pos++;
                    if (eof()) throw new ProtocolException("bad escape at " + pos);
                    char e = ch();
                    pos++;
                    switch (e) {
                        case '"' -> sb.append('"');
                        case '\\' -> sb.append('\\');
                        case '/' -> sb.append('/');
                        case 'b' -> sb.append('\b');
                        case 'f' -> sb.append('\f');
                        case 'n' -> sb.append('\n');
                        case 'r' -> sb.append('\r');
                        case 't' -> sb.append('\t');
                        case 'u' -> {
                            if (pos + 4 > s.length()) throw new ProtocolException("bad unicode escape at " + pos);
                            String hx = s.substring(pos, pos + 4);
                            if (!hx.matches("[0-9a-fA-F]{4}")) throw new ProtocolException("bad unicode escape at " + pos);
                            sb.append((char) Integer.parseInt(hx, 16));
                            pos += 4;
                        }
                        default -> throw new ProtocolException("bad escape '\\" + e + "' at " + (pos - 1));
                    }
                } else if (c < 0x20) {
                    throw new ProtocolException("raw control character in string at " + pos);
                } else {
                    sb.append(c);
                    pos++;
                }
            }
        }

        private Node parseNumber() {
            int start = pos;
            if (!eof() && ch() == '-') pos++;
            if (eof()) throw new ProtocolException("bad number at " + start);
            char c = ch();
            if (c == '0') {
                pos++;
            } else if (c >= '1' && c <= '9') {
                while (!eof() && ch() >= '0' && ch() <= '9') pos++;
            } else if (c >= 'a' && c <= 'z') {
                // NaN / Infinity / any alphabetic junk is not a JSON number
                int j = start;
                String word = "";
                while (j < s.length() && (Character.isLetterOrDigit(s.charAt(j)))) { j++; word += s.charAt(j); }
                throw new ProtocolException(word.matches("(?i)(nan|infinity)")
                    ? "non-finite number '" + word + "' is not allowed at " + start
                    : "not a valid number at " + start);
            } else {
                throw new ProtocolException("not a valid number at " + start);
            }
            int intEnd = pos;
            String intPart = s.substring(start, intEnd);
            if (!eof() && ch() == '.') {
                pos++;
                if (eof() || ch() < '0' || ch() > '9') throw new ProtocolException("bad fractional number at " + intEnd);
                while (!eof() && ch() >= '0' && ch() <= '9') pos++;
            }
            if (!eof() && (ch() == 'e' || ch() == 'E')) {
                pos++;
                if (!eof() && (ch() == '+' || ch() == '-')) pos++;
                if (eof() || ch() < '0' || ch() > '9') throw new ProtocolException("bad exponent at " + pos);
                while (!eof() && ch() >= '0' && ch() <= '9') pos++;
            }
            if (s.substring(start, intEnd).contains(".") || s.substring(start, pos).matches(".*[eE].*")) {
                // finite floats are protocol-legal only where declared; keep verbatim
                return new Node(Node.Kind.NUMBER, null, null, null, s.substring(start, pos), null);
            }
            String digits = intPart.startsWith("-") ? intPart.substring(1) : intPart;
            if (digits.length() > 20
                || (digits.length() == 20 && digits.compareTo(U64_MAX) > 0)) {
                throw new ProtocolException("integer " + intPart + " exceeds the protocol 64-bit range at " + start);
            }
            return new Node(Node.Kind.NUMBER, null, null, null, intPart, null);
        }
    }
}
