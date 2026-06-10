//! Sample package definitions for development, CI, and seed kits.
//!
//! All packages are signed with the test fixture keypair.
//! **Not for production distribution.**

use serde_json::json;

use crate::fixtures::build_test_package;
use crate::nmp::error::ContentResult;

// ── Shield: Water Purification ────────────────────────────────────────────────

/// Build `shield-water-purification` v1.0.0 — 5-step intro lesson.
pub fn build_shield_water_package() -> ContentResult<Vec<u8>> {
    let manifest_meta = json!({
        "module_id": "shield-water-purification",
        "axis": "Shield",
        "category_id": "shield-hygiene",
        "title_key": "shield.water.title",
        "description_key": "shield.water.desc",
        "estimated_minutes": 10,
        "locales": ["en", "ar", "sw"]
    });

    let lesson_index = json!({
        "lesson_id": "shield-water-lesson-01",
        "module_id": "shield-water-purification",
        "title_key": "shield.water.lesson1.title",
        "sort_order": 0,
        "step_count": 5
    });

    let step_0 = json!({
        "step_id": "water-s0", "sort_order": 0,
        "content_type": "text",
        "text_key": "shield.water.s0.text",
        "strings": {
            "en": "Unsafe water causes many illnesses. Learning to make water safe protects your family.",
            "ar": "\u{0627}\u{0644}\u{0645}\u{064a}\u{0627}\u{0647} \u{063a}\u{064a}\u{0631} \u{0627}\u{0644}\u{0622}\u{0645}\u{0646}\u{0629} \u{062a}\u{0633}\u{0628}\u{0628} \u{0623}\u{0645}\u{0631}\u{0627}\u{0636}.",
            "sw": "Maji yasiyo salama husababisha magonjwa mengi."
        }
    });

    let step_1 = json!({
        "step_id": "water-s1", "sort_order": 1,
        "content_type": "svg_placeholder",
        "aria_label_key": "shield.water.s1.aria",
        "svg_description": "Illustration: river water with bacteria symbols, open container, insects.",
        "strings": {
            "en": "Contamination sources: rivers, open containers, insects.",
            "sw": "Vyanzo vya uchafuzi: mito, vyombo wazi, wadudu."
        }
    });

    let step_2 = json!({
        "step_id": "water-s2", "sort_order": 2,
        "content_type": "acknowledge",
        "text_key": "shield.water.s2.confirm",
        "strings": {
            "en": "I understand the main sources of water contamination.",
            "sw": "Naelewa vyanzo vikuu vya uchafuzi wa maji."
        }
    });

    let step_3 = json!({
        "step_id": "water-s3", "sort_order": 3,
        "content_type": "multiple_choice",
        "question_key": "shield.water.s3.question",
        "strings": { "en": "Which method makes water safe to drink?" },
        "options": [
            { "id": "A", "key": "shield.water.s3.opt_a",
              "strings": { "en": "Boil for 1 minute" } },
            { "id": "B", "key": "shield.water.s3.opt_b",
              "strings": { "en": "Add sand" } },
            { "id": "C", "key": "shield.water.s3.opt_c",
              "strings": { "en": "Leave in sunlight for 30 seconds" } }
        ],
        "correct_id": "A"
    });

    let step_4 = json!({
        "step_id": "water-s4", "sort_order": 4,
        "content_type": "text",
        "text_key": "shield.water.s4.summary",
        "strings": {
            "en": "Boiling water for 1 minute kills harmful bacteria and viruses.",
            "ar": "\u{063a}\u{0644}\u{064a} \u{0627}\u{0644}\u{0645}\u{0627}\u{0621} \u{062f}\u{0642}\u{064a}\u{0642}\u{0629} \u{0648}\u{0627}\u{062d}\u{062f}\u{0629}.",
            "sw": "Chemsha maji kwa dakika moja kuua viini hatari."
        }
    });

    build_test_package(
        "shield-water-purification",
        vec![
            ("manifest-meta.json",   manifest_meta.to_string().into_bytes()),
            ("lesson-index.json",    lesson_index.to_string().into_bytes()),
            ("steps/step-00.json",   step_0.to_string().into_bytes()),
            ("steps/step-01.json",   step_1.to_string().into_bytes()),
            ("steps/step-02.json",   step_2.to_string().into_bytes()),
            ("steps/step-03.json",   step_3.to_string().into_bytes()),
            ("steps/step-04.json",   step_4.to_string().into_bytes()),
        ],
    )
}

// ── Spear: Basic Math & Logic ─────────────────────────────────────────────────

