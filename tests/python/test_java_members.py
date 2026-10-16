"""Self-authored fixtures: no game source or inspection artifacts."""

import pytest

from validation.evidence_report import ReportError
from validation.java_members import parse_javap_members, select_javap_group, validate_cfr_group


FQCN = "fixture.Box"


def javap(body):
    return 'Compiled from "Box.java"\npublic class fixture.Box {\n' + body + '}\n'


def test_reference_descriptor_is_not_a_second_declaration():
    text = javap("  public java.lang.String label();\n    descriptor: ()Ljava/lang/String;\n")
    members = parse_javap_members(text, FQCN)
    assert len(members) == 1
    member = members[0]
    assert member.canonical == (FQCN, "label", "()Ljava/lang/String;")
    assert (member.signature_line, member.start_line, member.end_line, member.descriptor_line) == (3, 3, 4, 4)
    assert member.signature == "public java.lang.String label();"
    assert member.block_text == "  public java.lang.String label();\n    descriptor: ()Ljava/lang/String;\n"
    selected = select_javap_group(text, FQCN, "label", ["()Ljava/lang/String;"])
    assert selected.members == members
    assert selected.text == "public class fixture.Box {\n" + member.block_text + "}\n"
    assert len(selected.source_sha256) == 64


def test_selection_cannot_leak_following_static_initializer_or_field():
    text = javap("  public void run();\n    descriptor: ()V\n    Code:\n       0: return\n\n"
                 "  static {};\n    descriptor: ()V\n    Code:\n       0: invokestatic #7 // unrelated\n"
                 "  private int field;\n    descriptor: I\n")
    members = parse_javap_members(text, FQCN)
    assert [m.name for m in members] == ["run", "<clinit>"]
    selected = select_javap_group(text, FQCN, "run", ["()V"])
    assert "return" in selected.text
    assert "static {}" not in selected.text
    assert "unrelated" not in selected.text
    assert "field" not in selected.text
    with pytest.raises(ReportError):
        select_javap_group(text, FQCN, "run", ["()V", "(I)V"])


def test_all_visibility_generics_arrays_throws_and_abstract_native():
    text = javap("  private int field;\n    descriptor: I\n"
                 "  public fixture.Box();\n    descriptor: ()V\n"
                 "  fixture.Box(int);\n    descriptor: (I)V\n"
                 "  protected abstract <T extends java.lang.Object> T[] get(java.util.List<T>) throws java.io.IOException;\n"
                 "    descriptor: (Ljava/util/List;)[Ljava/lang/Object;\n"
                 "  native int[][] get(int[], java.lang.String...);\n    descriptor: ([I[Ljava/lang/String;)[[I\n")
    members = parse_javap_members(text, FQCN)
    assert [m.canonical for m in members] == [
        (FQCN, "<init>", "()V"), (FQCN, "<init>", "(I)V"),
        (FQCN, "get", "(Ljava/util/List;)[Ljava/lang/Object;"),
        (FQCN, "get", "([I[Ljava/lang/String;)[[I"),
    ]
    selected = select_javap_group(text, FQCN, "get", ["([I[Ljava/lang/String;)[[I", "(Ljava/util/List;)[Ljava/lang/Object;"])
    assert [m.name for m in selected.members] == ["get", "get"]
    assert "fixture.Box();" not in selected.text
    with pytest.raises(ReportError):
        select_javap_group(text, FQCN, "get", ["([I[Ljava/lang/String;)[[I"])


@pytest.mark.parametrize("body", [
    "  public void run();\n",
    "  public void run();\n    descriptor: ()V\n    descriptor: ()V\n",
    "  public void run();\n    descriptor: I\n",
    "  public void run();\n    descriptor: (V)V\n",
    "  public void run();\n    descriptor: ()[V\n",
    "  public void run();\n    descriptor: ()Ljava/lang/String\n",
    "  public void run();\n    descriptor: ()V junk\n",
    "  public void run();\n    descriptor: ()V\n  private void run();\n    descriptor: ()V\n",
    "  public fixture.Box();\n    descriptor: ()I\n",
    "  static {};\n    descriptor: (I)V\n",
])
def test_malformed_missing_duplicate_descriptors_fail_closed(body):
    with pytest.raises(ReportError):
        parse_javap_members(javap(body), FQCN)


@pytest.mark.parametrize("text", ["public void run();\n descriptor: ()V\n", "public class other.Box {\n}\n", javap("") + javap("")])
def test_javap_requires_one_matching_class_wrapper(text):
    with pytest.raises(ReportError):
        parse_javap_members(text, FQCN)


