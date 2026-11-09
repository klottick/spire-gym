//! The pre-M34a projection of public `Debug` text (M34a plan a-Done: zero gameplay delta).
//!
//! M34a adds five public-view fields: `FullEnemyView.{intent_category, intent_hits, invincible_cap}`,
//! `PublicPlayer.bottled_cards` and `PublicDecision.room_params`. Frozen transcript receipts recorded
//! before M34a compare this projection, which removes exactly those fields (compact `{:?}` and pretty
//! `{:#?}` renderings) and nothing else, so every pre-existing byte stays checked.

const ENEMY_KEYS: [&str; 3] = ["intent_category: ", "intent_hits: ", "invincible_cap: "];
const OTHER_KEYS: [&str; 2] = ["bottled_cards: ", "room_params: "];

/// `text` without the M34a public-view fields.
pub fn pre_m34a_debug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        if ENEMY_KEYS.iter().chain(&OTHER_KEYS).any(|key| trimmed.starts_with(key)) {
            // Pretty rendering: the field is this line, plus its body up to the closing line at the same indent.
            let indent = line.len() - trimmed.len();
            let opens = trimmed.trim_end().ends_with(['(', '[', '{']);
            index += 1;
            if opens {
                while index < lines.len() {
                    let next = lines[index];
                    let closing = next.len() - next.trim_start().len() == indent && next.trim_start().starts_with([')', ']', '}']);
                    index += 1;
                    if closing { break; }
                }
            }
            continue;
        }
        out.push_str(&strip_compact(line));
        index += 1;
    }
    out
}

/// Compact rendering: `, key: value` (or `key: value, ` as a first field), value bracket-balanced.
fn strip_compact(line: &str) -> String {
    let mut out = line.to_owned();
    for key in ENEMY_KEYS.iter().chain(&OTHER_KEYS) {
        let mut from = 0;
        while let Some(found) = out[from..].find(key).map(|at| at + from) {
            let field_start = found == 0 || !out[..found].ends_with([' ', '{', '(']);
            if field_start { from = found + key.len(); continue; }
            let mut end = found + key.len();
            let mut depth = 0i32;
            let bytes = out.as_bytes();
            while end < bytes.len() {
                match bytes[end] {
                    b'(' | b'[' | b'{' => depth += 1,
                    b')' | b']' | b'}' if depth == 0 => break,
                    b')' | b']' | b'}' => depth -= 1,
                    b',' if depth == 0 => break,
                    _ => {}
                }
                end += 1;
            }
            // A last field's value ends before the closing ` }`.
            while end > found + key.len() && out.as_bytes()[end - 1] == b' ' { end -= 1; }
            if out[..found].ends_with(", ") {
                out.replace_range(found - 2..end, "");
                from = found - 2;
            } else {
                // First field: drop the following ", " instead.
                if out[end..].starts_with(", ") { end += 2; }
                out.replace_range(found..end, "");
                from = found;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::pre_m34a_debug;

    #[test]
    fn projection_removes_exactly_the_m34a_fields() {
        let compact = "D { player: PublicPlayer { bottled_cards: [(BottledFlame, 10)], relic_counters: [] }, \
            e: [FullEnemyView { intent_damage: Some(6), logical_slot: 0, intent_category: Some(Attack), intent_hits: Some(2), invincible_cap: None }], \
            terminal: None, room_params: Some(RoomParams { stage: [None, None, None, None], rows: [(ChoiceId(3), [Some(1), None, None])] }) }";
        assert_eq!(pre_m34a_debug(compact), "D { player: PublicPlayer { relic_counters: [] }, \
            e: [FullEnemyView { intent_damage: Some(6), logical_slot: 0 }], terminal: None }");
        let pretty = "E {\n    logical_slot: 0,\n    intent_category: Some(\n        Attack,\n    ),\n    intent_hits: None,\n    invincible_cap: Some(\n        300,\n    ),\n}\nP {\n    bottled_cards: [],\n    gold: 99,\n}\n";
        assert_eq!(pre_m34a_debug(pretty), "E {\n    logical_slot: 0,\n}\nP {\n    gold: 99,\n}\n");
        let untouched = "PublicDecision { act: 1, floor: 0 }";
        assert_eq!(pre_m34a_debug(untouched), untouched);
    }
}
