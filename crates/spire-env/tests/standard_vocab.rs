use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn enum_as_str_names(source: &str, enum_name: &str) -> BTreeSet<String> {
    let marker = format!("impl {enum_name}");
    let impl_start = source.find(&marker).unwrap_or_else(|| panic!("missing {marker}"));
    let impl_source = &source[impl_start + marker.len()..];
    let fn_start = impl_source
        .find("pub const fn as_str")
        .expect("enum as_str function");
    let function_source = &impl_source[fn_start..];
    let open = function_source.find('{').expect("as_str body opening brace");
    let mut depth = 0usize;
    let mut close = None;
    for (offset, byte) in function_source.as_bytes().iter().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &function_source[open..=close.expect("as_str body closing brace")];
    let bytes = body.as_bytes();
    let mut names = BTreeSet::new();
    let mut cursor = 0usize;
    while cursor + 6 < bytes.len() {
        let Some(relative) = body[cursor..].find("Self::") else {
            break;
        };
        let start = cursor + relative + "Self::".len();
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        let variant = &body[start..end];
        let arm_end = body[end..].find("=>").map(|n| end + n + 2);
        cursor = end;
        if variant == "Unknown" {
            continue;
        }
        let Some(rhs_start) = arm_end else { continue };
        let rhs = body[rhs_start..].trim_start();
        if let Some(rhs) = rhs.strip_prefix('"') {
            if let Some(name_end) = rhs.find('"') {
                names.insert(rhs[..name_end].to_owned());
            }
        }
    }
    names
}

fn manifest_family_names(manifest: &Value, family: &str) -> Vec<String> {
    manifest["families"][family]
        .as_array()
        .unwrap_or_else(|| panic!("missing vocabulary family {family}"))
        .iter()
        .map(|value| value.as_str().expect("vocabulary name").to_owned())
        .collect()
}

fn without_padding(names: &[String]) -> BTreeSet<String> {
    names.iter().skip(1).cloned().collect()
}

#[test]
fn standard_vocab_manifest_matches_registries() {
    let manifest = spire_env::standard_vocab_manifest();
    let families = manifest["families"].as_object().expect("families object");
    for (family, value) in families {
        let rows = value.as_array().expect("family array");
        let names = rows
            .iter()
            .map(|row| row.as_str().expect("family name"))
            .collect::<Vec<_>>();
        assert_eq!(names.len(), names.iter().copied().collect::<BTreeSet<_>>().len(), "duplicates in {family}");
        let is_open = manifest["open_families"]
            .as_array()
            .expect("open_families array")
            .iter()
            .any(|open| open == family);
        if is_open {
            assert_eq!(names.first(), Some(&"padding"), "open family {family} must start with padding");
        }
    }

    let source_families = [
        ("card", "CardId", include_str!("../../spire-core/src/content/ids/cards.rs")),
        ("relic", "RelicId", include_str!("../../spire-core/src/content/ids/items.rs")),
        ("potion", "PotionId", include_str!("../../spire-core/src/content/ids/items.rs")),
        ("event", "EventId", include_str!("../../spire-core/src/content/ids/events.rs")),
        ("enemy", "EnemyId", include_str!("../../spire-core/src/content/ids/encounters.rs")),
        ("encounter", "EncounterId", include_str!("../../spire-core/src/content/ids/encounters.rs")),
    ];
    for (family, enum_name, source) in source_families {
        let family_rows = manifest_family_names(manifest, family);
        assert_eq!(
            enum_as_str_names(source, enum_name),
            without_padding(&family_rows),
            "{family} does not match {enum_name}::as_str",
        );
    }
    let all_cards = spire_core::content::CardId::ALL
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(all_cards, without_padding(&manifest_family_names(manifest, "card")));

    for family in ["room", "choice", "selection_operation", "intent_category"] {
        let expected = manifest_family_names(manifest, family);
        let actual = spire_env::standard_registry_names(family).expect("supported Standard registry family");
        assert_eq!(actual, expected.into_iter().skip(1).collect::<Vec<_>>(), "registry ordering for {family}");
    }
}