@pytest.mark.parametrize("expected", [[], ["()V", "()V"], ["bad"]])
def test_approved_descriptor_set_must_be_nonempty_valid_unique(expected):
    with pytest.raises(ReportError):
        select_javap_group(javap("  public void run();\n    descriptor: ()V\n"), FQCN, "run", expected)


@pytest.mark.parametrize("extra", [
    "  public void hidden()\n    descriptor: ()V\n",
    "  static {}\n    descriptor: ()V\n",
    "  private int field\n    descriptor: I\n",
    "  static {}\n    Code:\n       0: return\n",
    "  public void hidden()\n    Code:\n       0: return\n",
    "  private int field\n",
    "}\n",
])
def test_javap_unknown_top_level_text_cannot_leak_into_selection(extra):
    with pytest.raises(ReportError):
        select_javap_group(javap("  public void run();\n    descriptor: ()V\n" + extra), FQCN, "run", ["()V"])


@pytest.mark.parametrize("descriptor", ["()Lfoo//Bar;", "()Lfoo/;", "()L/foo;", "(Lfoo//Bar;)V"])
def test_empty_internal_class_name_segments_are_malformed(descriptor):
    with pytest.raises(ReportError):
        parse_javap_members(javap("  public java.lang.Object run();\n    descriptor: " + descriptor + "\n"), FQCN)


@pytest.mark.parametrize("statement", ["return run();", "throw run();", "yield run();", "else run();"])
def test_cfr_statements_are_not_return_types(statement):
    with pytest.raises(ReportError):
        validate_cfr_group(statement, FQCN, "run", 1)


def test_cfr_comments_literals_char_escapes_and_calls_do_not_add_members():
    source = '''/* public void extra() {} */
public void run() {
    char quote = '\\'';
    String s = "\\\" } public void extra() {";
    Runnable r = new Runnable() { public void extra() {} };
    this.extra();
}
'''
    assert len(validate_cfr_group(source, FQCN, "run", 1).members) == 1


def test_javap_bytecode_descriptor_comments_are_not_declarations():
    source = javap("  public void run();\n    descriptor: ()V\n    Code:\n"
                   "       0: invokevirtual #9 // Method label:()Ljava/lang/String;\n"
                   "       3: return\n")
    assert len(parse_javap_members(source, FQCN)) == 1


def test_javap_crlf_and_header_trailing_whitespace_keep_original_anchors():
    source = "public class fixture.Box {  \r\n  public void run();\r\n    descriptor: ()V\r\n}\r\n"
    selection = select_javap_group(source, FQCN, "run", ["()V"])
    assert selection.class_header_line == 1
    assert selection.members[0].block_text == "  public void run();\r\n    descriptor: ()V\r\n"
    assert (selection.members[0].start_line, selection.members[0].end_line) == (2, 3)


@pytest.mark.parametrize("signature", [
    "public java.util.List<String run()",
    "public java.util.List<String>> run()",
    "public int[ run()",
    "public int] run()",
    "public void run(java.util.List<String value)",
    "public void run(int[]] value)",
])
def test_unbalanced_signature_delimiters_fail_closed_in_both_dialects(signature):
    with pytest.raises(ReportError):
        validate_cfr_group(signature + " {}", FQCN, "run", 1)
    with pytest.raises(ReportError):
        parse_javap_members(javap("  " + signature + ";\n    descriptor: ()V\n"), FQCN)


def test_cfr_literals_cannot_supply_or_hide_declaration_tokens():
    with pytest.raises(ReportError):
        validate_cfr_group('"noise" public void run() {}', FQCN, "run", 1)


def test_cfr_complex_generic_return_and_parameters():
    source = '''public <T extends Number & Comparable<T>> java.util.Map<String, java.util.List<T[]>> run(
        final java.util.Map<? super String, ? extends T> input, int... counts
    ) throws java.io.IOException { return null; }
'''
    assert len(validate_cfr_group(source, FQCN, "run", 1).members) == 1


@pytest.mark.parametrize("prefix", ["@Override\n", "@Override "])
@pytest.mark.parametrize("wrapped", [False, True])
def test_cfr_override_marker_is_accepted_with_exact_original_anchors(prefix, wrapped):
    method = prefix + 'public void update() { String s = "@Unknown fake() {}"; }'
    source = "public class Box {\n" + method + "\n}" if wrapped else method
    group = validate_cfr_group(source, FQCN, "update", 1)
    member = group.members[0]
    assert member.block_text == method
    assert member.signature == prefix + "public void update()"
    assert member.start_line == (2 if wrapped else 1)
    assert member.signature_line == member.start_line
    assert member.end_line == member.start_line + prefix.count("\n")


