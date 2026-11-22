//! Pure Rust Vietnamese Syllable Parser & Telex IME Engine.
//!
//! Replaces ad-hoc replacement with an architectural Vietnamese Syllable Model:
//! Initial Consonant (Phụ âm đầu) + Rhyme (Vần: Medial + Nucleus + Final Consonant) + Tone + Keystroke History.

use super::unicode_map::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineAction {
    /// Keystroke is passed through unmodified; engine records it
    PassThrough,
    /// Backspace `delete_count` characters from screen, then type `replacement`
    Transform {
        delete_count: usize,
        replacement: String,
    },
    /// Reset word boundary state (e.g. on whitespace, punctuation, mouse clicks)
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LastModifier {
    None,
    StrokeD { idx: usize },
    Circumflex { idx: usize, orig_char: char },
    HornWStandalone { idx: usize },
    HornWSingle { idx: usize, orig_char: char },
    HornWPair { idx1: usize, orig1: char, idx2: usize, orig2: char },
    Tone(Tone),
    ToneRemoved { removed_tone: Tone },
}

/// Snapshot of a committed word for the Word History buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordSnapshot {
    pub raw_chars: Vec<(char, bool)>,
    pub tone: Tone,
    pub(crate) last_modifier: LastModifier,
    pub current_display: String,
}

#[derive(Debug, Clone)]
pub struct TelexEngine {
    /// Raw characters of the syllable: (char, is_uppercase)
    pub raw_chars: Vec<(char, bool)>,
    /// Active tone applied to this syllable
    pub tone: Tone,
    /// Last applied modifier (for single-key undo on repeat)
    last_modifier: LastModifier,
    /// Displayed string currently representing this syllable
    pub current_display: String,
    /// Committed word history stack (capped at 8 words)
    pub history: Vec<WordSnapshot>,
    /// Track consecutive trailing separators (e.g. spaces) after committed words
    pub trailing_separators: usize,
}

impl Default for TelexEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TelexEngine {
    pub fn new() -> Self {
        Self {
            raw_chars: Vec::with_capacity(24),
            tone: Tone::None,
            last_modifier: LastModifier::None,
            current_display: String::with_capacity(32),
            history: Vec::with_capacity(8),
            trailing_separators: 0,
        }
    }

    /// Reset active syllable buffer cleanly (does not clear history stack)
    pub fn reset(&mut self) {
        self.raw_chars.clear();
        self.tone = Tone::None;
        self.last_modifier = LastModifier::None;
        self.current_display.clear();
    }

    /// Hard reset: clear both active syllable buffer AND entire history stack
    pub fn hard_reset(&mut self) {
        self.reset();
        self.history.clear();
        self.trailing_separators = 0;
    }

    /// Commit the current word buffer into the word history stack (capped at 8)
    pub fn commit_current_word(&mut self) {
        if !self.raw_chars.is_empty() {
            if self.history.len() >= 8 {
                self.history.remove(0);
            }
            self.history.push(WordSnapshot {
                raw_chars: self.raw_chars.clone(),
                tone: self.tone,
                last_modifier: self.last_modifier,
                current_display: self.current_display.clone(),
            });
            self.reset();
            self.trailing_separators = 1;
        } else if !self.history.is_empty() {
            self.trailing_separators += 1;
        }
    }

    /// Check if current word contains a literal 'z' / 'Z'.
    /// Because 'z' is not part of the Vietnamese alphabet, any word containing 'z'
    /// is treated as non-Vietnamese (English / foreign / raw) to prevent unintended
    /// subsequent Telex transforms on words like 'razer', 'azure', 'size', 'crazy', 'frozen', 'pizza', 'buzz'.
    #[inline]
    pub fn has_literal_z(&self) -> bool {
        self.raw_chars.iter().any(|&(c, _)| c == 'z' || c == 'Z')
    }

    /// Handle Backspace key: Smart Backspace Popping
    /// - If active syllable buffer is non-empty: pop last rendered character / raw keystroke, re-evaluate display.
    /// - If active syllable buffer is empty and history is non-empty: pop most recent word from history back into active buffer.
    pub fn backspace(&mut self) {
        if !self.raw_chars.is_empty() {
            self.raw_chars.pop();
            self.last_modifier = LastModifier::None;
            if self.raw_chars.is_empty() {
                self.tone = Tone::None;
                self.current_display.clear();
            } else {
                if !self.raw_chars.iter().any(|&(c, _)| is_vowel_char(c)) {
                    self.tone = Tone::None;
                }
                self.current_display = self.rebuild_display();
            }
        } else if !self.history.is_empty() {
            if self.trailing_separators > 1 {
                self.trailing_separators -= 1;
            } else {
                self.trailing_separators = 0;
                if let Some(last_word) = self.history.pop() {
                    self.raw_chars = last_word.raw_chars;
                    self.tone = last_word.tone;
                    self.last_modifier = last_word.last_modifier;
                    self.current_display = last_word.current_display;
                }
            }
        } else {
            self.trailing_separators = 0;
        }
    }

    /// Alias for backspace()
    pub fn on_backspace(&mut self) {
        self.backspace();
    }