#[test]
fn standard_vocab_manifest_is_append_only() {
    // Pins were computed once from the current M34b manifest families and the current Standard
    // descriptor's `intent`/`power` lists, hashing serde_json::to_string(list) as UTF-8.
    const PINS: &[(&str, usize, &str)] = &[
        ("card", 370, "4582bf0131d7f2e50d24c838a7f33f98b44b382f7f7e16ff49595731050eb04c"),
        ("card_cost_kind", 4, "6bd98aaa6a1ba72c828c3fb2f70d9cd6fab2fd6970aca79a06159d42b5131637"),
        ("card_type", 6, "d5e13f33ec28478cee2758c61d1db69fa00187562ecc6d84378ee805f558c606"),
        ("character", 5, "881d844e5b6e7f71baa609bcd0a2844d0fec3f7028921248343380b9696c2eeb"),
        ("choice", 41, "a7e633bf6641ce29c6cec53676f2e5cd8fbb940cb638d906e06262e33e1ee0a0"),
        ("choice_status", 5, "6ab454609cdb8ac089d6fdd62593313dbefbb39ba119cff2f93f644c6de10966"),
        ("combat_phase", 8, "38d8e3f9ed2e0fda4f57790c03c819b647aadfb316a30087bd8c346fbb10b2bd"),
        ("encounter", 64, "9b300189c421280bee0737cf45442e660f4263a9e178945f2105eeb85384bc0d"),
        ("encounter_side", 3, "408c1373de6d7e8f197e09813ba23770dab7ddd6dfb1da65b1c875cb9de665c6"),
        ("enemy", 66, "f81b96ff4d7b71760e741ea784d2aad87be7bb40869a6dc0e424513899cc86f5"),
        ("event", 53, "59204e5c6affc891458df714d7db8412e1a37db95b5569de985ff3c901c33eb0"),
        ("facing", 3, "408c1373de6d7e8f197e09813ba23770dab7ddd6dfb1da65b1c875cb9de665c6"),
        ("intent_category", 18, "dca6435a541c0501301f23070529e70ce713063cad618cd5c622454e567c4b9c"),
        ("map_kind", 9, "1c96ad1169a958394ff73c50bcacbff50cb8c9b816eed1083d7761904cae4fb9"),
        ("neow_drawback", 5, "d3e71c1ad8bf63357c4f5a1b0babca0c5875847056916e82af24e2308da5eb88"),
        ("neow_reward", 20, "49276621309c0df415d5d5f5152239f0bac199a535cc23819103305d97369b73"),
        ("orb", 5, "7b20fffcc6c3ba4c23d322f3b3b0432be6608f3140cff0664a4c6d341ca6cb56"),
        ("outcome", 4, "eff6b3734c0cc0f7ddf29616b4a3f29e5bcc4e065cf977c078312a5a345ae21a"),
        ("potion", 43, "9c567393e20149e784700e8e65b7615cece4c5f28b968b0f1d79a2d694cc7a3e"),
        ("profile", 2, "7bc8306d92a9e79640da8d866fcc85d671e2290bdde2ad99cfe052f79f0479bb"),
        ("relic", 181, "34e01547a8df07236819e0daa315c503967941f43d01d85c4cdee8c849d12dc8"),
        ("room", 24, "9dab519d37ecbeec9aef784c8e24760b2b6e2be532b95c6fa98b8c57fed6a579"),
        ("row_group", 12, "216e07cb33551ac4b119f4b4c4b70069522b210054dbf591e1600e63c4329027"),
        ("selection_operation", 39, "459529bfffe2705752a09ac17cc58d21dc61ea08b59dd5b07bc0c8c8b030f167"),
        ("stance", 4, "3eb38a92106457e5a723991454fa3c0ed9eb2c5c3abe3474fd18444a03b8a247"),
        ("terminal", 5, "eb4e3256c10d7ac284adb243b2038bcd37546f39c13b34cc62ee4c3c8e37dd0a"),
        ("verb", 16, "2400384e24a00c3d6f3f173ade8e7e0a6e743729c37917187a1c0bd21614830b"),
        ("zone", 10, "4da47f4794a91aba98a35a0ffd03a805055d275124dc00f5d148c59ffe6bc565"),
        ("intent", 192, "d8e261369f6851551798b9cd965d3963e6765077f5c9b9f885978fcc25a924f2"),
        ("power", 138, "926584da726a988288ad8d2225bb2d9670bd58d63ee529c3e06e834a4f768c7f"),
    ];
    let manifest = spire_env::standard_vocab_manifest();
    let spec = spire_env::standard_run_observation_spec(&spire_env::RunCapacities::standard_default())
        .expect("Standard default descriptor");
    let descriptor = spec.descriptor();
    let manifest_families = manifest["families"].as_object().expect("manifest families");
    assert_eq!(PINS.len(), manifest_families.len() + 2, "pins cover all manifest families plus intent/power");

    for (family, pinned_len, pinned_sha256) in PINS {
        let names = if *family == "intent" || *family == "power" {
            descriptor["vocabularies"][family].as_array().expect("descriptor vocabulary")
        } else {
            manifest["families"][family].as_array().expect("manifest vocabulary")
        };
        assert!(names.len() >= *pinned_len, "{family} shrank below its v1 length");
        let serialized = serde_json::to_string(&names[..*pinned_len]).expect("serialize pinned prefix");
        let digest = Sha256::digest(serialized.as_bytes());
        let actual = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(&actual, pinned_sha256, "{family} changed its pinned v1 prefix");
    }
}