@pytest.mark.parametrize("source", [
    "@Override() public void update() {}",
    '@Override(value="ignored") public void update() {}',
    "@java.lang.Override public void update() {}",
    "@OverrideExtra public void update() {}",
    "@Override @Unknown public void update() {}",
    "@Override @Override public void update() {}",
    '"@Override" public void update() {}',
    "@Override public Box() {}",
    "@Override private int field;\npublic void update() {}",
])
def test_cfr_override_support_does_not_accept_other_annotation_forms(source):
    with pytest.raises(ReportError):
        validate_cfr_group(source, FQCN, "update", 1)


@pytest.mark.parametrize("opcode", ["tableswitch", "lookupswitch"])
@pytest.mark.parametrize("selected_name", ["run", "dispatch"])
def test_javap_switch_in_selected_or_unrelated_method_is_bounded(opcode, selected_name):
    switch_block = ("  public int dispatch(int);\n    descriptor: (I)I\n    Code:\n"
                    "       0: iload_1\n       1: " + opcode + " { // synthetic cases\n"
                    "                     0: 20\n                     1: 22\n"
                    "               default: 24\n          }\n"
                    "      20: iconst_0\n      21: ireturn\n"
                    "      22: iconst_1\n      23: ireturn\n"
                    "      24: iconst_m1\n      25: ireturn\n")
    text = javap("  public void run();\n    descriptor: ()V\n    Code:\n       0: return\n"
                 + switch_block + "  static {};\n    descriptor: ()V\n    Code:\n       0: return\n")
    assert [m.name for m in parse_javap_members(text, FQCN)] == ["run", "dispatch", "<clinit>"]
    expected = "()V" if selected_name == "run" else "(I)I"
    selection = select_javap_group(text, FQCN, selected_name, [expected])
    assert len(selection.members) == 1
    assert (opcode in selection.text) == (selected_name == "dispatch")
    assert "static {};" not in selection.text
    if selected_name == "dispatch":
        assert selection.members[0].block_text == switch_block
        assert selection.members[0].start_line == 7
        assert selection.members[0].end_line == 21
        assert selection.text.endswith("      25: ireturn\n}\n")
    else:
        assert "dispatch" not in selection.text


@pytest.mark.parametrize("code", [
    "    Code:\n          }\n",
    "    Code:\n       0: tableswitch { // 0 to 0\n               default: 20\n",
    "    Code:\n       0: lookupswitch { // 0\n               default: 20\n}\n",
    "    Code:\n       0: lookupswitch { // 0\n               default: 20\n          }\n          }\n",
    "    Code:\n       0: tableswitch { // 0 to 0\n                     0: 20\n          }\n",
    "    Code:\n       0: lookupswitch { // 0\n          public void leaked();\n          }\n",
    "       0: lookupswitch { // 0\n               default: 20\n          }\n",
])
def test_javap_malformed_switch_or_orphan_closings_fail_closed(code):
    text = javap("  public void run();\n    descriptor: ()V\n" + code
                 + "  public void unrelated();\n    descriptor: ()V\n")
    with pytest.raises(ReportError):
        select_javap_group(text, FQCN, "run", ["()V"])


@pytest.mark.parametrize("extra", [
    "    static {}\n    Code:\n       0: invokestatic #7 // unrelated\n",
    "    {\n",
    "    {}\n",
    "    static {\n",
    "    public void hidden() { return; }\n",
])
def test_deeper_unsupported_javap_braces_cannot_leak_into_selection(extra):
    text = ("public class fixture.Box {\n  public void run();\n    descriptor: ()V\n"
            "    Code:\n       0: return\n" + extra + "}\n")
    with pytest.raises(ReportError):
        select_javap_group(text, FQCN, "run", ["()V"])


def test_javap_braces_in_instruction_comments_are_not_structural():
    text = javap("  public void run();\n    descriptor: ()V\n    Code:\n"
                 "       0: ldc #7 // String static {} public void fake() {\n"
                 "       2: pop\n       3: return\n")
    selection = select_javap_group(text, FQCN, "run", ["()V"])
    assert selection.members[0].canonical == (FQCN, "run", "()V")
    assert "// String static {}" in selection.text