    /// Process an incoming keystroke character. Returns `EngineAction`.
    pub fn process_char(&mut self, ch: char) -> EngineAction {
        // If not an alphabetic letter (space, tab, enter, punctuation, digits):
        // Commit current syllable to history if not empty, then reset
        if !ch.is_alphabetic() {
            self.commit_current_word();
            return EngineAction::Reset;
        }

        self.trailing_separators = 0;

        let is_upper = ch.is_uppercase();
        let lower = ch.to_ascii_lowercase();

        // 1. Check double-key undo for immediate repeat of modifier
        if let Some(action) = self.try_undo_repeat(lower, is_upper) {
            return action;
        }

        // 2. If active word already contains a literal 'z', mark as non-Vietnamese:
        // Skip all subsequent Telex transforms (no tones, no circumflex, no horn/breve, no stroke-d)
        // so English words like 'razer', 'azure', 'size', 'crazy', 'frozen', 'pizza', 'buzz'
        // are typed cleanly without unintended transformations.
        if self.has_literal_z() {
            self.raw_chars.push((lower, is_upper));
            self.last_modifier = LastModifier::None;
            let new_display = self.rebuild_display();
            let action = self.compute_action(&new_display, false);
            self.current_display = new_display;
            return action;
        }

        // 3. Check 'd' stroke: 'd' + 'd' -> 'đ'
        if lower == 'd' {
            if let Some(action) = self.try_stroke_d(is_upper) {
                return action;
            }
        }

        // 3. Check vowel circumflex: 'aa' -> 'â', 'ee' -> 'ê', 'oo' -> 'ô'
        if lower == 'a' || lower == 'e' || lower == 'o' {
            if let Some(action) = self.try_circumflex(lower, is_upper) {
                return action;
            }
        }

        // 4. Check horn / breve ('w')
        if lower == 'w' {
            if let Some(action) = self.try_horn_breve(is_upper) {
                return action;
            }
        }

        // 5. Check tone keys: 's', 'f', 'r', 'x', 'j', 'z'
        let tone = match lower {
            's' => Some(Tone::Acute),
            'f' => Some(Tone::Grave),
            'r' => Some(Tone::Hook),
            'x' => Some(Tone::Tilde),
            'j' => Some(Tone::Dot),
            'z' => Some(Tone::None),
            _ => None,
        };

        if let Some(target_tone) = tone {
            if let Some(action) = self.try_apply_tone(target_tone) {
                return action;
            }
        }

        // Default: append character to word buffer
        self.raw_chars.push((lower, is_upper));
        self.last_modifier = LastModifier::None;
        let new_display = self.rebuild_display();
        let action = self.compute_action(&new_display, false);
        self.current_display = new_display;
        action
    }

    /// Double-key undo handler: pressing modifier twice reverts to literal
    fn try_undo_repeat(&mut self, lower: char, is_upper: bool) -> Option<EngineAction> {
        let is_undo = match (self.last_modifier, lower) {
            (LastModifier::StrokeD { .. }, 'd') => true,
            (LastModifier::Circumflex { orig_char, .. }, k) if k == orig_char => true,
            (LastModifier::HornWStandalone { .. }, 'w') => true,
            (LastModifier::HornWSingle { .. }, 'w') => true,
            (LastModifier::HornWPair { .. }, 'w') => true,
            (LastModifier::Tone(t), k) => match (t, k) {
                (Tone::Acute, 's') => true,
                (Tone::Grave, 'f') => true,
                (Tone::Hook, 'r') => true,
                (Tone::Tilde, 'x') => true,
                (Tone::Dot, 'j') => true,
                _ => false,
            },
            (LastModifier::ToneRemoved { .. }, 'z') => true,
            _ => false,
        };

        if !is_undo {
            return None;
        }

        match self.last_modifier {
            LastModifier::StrokeD { idx } => {
                if idx < self.raw_chars.len() {
                    let (_, up) = self.raw_chars[idx];
                    self.raw_chars[idx] = ('d', up);
                }
                self.raw_chars.push(('d', is_upper));
            }
            LastModifier::Circumflex { idx, orig_char } => {
                if idx < self.raw_chars.len() {
                    let (_, up) = self.raw_chars[idx];
                    self.raw_chars[idx] = (orig_char, up);
                }
                self.raw_chars.push((orig_char, is_upper));
            }
            LastModifier::HornWStandalone { idx } => {
                // "Standalone w at the start of a syllable -> ư, and pressing w again (ww) reverts strictly to literal w (not uw)."
                if idx < self.raw_chars.len() {
                    self.raw_chars[idx] = ('w', is_upper);
                }
            }
            LastModifier::HornWSingle { idx, orig_char } => {
                // "u + w -> ư, and uw + w reverts to uw."
                if idx < self.raw_chars.len() {
                    let (_, up) = self.raw_chars[idx];
                    self.raw_chars[idx] = (orig_char, up);
                }
                self.raw_chars.push(('w', is_upper));
            }
            LastModifier::HornWPair { idx1, orig1, idx2, orig2 } => {
                if idx1 < self.raw_chars.len() && idx2 < self.raw_chars.len() {
                    let (_, up1) = self.raw_chars[idx1];
                    let (_, up2) = self.raw_chars[idx2];
                    self.raw_chars[idx1] = (orig1, up1);
                    self.raw_chars[idx2] = (orig2, up2);
                }
                self.raw_chars.push(('w', is_upper));
            }
            LastModifier::Tone(_) => {
                self.tone = Tone::None;
                self.raw_chars.push((lower, is_upper));
            }
            LastModifier::ToneRemoved { removed_tone } => {
                self.tone = removed_tone;
                self.raw_chars.push(('z', is_upper));
            }
            LastModifier::None => unreachable!(),
        }

        self.last_modifier = LastModifier::None;
        let new_display = self.rebuild_display();
        let action = self.compute_action(&new_display, true);
        self.current_display = new_display;
        Some(action)
    }

    /// Try transforming 'd' -> 'đ' / 'Đ'
    fn try_stroke_d(&mut self, is_upper: bool) -> Option<EngineAction> {
        if let Some((idx, &(last_c, last_up))) = self.raw_chars.iter().enumerate().rev().next() {
            if last_c == 'd' {
                self.raw_chars[idx] = ('đ', is_upper || last_up);
                self.last_modifier = LastModifier::StrokeD { idx };

                let new_display = self.rebuild_display();
                let action = self.compute_action(&new_display, true);
                self.current_display = new_display;
                return Some(action);
            }
        }
        None
    }