#[test]
fn standard_reuses_shared_power_and_intent_order() {
    let spec = spire_env::standard_run_observation_spec(&spire_env::RunCapacities::standard_default())
        .expect("Standard default descriptor");
    let vocabularies = spec.descriptor()["vocabularies"].clone();
    let intent = std::iter::once("padding")
        .chain(spire_core::combat::public_ids::PUBLIC_INTENTS.iter().copied())
        .chain(spire_core::combat::public_ids::EVENT_INTENTS.iter().copied())
        .collect::<Vec<_>>();
    let power = std::iter::once("padding")
        .chain(spire_core::combat::public_ids::PUBLIC_POWERS.iter().copied())
        .collect::<Vec<_>>();
    assert_eq!(vocabularies["intent"], json!(intent));
    assert_eq!(vocabularies["power"], json!(power));
}

#[test]
fn standard_descriptor_is_deterministic() {
    let capacities = spire_env::RunCapacities::standard_default();
    let first = spire_env::standard_run_observation_spec(&capacities).expect("first spec");
    let second = spire_env::standard_run_observation_spec(&capacities).expect("second spec");
    assert_eq!(first.descriptor(), second.descriptor());
    assert_eq!(first.fingerprint(), second.fingerprint());

    let mut changed_capacities = capacities;
    changed_capacities.map_nodes += 1;
    let changed = spire_env::standard_run_observation_spec(&changed_capacities).expect("changed spec");
    assert_ne!(first.fingerprint(), changed.fingerprint());
}

#[test]
fn standard_descriptor_carries_params_semantics() {
    let spec = spire_env::standard_run_observation_spec(&spire_env::RunCapacities::standard_default())
        .expect("Standard default descriptor");
    let descriptor = spec.descriptor();
    let semantics = &descriptor["semantics"];
    assert_eq!(semantics["vocabulary_manifest"], "standard-vocab-v1");
    assert_eq!(semantics["capacity_preset"], "standard-default-v1");
    let projected = spire_core::run::PARAM_SEMANTICS
        .iter()
        .map(|row| json!({
            "event": row.event.as_str(),
            "stage": row.stage,
            "option": row.option,
            "index": row.index,
            "meaning": row.meaning,
        }))
        .collect::<Vec<_>>();
    assert_eq!(projected.len(), 92);
    assert_eq!(semantics["param_semantics"], json!(projected));

    let event_constants: Value = serde_json::from_str(include_str!("../src/standard_event_constants_v1.json"))
        .expect("event constants JSON");
    assert_eq!(semantics["event_constants"], event_constants);
    let rows = event_constants["rows"].as_array().expect("event constants rows");
    let represented = rows
        .iter()
        .filter_map(|row| row["event"].as_str())
        .collect::<BTreeSet<_>>();
    for event in ["joust", "tomb_of_lord_red_mask", "falling", "upgrade_shrine", "purifier", "ancient_writing"] {
        assert!(represented.contains(event), "missing accepted-handler event {event}");
    }
    let has_numeric_constant = |event: &str, meaning: &str, value: i64| {
        rows.iter().any(|row| {
            row["event"] == event && row["meaning"] == meaning && row["value"] == value
        })
    };
    assert!(has_numeric_constant("joust", "bet_gold", 50));
    assert!(has_numeric_constant("joust", "murderer_win_gold", 100));
    assert!(has_numeric_constant("joust", "owner_win_gold", 250));
    assert!(has_numeric_constant("tomb_of_lord_red_mask", "gold_gained", 222));
}