@pytest.mark.parametrize("constructor_first", [True, False])
def test_javap_class_named_methods_coexist_with_constructor(constructor_first):
    constructor = "  public fixture.Box();\n    descriptor: ()V\n"
    methods = ("  public int Box();\n    descriptor: ()I\n"
               "  public java.lang.String Box(int);\n    descriptor: (I)Ljava/lang/String;\n")
    text = javap(constructor + methods if constructor_first else methods + constructor)
    members = parse_javap_members(text, FQCN)
    expected = [(FQCN, "Box", "()I"), (FQCN, "Box", "(I)Ljava/lang/String;")]
    assert [m.canonical for m in members] == ([(FQCN, "<init>", "()V")] + expected
                                             if constructor_first else expected + [(FQCN, "<init>", "()V")])
    method_group = select_javap_group(text, FQCN, "Box", ["()I", "(I)Ljava/lang/String;"])
    assert [m.canonical for m in method_group.members] == expected
    assert "public fixture.Box();" not in method_group.text
    init_group = select_javap_group(text, FQCN, "<init>", ["()V"])
    assert [m.canonical for m in init_group.members] == [(FQCN, "<init>", "()V")]
    assert "public int Box();" not in init_group.text


def test_cfr_class_named_methods_are_not_constructor_groups():
    constructor = "public Box() {}"
    methods = 'public int Box() { return 1; }\npublic String Box(int n) { return "s"; }'
    group = validate_cfr_group(methods, FQCN, "Box", 2)
    assert [m.name for m in group.members] == ["Box", "Box"]
    assert [m.signature for m in group.members] == ["public int Box()", "public String Box(int n)"]
    assert validate_cfr_group(constructor, FQCN, "<init>", 1).members[0].name == "<init>"
    # A wrapper containing both is a mixed-name inventory, not a selected group.
    # CFR does not select declarations or prove descriptor equivalence.
    combined = "public class Box {\n" + constructor + "\n" + methods + "\n}"
    for requested in ("Box", "<init>"):
        with pytest.raises(ReportError, match="extra CFR member name"):
            validate_cfr_group(combined, FQCN, requested, 3)
    with pytest.raises(ReportError):
        validate_cfr_group(methods, FQCN, "<init>", 2)


def test_no_return_type_constructor_still_requires_void_descriptor():
    with pytest.raises(ReportError):
        parse_javap_members(javap("  public fixture.Box();\n    descriptor: ()I\n"), FQCN)


def test_column_zero_cfr_constructor_fragments_are_counted():
    source = 'public Box() { this(1); }\nBox(int n) { String s = "fake() { }"; }\n'
    group = validate_cfr_group(source, FQCN, "<init>", 2)
    assert group.fqcn == FQCN and group.name == "<init>"
    assert [m.name for m in group.members] == ["<init>", "<init>"]
    assert [(m.start_line, m.end_line) for m in group.members] == [(1, 1), (2, 2)]
    with pytest.raises(ReportError):
        validate_cfr_group(source, FQCN, "<init>", 1)
    with pytest.raises(ReportError):
        validate_cfr_group(source, FQCN, "Box", 2)


def test_cfr_wrapped_generic_multiline_throws_and_nested_body():
    source = '''package fixture;
public class Box<T> {
    // public void unwanted() {}
    private String field = factory("run() {}");
    public <U extends Number> U[] run(
        U[] values
    ) throws IllegalStateException {
        if (values != null) { factory("}"); }
        return values;
    }
    protected native int run(int value);
}
'''
    group = validate_cfr_group(source, FQCN, "run", 2)
    assert [m.name for m in group.members] == ["run", "run"]
    assert [(m.start_line, m.end_line) for m in group.members] == [(5, 10), (11, 11)]


@pytest.mark.parametrize("source,name,count", [
    ("public Box() {}\npublic void extra() {}", "<init>", 1),
    ("public void run() {}\npublic void extra() {}", "run", 1),
    ("public void run() {}\npublic void run(int n) {}", "run", 1),
    ("String value = run();", "run", 1),
    ("// public void run() {}\nString s = \"public void run() {}\";", "run", 1),
    ("public void run() {", "run", 1),
    ("public void run() {} /* unfinished", "run", 1),
    ("public class Other { public void run() {} }", "run", 1),
    ("public class Box { class Inner { public void run() {} } }", "run", 1),
    ("static { run(); }\npublic void run() {}", "run", 1),
    ("@Unknown public void run() {}", "run", 1),
    ("public void run() {}", "run", 0),
])
def test_cfr_rejects_wrong_groups_false_positives_and_ambiguity(source, name, count):
    with pytest.raises(ReportError):
        validate_cfr_group(source, FQCN, name, count)