    /// Try circumflex: 'a'->'â', 'e'->'ê', 'o'->'ô'
    fn try_circumflex(&mut self, lower: char, is_upper: bool) -> Option<EngineAction> {
        let modified_char = match lower {
            'a' => 'â',
            'e' => 'ê',
            'o' => 'ô',
            _ => return None,
        };

        // Scan backwards for a matching plain vowel (supporting delayed typing)
        for i in (0..self.raw_chars.len()).rev() {
            let (c, up) = self.raw_chars[i];
            if c == lower {
                self.raw_chars[i] = (modified_char, up || is_upper);
                self.last_modifier = LastModifier::Circumflex { idx: i, orig_char: lower };

                let new_display = self.rebuild_display();
                let action = self.compute_action(&new_display, true);
                self.current_display = new_display;
                return Some(action);
            }
        }

        None
    }

    /// Try horn/breve ('w') rules:
    /// - Standalone 'w' at start of syllable -> 'ư'
    /// - 'uo' + 'w' -> 'ươ'
    /// - 'ua' + 'w' -> 'ưa' (NOT 'uă')
    /// - 'u' + 'w' -> 'ư'
    /// - 'o' + 'w' -> 'ơ'
    /// - 'a' + 'w' -> 'ă'
    fn try_horn_breve(&mut self, is_upper: bool) -> Option<EngineAction> {
        // 1. Standalone 'w' at start of syllable
        if self.raw_chars.is_empty() {
            self.raw_chars.push(('ư', is_upper));
            self.last_modifier = LastModifier::HornWStandalone { idx: 0 };

            let new_display = self.rebuild_display();
            let action = self.compute_action(&new_display, true);
            self.current_display = new_display;
            return Some(action);
        }

        // 2. Check compound 'uo' + 'w' -> 'ươ' (or 'uơ' for 'thuở', or 'qu' + 'ơ')
        if let Some((idx_u, idx_o)) = self.find_vowel_pair('u', 'o') {
            let (_, up_u) = self.raw_chars[idx_u];
            let (_, up_o) = self.raw_chars[idx_o];

            // In 'qu', 'u' is part of initial consonant, so only 'o' gets horn -> 'quơ' ('quở')
            let is_quo = idx_u == 1
                && (self.raw_chars[0].0 == 'q' || self.raw_chars[0].0 == 'Q');

            if is_quo {
                self.raw_chars[idx_o] = ('ơ', up_o);
                self.last_modifier = LastModifier::HornWSingle { idx: idx_o, orig_char: 'o' };
                let new_display = self.rebuild_display();
                let action = self.compute_action(&new_display, true);
                self.current_display = new_display;
                return Some(action);
            }

            let is_thuo = idx_u == 2
                && self.raw_chars.len() == 4
                && (self.raw_chars[0].0 == 't' || self.raw_chars[0].0 == 'T')
                && (self.raw_chars[1].0 == 'h' || self.raw_chars[1].0 == 'H');

            if is_thuo {
                self.raw_chars[idx_u] = ('u', up_u);
                self.raw_chars[idx_o] = ('ơ', up_o);
            } else {
                self.raw_chars[idx_u] = ('ư', up_u);
                self.raw_chars[idx_o] = ('ơ', up_o);
            }

            self.last_modifier = LastModifier::HornWPair {
                idx1: idx_u,
                orig1: 'u',
                idx2: idx_o,
                orig2: 'o',
            };

            let new_display = self.rebuild_display();
            let action = self.compute_action(&new_display, true);
            self.current_display = new_display;
            return Some(action);
        }

        // 3. Check compound 'ua' + 'w' -> 'ưa' (or 'qu' + 'ă' for 'quăng')
        if let Some((idx_u, idx_a)) = self.find_vowel_pair('u', 'a') {
            let (_, up_u) = self.raw_chars[idx_u];
            let (_, up_a) = self.raw_chars[idx_a];

            // In 'qu', 'u' is part of initial consonant, so 'a' gets breve -> 'quă' ('quăng', 'quặng')
            let is_qua = idx_u == 1
                && (self.raw_chars[0].0 == 'q' || self.raw_chars[0].0 == 'Q');

            if is_qua {
                self.raw_chars[idx_a] = ('ă', up_a);
                self.last_modifier = LastModifier::HornWSingle { idx: idx_a, orig_char: 'a' };
                let new_display = self.rebuild_display();
                let action = self.compute_action(&new_display, true);
                self.current_display = new_display;
                return Some(action);
            }

            self.raw_chars[idx_u] = ('ư', up_u);
            self.raw_chars[idx_a] = ('a', up_a);
            self.last_modifier = LastModifier::HornWPair {
                idx1: idx_u,
                orig1: 'u',
                idx2: idx_a,
                orig2: 'a',
            };

            let new_display = self.rebuild_display();
            let action = self.compute_action(&new_display, true);
            self.current_display = new_display;
            return Some(action);
        }

        // 4. Single vowel horn/breve ('u'->'ư', 'o'->'ơ', 'a'->'ă')
        for i in (0..self.raw_chars.len()).rev() {
            let (c, up) = self.raw_chars[i];
            match c {
                'u' => {
                    self.raw_chars[i] = ('ư', up || is_upper);
                    self.last_modifier = LastModifier::HornWSingle { idx: i, orig_char: 'u' };
                    let new_display = self.rebuild_display();
                    let action = self.compute_action(&new_display, true);
                    self.current_display = new_display;
                    return Some(action);
                }
                'o' => {
                    self.raw_chars[i] = ('ơ', up || is_upper);
                    self.last_modifier = LastModifier::HornWSingle { idx: i, orig_char: 'o' };
                    let new_display = self.rebuild_display();
                    let action = self.compute_action(&new_display, true);
                    self.current_display = new_display;
                    return Some(action);
                }
                'a' => {
                    self.raw_chars[i] = ('ă', up || is_upper);
                    self.last_modifier = LastModifier::HornWSingle { idx: i, orig_char: 'a' };
                    let new_display = self.rebuild_display();
                    let action = self.compute_action(&new_display, true);
                    self.current_display = new_display;
                    return Some(action);
                }
                _ => {}
            }
        }

        // 5. Standalone 'w' after consonant (e.g. 'tw' -> 'tư', 'sw' -> 'sư')
        // If preceded by a vowel that does not combine with 'w' (e.g. 'e', 'i', 'y' as in 'new', 'view'),
        // do not map 'w' to 'ư' (preserves English words).
        if let Some(&(last_c, _)) = self.raw_chars.last() {
            if is_vowel_char(last_c) {
                return None;
            }
        }

        let idx = self.raw_chars.len();
        self.raw_chars.push(('ư', is_upper));
        self.last_modifier = LastModifier::HornWStandalone { idx };
        let new_display = self.rebuild_display();
        let action = self.compute_action(&new_display, true);
        self.current_display = new_display;
        Some(action)
    }