/// Build `spear-basic-math` v1.0.0 — 4-step counting and ordering lesson.
pub fn build_spear_math_package() -> ContentResult<Vec<u8>> {
    let manifest_meta = json!({
        "module_id": "spear-basic-math",
        "axis": "Spear",
        "category_id": "spear-logic",
        "title_key": "spear.math.title",
        "description_key": "spear.math.desc",
        "estimated_minutes": 8,
        "locales": ["en"]
    });

    let lesson_index = json!({
        "lesson_id": "spear-math-lesson-01",
        "module_id": "spear-basic-math",
        "title_key": "spear.math.lesson1.title",
        "sort_order": 0,
        "step_count": 4
    });

    let step_0 = json!({
        "step_id": "math-s0", "sort_order": 0,
        "content_type": "text",
        "text_key": "spear.math.s0.text",
        "strings": {
            "en": "Numbers help us count resources, trade fairly, and plan ahead."
        }
    });

    let step_1 = json!({
        "step_id": "math-s1", "sort_order": 1,
        "content_type": "multiple_choice",
        "question_key": "spear.math.s1.question",
        "strings": { "en": "You have 3 bags of rice. You give away 1. How many do you have?" },
        "options": [
            { "id": "A", "strings": { "en": "1" } },
            { "id": "B", "strings": { "en": "2" } },
            { "id": "C", "strings": { "en": "4" } }
        ],
        "correct_id": "B"
    });

    let step_2 = json!({
        "step_id": "math-s2", "sort_order": 2,
        "content_type": "ordering",
        "question_key": "spear.math.s2.question",
        "strings": { "en": "Put these amounts in order from smallest to largest:" },
        "items": [
            { "id": "five",  "strings": { "en": "5" } },
            { "id": "one",   "strings": { "en": "1" } },
            { "id": "three", "strings": { "en": "3" } }
        ],
        "correct_order": ["one", "three", "five"]
    });

    let step_3 = json!({
        "step_id": "math-s3", "sort_order": 3,
        "content_type": "text",
        "text_key": "spear.math.s3.summary",
        "strings": {
            "en": "Well done. Counting and ordering help you manage what you have and plan for tomorrow."
        }
    });

    build_test_package(
        "spear-basic-math",
        vec![
            ("manifest-meta.json",  manifest_meta.to_string().into_bytes()),
            ("lesson-index.json",   lesson_index.to_string().into_bytes()),
            ("steps/step-00.json",  step_0.to_string().into_bytes()),
            ("steps/step-01.json",  step_1.to_string().into_bytes()),
            ("steps/step-02.json",  step_2.to_string().into_bytes()),
            ("steps/step-03.json",  step_3.to_string().into_bytes()),
        ],
    )
}

// ── Shield: First Aid Basics ──────────────────────────────────────────────────

/// Build `shield-first-aid-basics` v1.0.0 — 3-step emergency response primer.
pub fn build_shield_first_aid_package() -> ContentResult<Vec<u8>> {
    let manifest_meta = json!({
        "module_id": "shield-first-aid-basics",
        "axis": "Shield",
        "category_id": "shield-medical",
        "title_key": "shield.firstaid.title",
        "description_key": "shield.firstaid.desc",
        "estimated_minutes": 7,
        "locales": ["en", "ar"]
    });

    let lesson_index = json!({
        "lesson_id": "shield-firstaid-lesson-01",
        "module_id": "shield-first-aid-basics",
        "title_key": "shield.firstaid.lesson1.title",
        "sort_order": 0,
        "step_count": 3
    });

    let step_0 = json!({
        "step_id": "aid-s0", "sort_order": 0,
        "content_type": "text",
        "text_key": "shield.firstaid.s0.text",
        "strings": {
            "en": "In an emergency: Stay calm. Check for danger. Call for help if possible.",
            "ar": "\u{0641}\u{064a} \u{062d}\u{0627}\u{0644}\u{0629} \u{0637}\u{0648}\u{0627}\u{0631}\u{0626}: \u{0627}\u{0628}\u{0642} \u{0647}\u{0627}\u{062f}\u{0626}\u{064b}\u{0627}."
        }
    });

    let step_1 = json!({
        "step_id": "aid-s1", "sort_order": 1,
        "content_type": "ordering",
        "question_key": "shield.firstaid.s1.question",
        "strings": { "en": "Put these emergency steps in the right order:" },
        "items": [
            { "id": "assess", "strings": { "en": "Check for danger" } },
            { "id": "help",   "strings": { "en": "Call for help" } },
            { "id": "calm",   "strings": { "en": "Stay calm" } }
        ],
        "correct_order": ["calm", "assess", "help"]
    });

    let step_2 = json!({
        "step_id": "aid-s2", "sort_order": 2,
        "content_type": "acknowledge",
        "text_key": "shield.firstaid.s2.confirm",
        "strings": {
            "en": "I know the three steps: Stay calm, check for danger, call for help."
        }
    });

    build_test_package(
        "shield-first-aid-basics",
        vec![
            ("manifest-meta.json", manifest_meta.to_string().into_bytes()),
            ("lesson-index.json",  lesson_index.to_string().into_bytes()),
            ("steps/step-00.json", step_0.to_string().into_bytes()),
            ("steps/step-01.json", step_1.to_string().into_bytes()),
            ("steps/step-02.json", step_2.to_string().into_bytes()),
        ],
    )
}

/// Build all sample packages as `(module_id, nmp_bytes)` pairs.
pub fn build_all_sample_packages() -> Vec<(&'static str, Vec<u8>)> {
    let mut out = Vec::new();
    if let Ok(b) = build_shield_water_package() {
        out.push(("shield-water-purification", b));
    }
    if let Ok(b) = build_spear_math_package() {
        out.push(("spear-basic-math", b));
    }
    if let Ok(b) = build_shield_first_aid_package() {
        out.push(("shield-first-aid-basics", b));
    }
    out
}
