//! End-to-End Automated Verification Test for UniRush / UniKey-rs
//!
//! Tests:
//! 1. Verifies that normal inputs dispatch atomic backspace + unicode batches with ZERO mid-word Space events.
//! 2. Verifies that all synthetic inputs carry UNIKEY_EXTRA_INFO (0x554E494B) for loopback prevention.
//! 3. Verifies zero invisible characters (no U+200B/U+2060) and strict pre-composed NFC Unicode.
//! 4. Verifies mid-word repair: "ddesn" + Backspace + "en" -> "đến".
//! 5. Verifies Omnibox autocomplete cancellation workaround (VK_SPACE + VK_BACK) is isolated.

use unirust::engine::{EngineAction, TelexEngine};
use unirust::hook::{
    build_normal_replacement_inputs, build_omnibox_cancellation_inputs, UNIKEY_EXTRA_INFO,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_BACK, VK_SPACE};

const TARGET_WORDS: &[&str] = &[
    "tiếng", "việt", "trường", "người", "sửa", "giữa", "đến", "hoà", "thuỷ",
];

#[test]
fn test_e2e_typing_test_harness() {
    let mut engine = TelexEngine::new();
    let mut screen_buffer = String::new();
    let mut completed_words: Vec<String> = Vec::new();
    let mut unexpected_space_events: Vec<String> = Vec::new();

    // Sequence of typed words mimicking Monkeytype / typing test:
    // Word 1: "tieengs " -> "tiếng"
    // Word 2: "vieetj " -> "việt"
    // Word 3: "truongwf " -> "trường"
    // Word 4: "nguoiwf " -> "người"
    // Word 5: "suawr " -> "sửa"
    // Word 6: "giuawx " -> "giữa"
    // Word 7: "ddesn" + Backspace + "en " -> "đến" (mid-word repair)
    // Word 8: "hoaf " -> "hoà"
    // Word 9: "thuyr " -> "thuỷ"

    let typing_sequences = [
        ("tieengs", false),
        ("vieetj", false),
        ("truongwf", false),
        ("nguoiwf", false),
        ("suawr", false),
        ("giuawx", false),
        ("ddesn", true), // needs backspace repair
        ("hoaf", false),
        ("thuyr", false),
    ];

    for (seq_idx, (seq, needs_repair)) in typing_sequences.iter().enumerate() {
        if *needs_repair {
            // Type "ddesn" -> "đén"
            for ch in seq.chars() {
                process_key_simulated(
                    &mut engine,
                    ch,
                    &mut screen_buffer,
                    &mut unexpected_space_events,
                    seq_idx,
                );
            }
            assert_eq!(screen_buffer, "đén");

            // User presses physical Backspace -> screen buffer deletes 'n', engine pops
            screen_buffer.pop();
            engine.backspace();
            assert_eq!(screen_buffer, "đé");

            // User continues typing "en" -> transforms to "đến"
            for ch in "en".chars() {
                process_key_simulated(
                    &mut engine,
                    ch,
                    &mut screen_buffer,
                    &mut unexpected_space_events,
                    seq_idx,
                );
            }
            assert_eq!(screen_buffer, "đến");
        } else {
            for ch in seq.chars() {
                process_key_simulated(
                    &mut engine,
                    ch,
                    &mut screen_buffer,
                    &mut unexpected_space_events,
                    seq_idx,
                );
            }
        }

        // Expected word boundary Space
        let word = screen_buffer.clone();
        completed_words.push(word);
        screen_buffer.clear();

        // Process word boundary Space in engine
        let space_action = engine.process_char(' ');
        assert!(
            matches!(space_action, EngineAction::PassThrough | EngineAction::Reset),
            "Word separator Space must never trigger character transformation!"
        );
    }

    // 1. Assert ZERO premature mid-word Space events occurred!
    assert!(
        unexpected_space_events.is_empty(),
        "PREMATURE MID-WORD SPACE DETECTED! Events:\n{}",
        unexpected_space_events.join("\n")
    );

    // 2. Assert all 9 composed words match expected target words with 100% accuracy
    assert_eq!(
        completed_words, TARGET_WORDS,
        "Composed words do not match expected target words! Got: {:?}, Expected: {:?}",
        completed_words, TARGET_WORDS
    );

    // 3. Verify Omnibox isolation: cancel inputs contain VK_SPACE + VK_BACK tagged with UNIKEY_EXTRA_INFO
    let omni_cancel = build_omnibox_cancellation_inputs();
    assert_eq!(omni_cancel.len(), 4);
    let ki0 = unsafe { omni_cancel[0].Anonymous.ki };
    assert_eq!(ki0.wVk, VK_SPACE);
    assert_eq!(ki0.dwExtraInfo, UNIKEY_EXTRA_INFO);
    let ki2 = unsafe { omni_cancel[2].Anonymous.ki };
    assert_eq!(ki2.wVk, VK_BACK);
    assert_eq!(ki2.dwExtraInfo, UNIKEY_EXTRA_INFO);

    println!("==========================================================");
    println!(" [PASS] E2E Vietnamese Typing Test 100% SUCCEEDED!");
    println!(" Words Completed: {:?}", completed_words);
    println!(" Zero premature mid-word spaces detected.");
    println!(" Atomic SendInput batches verified cleanly.");
    println!("==========================================================");
}

fn process_key_simulated(
    engine: &mut TelexEngine,
    ch: char,
    screen_buffer: &mut String,
    unexpected_spaces: &mut Vec<String>,
    word_idx: usize,
) {
    let action = engine.process_char(ch);
    match action {
        EngineAction::PassThrough => {
            screen_buffer.push(ch);
        }
        EngineAction::Transform { delete_count, replacement } => {
            // Build the atomic SendInput batch for normal inputs
            let batch = build_normal_replacement_inputs(delete_count, &replacement);

            // Audit EVERY single input in the batch
            for inp in &batch {
                let ki = unsafe { inp.Anonymous.ki };

                // Guard 1: Must NEVER contain VK_SPACE mid-word!
                if ki.wVk == VK_SPACE {
                    unexpected_spaces.push(format!(
                        "Premature VK_SPACE injected at word index {} ('{}'), char '{}', buffer was: '{}'",
                        word_idx, TARGET_WORDS[word_idx], ch, screen_buffer
                    ));
                }

                // Guard 2: Must be tagged with UNIKEY_EXTRA_INFO
                assert_eq!(
                    ki.dwExtraInfo, UNIKEY_EXTRA_INFO,
                    "Injected input missing UNIKEY_EXTRA_INFO tag!"
                );

                // Guard 3: Must NOT contain invisible characters or decomposed marks
                let unit = ki.wScan;
                assert_ne!(unit, 0x200B, "Contains U+200B zero-width space");
                assert_ne!(unit, 0x2060, "Contains U+2060 word joiner");
                assert_ne!(unit, 0xFEFF, "Contains U+FEFF zero-width no-break space");
                assert!(
                    !(0x0300..=0x036F).contains(&unit),
                    "Contains decomposed combining mark U+{:04X}",
                    unit
                );
            }

            // Apply atomic deletion and replacement to simulated screen
            for _ in 0..delete_count {
                screen_buffer.pop();
            }
            screen_buffer.push_str(&replacement);
        }
        EngineAction::Reset => {
            screen_buffer.push(ch);
        }
    }
}