    /// Find index pair of two adjacent vowels in raw_chars
    fn find_vowel_pair(&self, v1: char, v2: char) -> Option<(usize, usize)> {
        for i in 0..self.raw_chars.len().saturating_sub(1) {
            if self.raw_chars[i].0 == v1 && self.raw_chars[i + 1].0 == v2 {
                return Some((i, i + 1));
            }
        }
        None
    }

    /// Try applying or switching tone marks (Free Tone Marking)
    fn try_apply_tone(&mut self, target_tone: Tone) -> Option<EngineAction> {
        // If syllable has no vowels, do not treat as tone key (pass through as consonant)
        let has_vowel = self.raw_chars.iter().any(|&(c, _)| is_vowel_char(c));
        if !has_vowel {
            return None;
        }

        // Special handling for Tone::None ('z'):
        // Only treat 'z' as a tone-removal command if the syllable currently has a non-empty tone.
        // If tone is already None, return None so 'z' is emitted as a literal consonant.
        if target_tone == Tone::None {
            if self.tone == Tone::None {
                return None;
            }
            let removed_tone = self.tone;
            self.tone = Tone::None;
            self.last_modifier = LastModifier::ToneRemoved { removed_tone };

            let new_display = self.rebuild_display();
            let action = self.compute_action(&new_display, true);
            self.current_display = new_display;
            return Some(action);
        }

        // UniKey rule: Stop consonants (c, ch, p, t) can ONLY take Acute ('s') or Dot ('j') tones in Vietnamese.
        // Grave ('f'), Hook ('r'), and Tilde ('x') are phonetically invalid on stop consonants.
        if (target_tone == Tone::Grave || target_tone == Tone::Hook || target_tone == Tone::Tilde)
            && self.ends_with_stop_consonant()
        {
            return None;
        }

        self.tone = target_tone;
        self.last_modifier = LastModifier::Tone(target_tone);

        let new_display = self.rebuild_display();
        let action = self.compute_action(&new_display, true);
        self.current_display = new_display;
        Some(action)
    }

    /// Check if word buffer ends in Vietnamese stop consonants: c, ch, p, t
    fn ends_with_stop_consonant(&self) -> bool {
        let len = self.raw_chars.len();
        if len == 0 {
            return false;
        }

        // Check final 'ch'
        if len >= 2 && self.raw_chars[len - 2].0 == 'c' && self.raw_chars[len - 1].0 == 'h' {
            return self.raw_chars[..len - 2].iter().any(|&(c, _)| is_vowel_char(c));
        }

        // Check final 'c', 'p', 't'
        let last_c = self.raw_chars[len - 1].0;
        if last_c == 'c' || last_c == 'p' || last_c == 't' {
            return self.raw_chars[..len - 1].iter().any(|&(c, _)| is_vowel_char(c));
        }

        false
    }

    /// Rebuild display string from current `raw_chars` with `self.tone` applied
    fn rebuild_display(&self) -> String {
        self.build_syllable_with_tone(self.tone)
    }

    /// Build the full Vietnamese syllable with proper Tone Placement
    fn build_syllable_with_tone(&self, tone: Tone) -> String {
        if self.raw_chars.is_empty() {
            return String::new();
        }

        let mut chars: Vec<(char, bool)> = self.raw_chars.clone();

        if tone == Tone::None {
            // Strip any tones, assemble string
            return chars
                .iter()
                .map(|&(c, up)| {
                    let stripped = strip_tone_from_char(c);
                    if up { to_vietnamese_uppercase(stripped) } else { stripped }
                })
                .collect();
        }

        // 1. Identify initial consonant edge cases: 'qu' and 'gi'
        let mut vowel_start_idx = 0;
        let len = chars.len();

        if len >= 2 {
            let c0 = chars[0].0;
            let c1 = chars[1].0;

            // 'qu': 'u' is part of initial consonant cluster
            if (c0 == 'q' || c0 == 'Q') && (c1 == 'u' || c1 == 'U' || c1 == 'ư' || c1 == 'Ư') {
                vowel_start_idx = 2; // Vowels start AFTER 'qu'
            }
            // 'gi': if followed by another vowel, 'i' is part of initial consonant
            else if (c0 == 'g' || c0 == 'G') && (c1 == 'i' || c1 == 'I') {
                let has_another_vowel = chars[2..].iter().any(|&(c, _)| is_vowel_char(c));
                if has_another_vowel {
                    vowel_start_idx = 2; // Vowels start AFTER 'gi'
                } else {
                    vowel_start_idx = 1; // 'i' is the vowel!
                }
            }
        }

        // 2. Collect vowel indices in rhyme (stack-allocated buffer - zero heap allocation)
        let mut vowel_indices = [0usize; 8];
        let mut vowel_count = 0;
        for i in vowel_start_idx..chars.len() {
            if is_vowel_char(chars[i].0) && vowel_count < vowel_indices.len() {
                vowel_indices[vowel_count] = i;
                vowel_count += 1;
            }
        }
        let active_vowels = &vowel_indices[..vowel_count];

        if vowel_count == 0 {
            // No rhyming vowel found; assemble un-toned
            return chars
                .iter()
                .map(|&(c, up)| if up { to_vietnamese_uppercase(c) } else { c })
                .collect();
        }

        // 3. Determine target vowel index according to Standard Tone Placement Rules
        let target_idx = self.select_tone_vowel_index(&chars, active_vowels);

        // 4. Apply tone to target vowel, ensure all others have Tone::None
        for (i, item) in chars.iter_mut().enumerate() {
            if active_vowels.contains(&i) {
                let stripped = strip_tone_from_char(item.0);
                if i == target_idx {
                    item.0 = apply_tone_to_char(stripped, tone);
                } else {
                    item.0 = stripped;
                }
            }
        }

        // 5. Format casing
        chars
            .iter()
            .map(|&(c, up)| if up { to_vietnamese_uppercase(c) } else { c })
            .collect()
    }

    /// Select the vowel index that receives the tone mark based on Standard Vietnamese Orthography
    fn select_tone_vowel_index(&self, chars: &[(char, bool)], vowel_indices: &[usize]) -> usize {
        let count = vowel_indices.len();
        if count == 1 {
            return vowel_indices[0];
        }

        // Rule 1: Circumflex, horn, and breve priority (â, ă, ê, ô, ơ)
        // In UniKey: if (info.roofPos != -1) return info.roofPos;
        for &idx in vowel_indices {
            let c = chars[idx].0;
            if c == 'â' || c == 'ă' || c == 'ê' || c == 'ô' || c == 'ơ' {
                return idx;
            }
        }

        let last_vowel_idx = *vowel_indices.last().unwrap();
        let has_final_consonant = last_vowel_idx < chars.len() - 1;

        // Rule 2: If there is a final consonant (c, ch, m, n, ng, nh, p, t), tone goes on the LAST vowel
        if has_final_consonant {
            return last_vowel_idx;
        }

        // Rule 3: Open syllable with 2 vowels
        if count == 2 {
            let v1 = chars[vowel_indices[0]].0;
            let v2 = chars[vowel_indices[1]].0;

            // Diphthongs 'oa', 'oe', 'uy': tone on second vowel ('hoá', 'khoẻ', 'thuỷ', 'quý')
            if (v1 == 'o' && (v2 == 'a' || v2 == 'e')) || (v1 == 'u' && v2 == 'y') {
                return vowel_indices[1];
            }

            // Other open diphthongs ('ua', 'ưa', 'ia', 'ai', 'ao', 'au', 'ay', ...): tone on FIRST vowel ('sửa', 'mía', 'mùa', 'mái')
            return vowel_indices[0];
        }

        // Rule 4: Open syllable with 3 vowels ('iêu', 'yêu', 'oai', 'uôi', 'ươi', 'uya', 'uyê'): tone on MIDDLE vowel
        if count >= 3 {
            return vowel_indices[1];
        }

        vowel_indices[0]
    }

    /// Compute minimal delete_count and replacement string between current_display and new_display
    fn compute_action(&self, new_display: &str, is_transform: bool) -> EngineAction {
        if !is_transform && self.current_display == new_display {
            return EngineAction::PassThrough;
        }

        // Find common prefix length
        let prev_chars: Vec<char> = self.current_display.chars().collect();
        let new_chars: Vec<char> = new_display.chars().collect();

        let mut common_prefix_len = 0;
        let min_len = prev_chars.len().min(new_chars.len());

        while common_prefix_len < min_len && prev_chars[common_prefix_len] == new_chars[common_prefix_len] {
            common_prefix_len += 1;
        }

        let delete_count = prev_chars.len() - common_prefix_len;
        let replacement: String = new_chars[common_prefix_len..].iter().collect();

        if delete_count == 0 && replacement.chars().count() == 1 && !is_transform {
            EngineAction::PassThrough
        } else {
            EngineAction::Transform {
                delete_count,
                replacement,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_word(word: &str) -> String {
        let mut engine = TelexEngine::new();
        let mut output = String::new();

        for ch in word.chars() {
            match engine.process_char(ch) {
                EngineAction::PassThrough => {
                    output.push(ch);
                }
                EngineAction::Transform { delete_count, replacement } => {
                    for _ in 0..delete_count {
                        output.pop();
                    }
                    output.push_str(&replacement);
                }
                EngineAction::Reset => {
                    output.push(ch);
                }
            }
        }
        output
    }

    #[test]
    fn test_required_cases_batch_1() {
        // 1. s+u+a+w+r -> sửa
        assert_eq!(type_word("suawr"), "sửa");
        // 2. w+w -> w
        assert_eq!(type_word("ww"), "w");
        // 3. u+w+w -> uw
        assert_eq!(type_word("uww"), "uw");
        // 4. t+r+u+o+n+g+w+f -> trường
        assert_eq!(type_word("truongwf"), "trường");
        // 5. n+g+u+o+i+w+f -> người
        assert_eq!(type_word("nguoiwf"), "người");
        // 6. q+u+a+r -> quả
        assert_eq!(type_word("quar"), "quả");
        // 7. g+i+a+f -> già
        assert_eq!(type_word("giaf"), "già");
        // 8. g+i+f -> gì
        assert_eq!(type_word("gif"), "gì");
        // 9. k+h+u+y+a -> khuya
        assert_eq!(type_word("khuya"), "khuya");
        // 10. n+g+u+y+e+n+e+x -> nguyễn
        assert_eq!(type_word("nguyenex"), "nguyễn");
        // 11. d+d+a+k+w+s -> đắk
        assert_eq!(type_word("ddakws"), "đắk");
        // 12. free-order typing (t+r+u+o+f+n+g+w -> trường)
        assert_eq!(type_word("truofngw"), "trường");
        // 13. tone-switching (a+s+f -> à)
        assert_eq!(type_word("asf"), "à");
        // 14. clear tone (a+s+z -> a)
        assert_eq!(type_word("asz"), "a");
        // 15. tone undo on repeat (a+s+s -> as)
        assert_eq!(type_word("ass"), "as");
    }

    #[test]
    fn test_circumflex_and_undo() {
        assert_eq!(type_word("aa"), "â");
        assert_eq!(type_word("aaa"), "aa");
        assert_eq!(type_word("ee"), "ê");
        assert_eq!(type_word("eee"), "ee");
        assert_eq!(type_word("oo"), "ô");
        assert_eq!(type_word("ooo"), "oo");
        assert_eq!(type_word("dd"), "đ");
        assert_eq!(type_word("ddd"), "dd");
        assert_eq!(type_word("aww"), "aw");
        assert_eq!(type_word("oww"), "ow");
    }

    #[test]
    fn test_real_vietnamese_syllables_batch_2() {
        // Words with final consonants (both standard and delayed modifier typing)
        assert_eq!(type_word("toans"), "toán");
        assert_eq!(type_word("hoangf"), "hoàng");
        assert_eq!(type_word("tuaanf"), "tuần");
        assert_eq!(type_word("tuanaf"), "tuần");
        assert_eq!(type_word("tieengs"), "tiếng");
        assert_eq!(type_word("tienges"), "tiếng");
        assert_eq!(type_word("cuoois"), "cuối");
        assert_eq!(type_word("cuoios"), "cuối");
        assert_eq!(type_word("hieeur"), "hiểu");
        assert_eq!(type_word("hieuer"), "hiểu");
        assert_eq!(type_word("ngoaif"), "ngoài");
        assert_eq!(type_word("tuoiw"), "tươi");
        assert_eq!(type_word("tuoiwf"), "tười");
        assert_eq!(type_word("duyeetj"), "duyệt");
        assert_eq!(type_word("duyetej"), "duyệt");
        assert_eq!(type_word("luaanj"), "luận");
        assert_eq!(type_word("vieetj"), "việt");
        assert_eq!(type_word("vietej"), "việt");
    }

    #[test]
    fn test_real_vietnamese_syllables_batch_3() {
        // Open syllables
        assert_eq!(type_word("mais"), "mái");
        assert_eq!(type_word("muaf"), "mùa");
        assert_eq!(type_word("mias"), "mía");
        assert_eq!(type_word("thuyr"), "thuỷ");
        assert_eq!(type_word("hoas"), "hoá");
        assert_eq!(type_word("khoer"), "khoẻ");
        assert_eq!(type_word("quys"), "quý");
        assert_eq!(type_word("quocos"), "quốc");
        assert_eq!(type_word("ruouwj"), "rượu");
        assert_eq!(type_word("huouw"), "hươu");
        assert_eq!(type_word("cuawr"), "cửa");
        assert_eq!(type_word("muaw"), "mưa");
        assert_eq!(type_word("bayr"), "bảy");
        assert_eq!(type_word("bayar"), "bẩy");
        assert_eq!(type_word("chows"), "chớ");
        assert_eq!(type_word("bucws"), "bức");
        assert_eq!(type_word("xuongw"), "xương");
        assert_eq!(type_word("thuowr"), "thuở");
        assert_eq!(type_word("ddawngs"), "đắng");
    }

    #[test]
    fn test_case_preservation() {
        assert_eq!(type_word("Truongwf"), "Trường");
        assert_eq!(type_word("DDAKWS"), "ĐẮK");
        assert_eq!(type_word("VIEETJ"), "VIỆT");
        assert_eq!(type_word("VIETEJ"), "VIỆT");
        assert_eq!(type_word("NAM"), "NAM");
        assert_eq!(type_word("QUAR"), "QUẢ");
        assert_eq!(type_word("GIAF"), "GIÀ");
    }

    #[test]
    fn test_unikey_synced_edge_cases() {
        // 1. 'qu' + 'w' edge cases from UniKey (u in qu is consonant, not part of compound vowel)
        assert_eq!(type_word("quawngs"), "quắng");
        assert_eq!(type_word("quawngj"), "quặng");
        assert_eq!(type_word("quawts"), "quắt");
        assert_eq!(type_word("quowr"), "quở");

        // 2. 'gi' edge cases from UniKey cs_gi and cs_gin
        assert_eq!(type_word("ginf"), "gìn");
        assert_eq!(type_word("gieengs"), "giếng");
        assert_eq!(type_word("gienges"), "giếng");
        assert_eq!(type_word("giuax"), "giũa");
        assert_eq!(type_word("giuawx"), "giữa");

        // 3. 'oă' diphthong
        assert_eq!(type_word("hoawcj"), "hoặc");
        assert_eq!(type_word("thoawts"), "thoắt");

        // 4. Circumflex priority on 'â' (roofPos in UniKey)
        assert_eq!(type_word("cauf"), "càu"); // cau mày -> càu nhàu
        assert_eq!(type_word("caauf"), "cầu"); // cây cầu
        assert_eq!(type_word("cayf"), "cày"); // cày ruộng
        assert_eq!(type_word("caayf"), "cầy"); // con cầy
        assert_eq!(type_word("khuaays"), "khuấy"); // khuấy nước

        // 5. 'uyê' triphthong tone placement on 'ê'
        assert_eq!(type_word("chuyeens"), "chuyến");
        assert_eq!(type_word("chuyenes"), "chuyến");
        assert_eq!(type_word("truyeenj"), "truyện");
        assert_eq!(type_word("truyenej"), "truyện");

        // 6. English words with 'w' after 'e'/'i' preserved
        assert_eq!(type_word("new"), "new");
        assert_eq!(type_word("view"), "view");

        // 7. Stop consonant (c, ch, p, t) tone rule from UniKey:
        // Suffixes c, ch, p, t do not allow Grave (`), Hook (?), Tilde (~)
        assert_eq!(type_word("catf"), "catf");
        assert_eq!(type_word("batf"), "batf");
        assert_eq!(type_word("cats"), "cát");
        assert_eq!(type_word("bats"), "bát");
        assert_eq!(type_word("batj"), "bạt");
    }

    #[test]
    fn test_mid_word_repair() {
        let mut engine = TelexEngine::new();
        let mut output = String::new();

        // 1. Type "ddesn" -> "đén"
        for ch in "ddesn".chars() {
            match engine.process_char(ch) {
                EngineAction::PassThrough => output.push(ch),
                EngineAction::Transform { delete_count, replacement } => {
                    for _ in 0..delete_count { output.pop(); }
                    output.push_str(&replacement);
                }
                EngineAction::Reset => output.push(ch),
            }
        }
        assert_eq!(output, "đén");

        // 2. Physical Backspace deletes 'n' on screen
        output.pop();
        engine.backspace();
        assert_eq!(output, "đé");
        assert_eq!(engine.current_display, "đé");

        // 3. Type "en" -> transforms "đé" -> "đế" -> "đến"
        for ch in "en".chars() {
            match engine.process_char(ch) {
                EngineAction::PassThrough => output.push(ch),
                EngineAction::Transform { delete_count, replacement } => {
                    for _ in 0..delete_count { output.pop(); }
                    output.push_str(&replacement);
                }
                EngineAction::Reset => output.push(ch),
            }
        }
        assert_eq!(output, "đến");
    }

    #[test]
    fn test_cross_space_restoration() {
        let mut engine = TelexEngine::new();
        let mut output = String::new();

        // 1. Type "hoaf" -> "hoà"
        for ch in "hoaf".chars() {
            match engine.process_char(ch) {
                EngineAction::PassThrough => output.push(ch),
                EngineAction::Transform { delete_count, replacement } => {
                    for _ in 0..delete_count { output.pop(); }
                    output.push_str(&replacement);
                }
                EngineAction::Reset => output.push(ch),
            }
        }
        assert_eq!(output, "hoà");

        // 2. Space commits "hoà" to history
        match engine.process_char(' ') {
            EngineAction::PassThrough => output.push(' '),
            EngineAction::Transform { delete_count, replacement } => {
                for _ in 0..delete_count { output.pop(); }
                output.push_str(&replacement);
            }
            EngineAction::Reset => output.push(' '),
        }
        assert_eq!(output, "hoà ");
        assert_eq!(engine.history.len(), 1);

        // 3. Physical Backspace deletes Space on screen and restores "hoà" into active buffer
        output.pop();
        engine.backspace();
        assert_eq!(output, "hoà");
        assert_eq!(engine.current_display, "hoà");
        assert!(engine.history.is_empty());

        // 4. Type 's' switches tone grave -> acute -> "hoá"
        match engine.process_char('s') {
            EngineAction::PassThrough => output.push('s'),
            EngineAction::Transform { delete_count, replacement } => {
                for _ in 0..delete_count { output.pop(); }
                output.push_str(&replacement);
            }
            EngineAction::Reset => output.push('s'),
        }
        assert_eq!(output, "hoá");
    }

    #[test]
    fn test_desync_hard_reset() {
        let mut engine = TelexEngine::new();
        engine.process_char('h');
        engine.process_char('o');
        engine.process_char('a');
        engine.process_char('f');
        assert_eq!(engine.current_display, "hoà");

        // Commit to history via space
        engine.process_char(' ');
        assert_eq!(engine.history.len(), 1);
        assert!(engine.raw_chars.is_empty());

        // Type another word
        engine.process_char('t');
        engine.process_char('e');
        engine.process_char('e');
        assert_eq!(engine.current_display, "tê");

        // hard_reset wipes BOTH active buffer AND history stack
        engine.hard_reset();
        assert!(engine.raw_chars.is_empty());
        assert_eq!(engine.tone, Tone::None);
        assert!(engine.current_display.is_empty());
        assert!(engine.history.is_empty());
    }

    #[test]
    fn test_word_history_cap_at_8() {
        let mut engine = TelexEngine::new();
        for i in 0..12 {
            engine.process_char(('a' as u8 + i) as char);
            engine.process_char(' ');
        }
        assert_eq!(engine.history.len(), 8);
    }

    fn assert_clean_nfc_word(typed: &str, expected: &str) {
        let result = type_word(typed);
        assert_eq!(result, expected, "Failed typing word: {}", typed);
        assert!(!result.contains(' '), "Word contains stray space: {}", result);
        for ch in result.chars() {
            let code = ch as u32;
            assert!(
                code != 0x200B && code != 0x200C && code != 0x200D && code != 0x2060 && code != 0xFEFF,
                "Result contains invisible zero-width character U+{:04X}: {}", code, result
            );
            assert!(
                !(0x0300..=0x036F).contains(&code),
                "Result contains decomposed combining mark U+{:04X}: {}", code, result
            );
        }
    }

    fn assert_clean_nfc_sentence(typed_sentence: &str, expected_sentence: &str) {
        let result = type_word(typed_sentence);
        assert_eq!(result, expected_sentence, "Failed typing sentence: {}", typed_sentence);
        for ch in result.chars() {
            let code = ch as u32;
            assert!(
                code != 0x200B && code != 0x200C && code != 0x200D && code != 0x2060 && code != 0xFEFF,
                "Sentence contains invisible zero-width character U+{:04X}: {}", code, result
            );
            assert!(
                !(0x0300..=0x036F).contains(&code),
                "Sentence contains decomposed combining mark U+{:04X}: {}", code, result
            );
        }
    }

    #[test]
    fn test_nfc_and_token_cleanliness_corpus() {
        // Individual required and core Vietnamese words
        assert_clean_nfc_word("vieetj", "việt");
        assert_clean_nfc_word("truongwf", "trường");
        assert_clean_nfc_word("nguoiwf", "người");
        assert_clean_nfc_word("suawr", "sửa");
        assert_clean_nfc_word("quawng", "quăng");
        assert_clean_nfc_word("giuawx", "giữa");
        assert_clean_nfc_word("hoaf", "hoà");
        assert_clean_nfc_word("thuyr", "thuỷ");
        assert_clean_nfc_word("khoer", "khoẻ");
        assert_clean_nfc_word("quocos", "quốc");
        assert_clean_nfc_word("ruouwj", "rượu");
        assert_clean_nfc_word("chuyeens", "chuyến");
        assert_clean_nfc_word("nguyenex", "nguyễn");
        assert_clean_nfc_word("tieengs", "tiếng");
        assert_clean_nfc_word("nhanh", "nhanh");
        assert_clean_nfc_word("nhejn", "nhẹn");
        assert_clean_nfc_word("nheejn", "nhện");
        assert_clean_nfc_word("ddas", "đá");
        assert_clean_nfc_word("monf", "mòn");
        assert_clean_nfc_word("hanhj", "hạnh");
        assert_clean_nfc_word("phusc", "phúc");
        assert_clean_nfc_word("ddoocj", "độc");
        assert_clean_nfc_word("laapj", "lập");
        assert_clean_nfc_word("tuwj", "tự");
        assert_clean_nfc_word("do", "do");

        // Broad corpus sentences
        assert_clean_nfc_sentence(
            "coongj hoaf xax hooij chur nghiax vieetj nam",
            "cộng hoà xã hội chủ nghĩa việt nam",
        );
        assert_clean_nfc_sentence(
            "ddoocj laapj - tuwj do - hanhj phusc",
            "độc lập - tự do - hạnh phúc",
        );
        assert_clean_nfc_sentence(
            "truowngf sa, hoangf sa laf cuar vieetj nam",
            "trường sa, hoàng sa là của việt nam",
        );
        assert_clean_nfc_sentence(
            "nuowcs chayr ddas monf",
            "nước chảy đá mòn",
        );
        assert_clean_nfc_sentence(
            "awn quar nhows ker troongf caay",
            "ăn quả nhớ kẻ trồng cây",
        );
    }

    #[test]
    fn test_telex_z_specification() {
        // 1. Tone removal only when tone exists:
        assert_eq!(type_word("raz"), "raz");
        assert_eq!(type_word("RAZ"), "RAZ");
        assert_eq!(type_word("siz"), "siz");
        assert_eq!(type_word("quiz"), "quiz");
        assert_eq!(type_word("zoo"), "zoo");

        // 2. English words with literal 'z' do not trigger retroactive Telex transforms:
        assert_eq!(type_word("razer"), "razer");
        assert_eq!(type_word("azure"), "azure");
        assert_eq!(type_word("size"), "size");
        assert_eq!(type_word("crazy"), "crazy");
        assert_eq!(type_word("frozen"), "frozen");
        assert_eq!(type_word("pizza"), "pizza");
        assert_eq!(type_word("buzz"), "buzz");

        // 3. Valid tone removal:
        assert_eq!(type_word("rasz"), "ra");
        assert_eq!(type_word("toasz"), "toa");

        // 4. Double-z undo:
        assert_eq!(type_word("raszz"), "ráz");
        assert_eq!(type_word("toaszz"), "toáz");
    }

    #[test]
    fn test_telex_z_backspace() {
        let mut engine = TelexEngine::new();
        let mut output = String::new();

        // Type "raz"
        for ch in "raz".chars() {
            match engine.process_char(ch) {
                EngineAction::PassThrough => output.push(ch),
                EngineAction::Transform { delete_count, replacement } => {
                    for _ in 0..delete_count { output.pop(); }
                    output.push_str(&replacement);
                }
                EngineAction::Reset => output.push(ch),
            }
        }
        assert_eq!(output, "raz");
        assert_eq!(engine.current_display, "raz");

        // Press Backspace: "raz" + Backspace -> "ra"
        engine.backspace();
        output.pop();
        assert_eq!(engine.current_display, "ra");
        assert_eq!(output, "ra");

        // Subsequent typing after backspacing 'z' resumes normal Vietnamese transforms:
        // "ra" + 's' -> "rá"
        match engine.process_char('s') {
            EngineAction::PassThrough => output.push('s'),
            EngineAction::Transform { delete_count, replacement } => {
                for _ in 0..delete_count { output.pop(); }
                output.push_str(&replacement);
            }
            EngineAction::Reset => output.push('s'),
        }
        assert_eq!(output, "rá");
        assert_eq!(engine.current_display, "rá");
    }
}
