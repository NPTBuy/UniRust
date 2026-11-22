//! Pre-composed Vietnamese Unicode (NFC) character mapping and decomposition.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    None,
    Acute, // Sắc  ('s')
    Grave, // Huyền ('f')
    Hook,  // Hỏi  ('r')
    Tilde, // Ngã  ('x')
    Dot,   // Nặng ('j')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VowelMod {
    Plain,      // a, e, o, u, i, y
    Circumflex, // â, ê, ô
    Breve,      // ă
    Horn,       // ơ, ư
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseVowel {
    A,
    E,
    I,
    O,
    U,
    Y,
}

/// Compose a Vietnamese vowel from components into NFC Unicode.
pub fn compose_vowel(base: BaseVowel, modifier: VowelMod, tone: Tone, is_upper: bool) -> char {
    let lower_ch = match (base, modifier, tone) {
        // --- A ---
        (BaseVowel::A, VowelMod::Plain, Tone::None) => 'a',
        (BaseVowel::A, VowelMod::Plain, Tone::Acute) => 'á',
        (BaseVowel::A, VowelMod::Plain, Tone::Grave) => 'à',
        (BaseVowel::A, VowelMod::Plain, Tone::Hook) => 'ả',
        (BaseVowel::A, VowelMod::Plain, Tone::Tilde) => 'ã',
        (BaseVowel::A, VowelMod::Plain, Tone::Dot) => 'ạ',

        (BaseVowel::A, VowelMod::Circumflex, Tone::None) => 'â',
        (BaseVowel::A, VowelMod::Circumflex, Tone::Acute) => 'ấ',
        (BaseVowel::A, VowelMod::Circumflex, Tone::Grave) => 'ầ',
        (BaseVowel::A, VowelMod::Circumflex, Tone::Hook) => 'ẩ',
        (BaseVowel::A, VowelMod::Circumflex, Tone::Tilde) => 'ẫ',
        (BaseVowel::A, VowelMod::Circumflex, Tone::Dot) => 'ậ',

        (BaseVowel::A, VowelMod::Breve, Tone::None) => 'ă',
        (BaseVowel::A, VowelMod::Breve, Tone::Acute) => 'ắ',
        (BaseVowel::A, VowelMod::Breve, Tone::Grave) => 'ằ',
        (BaseVowel::A, VowelMod::Breve, Tone::Hook) => 'ẳ',
        (BaseVowel::A, VowelMod::Breve, Tone::Tilde) => 'ẵ',
        (BaseVowel::A, VowelMod::Breve, Tone::Dot) => 'ặ',

        // --- E ---
        (BaseVowel::E, VowelMod::Plain, Tone::None) => 'e',
        (BaseVowel::E, VowelMod::Plain, Tone::Acute) => 'é',
        (BaseVowel::E, VowelMod::Plain, Tone::Grave) => 'è',
        (BaseVowel::E, VowelMod::Plain, Tone::Hook) => 'ẻ',
        (BaseVowel::E, VowelMod::Plain, Tone::Tilde) => 'ẽ',
        (BaseVowel::E, VowelMod::Plain, Tone::Dot) => 'ẹ',

        (BaseVowel::E, VowelMod::Circumflex, Tone::None) => 'ê',
        (BaseVowel::E, VowelMod::Circumflex, Tone::Acute) => 'ế',
        (BaseVowel::E, VowelMod::Circumflex, Tone::Grave) => 'ề',
        (BaseVowel::E, VowelMod::Circumflex, Tone::Hook) => 'ể',
        (BaseVowel::E, VowelMod::Circumflex, Tone::Tilde) => 'ễ',
        (BaseVowel::E, VowelMod::Circumflex, Tone::Dot) => 'ệ',

        // --- I ---
        (BaseVowel::I, VowelMod::Plain, Tone::None) => 'i',
        (BaseVowel::I, VowelMod::Plain, Tone::Acute) => 'í',
        (BaseVowel::I, VowelMod::Plain, Tone::Grave) => 'ì',
        (BaseVowel::I, VowelMod::Plain, Tone::Hook) => 'ỉ',
        (BaseVowel::I, VowelMod::Plain, Tone::Tilde) => 'ĩ',
        (BaseVowel::I, VowelMod::Plain, Tone::Dot) => 'ị',

        // --- O ---
        (BaseVowel::O, VowelMod::Plain, Tone::None) => 'o',
        (BaseVowel::O, VowelMod::Plain, Tone::Acute) => 'ó',
        (BaseVowel::O, VowelMod::Plain, Tone::Grave) => 'ò',
        (BaseVowel::O, VowelMod::Plain, Tone::Hook) => 'ỏ',
        (BaseVowel::O, VowelMod::Plain, Tone::Tilde) => 'õ',
        (BaseVowel::O, VowelMod::Plain, Tone::Dot) => 'ọ',

        (BaseVowel::O, VowelMod::Circumflex, Tone::None) => 'ô',
        (BaseVowel::O, VowelMod::Circumflex, Tone::Acute) => 'ố',
        (BaseVowel::O, VowelMod::Circumflex, Tone::Grave) => 'ồ',
        (BaseVowel::O, VowelMod::Circumflex, Tone::Hook) => 'ổ',
        (BaseVowel::O, VowelMod::Circumflex, Tone::Tilde) => 'ỗ',
        (BaseVowel::O, VowelMod::Circumflex, Tone::Dot) => 'ộ',

        (BaseVowel::O, VowelMod::Horn, Tone::None) => 'ơ',
        (BaseVowel::O, VowelMod::Horn, Tone::Acute) => 'ớ',
        (BaseVowel::O, VowelMod::Horn, Tone::Grave) => 'ờ',
        (BaseVowel::O, VowelMod::Horn, Tone::Hook) => 'ở',
        (BaseVowel::O, VowelMod::Horn, Tone::Tilde) => 'ỡ',
        (BaseVowel::O, VowelMod::Horn, Tone::Dot) => 'ợ',

        // --- U ---
        (BaseVowel::U, VowelMod::Plain, Tone::None) => 'u',
        (BaseVowel::U, VowelMod::Plain, Tone::Acute) => 'ú',
        (BaseVowel::U, VowelMod::Plain, Tone::Grave) => 'ù',
        (BaseVowel::U, VowelMod::Plain, Tone::Hook) => 'ủ',
        (BaseVowel::U, VowelMod::Plain, Tone::Tilde) => 'ũ',
        (BaseVowel::U, VowelMod::Plain, Tone::Dot) => 'ụ',

        (BaseVowel::U, VowelMod::Horn, Tone::None) => 'ư',
        (BaseVowel::U, VowelMod::Horn, Tone::Acute) => 'ứ',
        (BaseVowel::U, VowelMod::Horn, Tone::Grave) => 'ừ',
        (BaseVowel::U, VowelMod::Horn, Tone::Hook) => 'ử',
        (BaseVowel::U, VowelMod::Horn, Tone::Tilde) => 'ữ',
        (BaseVowel::U, VowelMod::Horn, Tone::Dot) => 'ự',

        // --- Y ---
        (BaseVowel::Y, VowelMod::Plain, Tone::None) => 'y',
        (BaseVowel::Y, VowelMod::Plain, Tone::Acute) => 'ý',
        (BaseVowel::Y, VowelMod::Plain, Tone::Grave) => 'ỳ',
        (BaseVowel::Y, VowelMod::Plain, Tone::Hook) => 'ỷ',
        (BaseVowel::Y, VowelMod::Plain, Tone::Tilde) => 'ỹ',
        (BaseVowel::Y, VowelMod::Plain, Tone::Dot) => 'ỵ',

        // Fallbacks for invalid combinations (e.g. Breve/Circumflex on U)
        (base, _, tone) => return compose_vowel(base, VowelMod::Plain, tone, is_upper),
    };

    if is_upper {
        to_vietnamese_uppercase(lower_ch)
    } else {
        lower_ch
    }
}

/// Decompose a character into (BaseVowel, VowelMod, Tone, is_upper).
pub fn decompose_vowel(ch: char) -> Option<(BaseVowel, VowelMod, Tone, bool)> {
    let lower = to_vietnamese_lowercase(ch);
    let is_upper = ch.is_uppercase();

    let (base, modifier, tone) = match lower {
        'a' => (BaseVowel::A, VowelMod::Plain, Tone::None),
        'á' => (BaseVowel::A, VowelMod::Plain, Tone::Acute),
        'à' => (BaseVowel::A, VowelMod::Plain, Tone::Grave),
        'ả' => (BaseVowel::A, VowelMod::Plain, Tone::Hook),
        'ã' => (BaseVowel::A, VowelMod::Plain, Tone::Tilde),
        'ạ' => (BaseVowel::A, VowelMod::Plain, Tone::Dot),

        'â' => (BaseVowel::A, VowelMod::Circumflex, Tone::None),
        'ấ' => (BaseVowel::A, VowelMod::Circumflex, Tone::Acute),
        'ầ' => (BaseVowel::A, VowelMod::Circumflex, Tone::Grave),
        'ẩ' => (BaseVowel::A, VowelMod::Circumflex, Tone::Hook),
        'ẫ' => (BaseVowel::A, VowelMod::Circumflex, Tone::Tilde),
        'ậ' => (BaseVowel::A, VowelMod::Circumflex, Tone::Dot),

        'ă' => (BaseVowel::A, VowelMod::Breve, Tone::None),
        'ắ' => (BaseVowel::A, VowelMod::Breve, Tone::Acute),
        'ằ' => (BaseVowel::A, VowelMod::Breve, Tone::Grave),
        'ẳ' => (BaseVowel::A, VowelMod::Breve, Tone::Hook),
        'ẵ' => (BaseVowel::A, VowelMod::Breve, Tone::Tilde),
        'ặ' => (BaseVowel::A, VowelMod::Breve, Tone::Dot),

        'e' => (BaseVowel::E, VowelMod::Plain, Tone::None),
        'é' => (BaseVowel::E, VowelMod::Plain, Tone::Acute),
        'è' => (BaseVowel::E, VowelMod::Plain, Tone::Grave),
        'ẻ' => (BaseVowel::E, VowelMod::Plain, Tone::Hook),
        'ẽ' => (BaseVowel::E, VowelMod::Plain, Tone::Tilde),
        'ẹ' => (BaseVowel::E, VowelMod::Plain, Tone::Dot),

        'ê' => (BaseVowel::E, VowelMod::Circumflex, Tone::None),
        'ế' => (BaseVowel::E, VowelMod::Circumflex, Tone::Acute),
        'ề' => (BaseVowel::E, VowelMod::Circumflex, Tone::Grave),
        'ể' => (BaseVowel::E, VowelMod::Circumflex, Tone::Hook),
        'ễ' => (BaseVowel::E, VowelMod::Circumflex, Tone::Tilde),
        'ệ' => (BaseVowel::E, VowelMod::Circumflex, Tone::Dot),

        'i' => (BaseVowel::I, VowelMod::Plain, Tone::None),
        'í' => (BaseVowel::I, VowelMod::Plain, Tone::Acute),
        'ì' => (BaseVowel::I, VowelMod::Plain, Tone::Grave),
        'ỉ' => (BaseVowel::I, VowelMod::Plain, Tone::Hook),
        'ĩ' => (BaseVowel::I, VowelMod::Plain, Tone::Tilde),
        'ị' => (BaseVowel::I, VowelMod::Plain, Tone::Dot),

        'o' => (BaseVowel::O, VowelMod::Plain, Tone::None),
        'ó' => (BaseVowel::O, VowelMod::Plain, Tone::Acute),
        'ò' => (BaseVowel::O, VowelMod::Plain, Tone::Grave),
        'ỏ' => (BaseVowel::O, VowelMod::Plain, Tone::Hook),
        'õ' => (BaseVowel::O, VowelMod::Plain, Tone::Tilde),
        'ọ' => (BaseVowel::O, VowelMod::Plain, Tone::Dot),

        'ô' => (BaseVowel::O, VowelMod::Circumflex, Tone::None),
        'ố' => (BaseVowel::O, VowelMod::Circumflex, Tone::Acute),
        'ồ' => (BaseVowel::O, VowelMod::Circumflex, Tone::Grave),
        'ổ' => (BaseVowel::O, VowelMod::Circumflex, Tone::Hook),
        'ỗ' => (BaseVowel::O, VowelMod::Circumflex, Tone::Tilde),
        'ộ' => (BaseVowel::O, VowelMod::Circumflex, Tone::Dot),

        'ơ' => (BaseVowel::O, VowelMod::Horn, Tone::None),
        'ớ' => (BaseVowel::O, VowelMod::Horn, Tone::Acute),
        'ờ' => (BaseVowel::O, VowelMod::Horn, Tone::Grave),
        'ở' => (BaseVowel::O, VowelMod::Horn, Tone::Hook),
        'ỡ' => (BaseVowel::O, VowelMod::Horn, Tone::Tilde),
        'ợ' => (BaseVowel::O, VowelMod::Horn, Tone::Dot),

        'u' => (BaseVowel::U, VowelMod::Plain, Tone::None),
        'ú' => (BaseVowel::U, VowelMod::Plain, Tone::Acute),
        'ù' => (BaseVowel::U, VowelMod::Plain, Tone::Grave),
        'ủ' => (BaseVowel::U, VowelMod::Plain, Tone::Hook),
        'ũ' => (BaseVowel::U, VowelMod::Plain, Tone::Tilde),
        'ụ' => (BaseVowel::U, VowelMod::Plain, Tone::Dot),

        'ư' => (BaseVowel::U, VowelMod::Horn, Tone::None),
        'ứ' => (BaseVowel::U, VowelMod::Horn, Tone::Acute),
        'ừ' => (BaseVowel::U, VowelMod::Horn, Tone::Grave),
        'ử' => (BaseVowel::U, VowelMod::Horn, Tone::Hook),
        'ữ' => (BaseVowel::U, VowelMod::Horn, Tone::Tilde),
        'ự' => (BaseVowel::U, VowelMod::Horn, Tone::Dot),

        'y' => (BaseVowel::Y, VowelMod::Plain, Tone::None),
        'ý' => (BaseVowel::Y, VowelMod::Plain, Tone::Acute),
        'ỳ' => (BaseVowel::Y, VowelMod::Plain, Tone::Grave),
        'ỷ' => (BaseVowel::Y, VowelMod::Plain, Tone::Hook),
        'ỹ' => (BaseVowel::Y, VowelMod::Plain, Tone::Tilde),
        'ỵ' => (BaseVowel::Y, VowelMod::Plain, Tone::Dot),

        _ => return None,
    };

    Some((base, modifier, tone, is_upper))
}

pub fn is_vowel_char(ch: char) -> bool {
    decompose_vowel(ch).is_some()
}

pub fn strip_tone_from_char(ch: char) -> char {
    if let Some((base, modifier, _, is_upper)) = decompose_vowel(ch) {
        compose_vowel(base, modifier, Tone::None, is_upper)
    } else {
        ch
    }
}

pub fn apply_tone_to_char(ch: char, tone: Tone) -> char {
    if let Some((base, modifier, _, is_upper)) = decompose_vowel(ch) {
        compose_vowel(base, modifier, tone, is_upper)
    } else {
        ch
    }
}

pub fn to_vietnamese_uppercase(ch: char) -> char {
    match ch {
        'a' => 'A', 'á' => 'Á', 'à' => 'À', 'ả' => 'Ả', 'ã' => 'Ã', 'ạ' => 'Ạ',
        'â' => 'Â', 'ấ' => 'Ấ', 'ầ' => 'Ầ', 'ẩ' => 'Ẩ', 'ẫ' => 'Ẫ', 'ậ' => 'Ậ',
        'ă' => 'Ă', 'ắ' => 'Ắ', 'ằ' => 'Ằ', 'ẳ' => 'Ẳ', 'ẵ' => 'Ẵ', 'ặ' => 'Ặ',
        'e' => 'E', 'é' => 'É', 'è' => 'È', 'ẻ' => 'Ẻ', 'ẽ' => 'Ẽ', 'ẹ' => 'Ẹ',
        'ê' => 'Ê', 'ế' => 'Ế', 'ề' => 'Ề', 'ể' => 'Ể', 'ễ' => 'Ễ', 'ệ' => 'Ệ',
        'i' => 'I', 'í' => 'Í', 'ì' => 'Ì', 'ỉ' => 'Ỉ', 'ĩ' => 'Ĩ', 'ị' => 'Ị',
        'o' => 'O', 'ó' => 'Ó', 'ò' => 'Ò', 'ỏ' => 'Ỏ', 'õ' => 'Õ', 'ọ' => 'Ọ',
        'ô' => 'Ô', 'ố' => 'Ố', 'ồ' => 'Ồ', 'ổ' => 'Ổ', 'ỗ' => 'Ỗ', 'ộ' => 'Ộ',
        'ơ' => 'Ơ', 'ớ' => 'Ớ', 'ờ' => 'Ờ', 'ở' => 'Ở', 'ỡ' => 'Ỡ', 'ợ' => 'Ợ',
        'u' => 'U', 'ú' => 'Ú', 'ù' => 'Ù', 'ủ' => 'Ủ', 'ũ' => 'Ũ', 'ụ' => 'Ụ',
        'ư' => 'Ư', 'ứ' => 'Ứ', 'ừ' => 'Ừ', 'ử' => 'Ử', 'ữ' => 'Ữ', 'ự' => 'Ự',
        'y' => 'Y', 'ý' => 'Ý', 'ỳ' => 'Ỳ', 'ỷ' => 'Ỷ', 'ỹ' => 'Ỹ', 'ỵ' => 'Ỵ',
        'đ' => 'Đ',
        other => other.to_ascii_uppercase(),
    }
}

pub fn to_vietnamese_lowercase(ch: char) -> char {
    match ch {
        'A' => 'a', 'Á' => 'á', 'À' => 'à', 'Ả' => 'ả', 'Ã' => 'ã', 'Ạ' => 'ạ',
        'Â' => 'â', 'Ấ' => 'ấ', 'Ầ' => 'ầ', 'Ẩ' => 'ẩ', 'Ẫ' => 'ẫ', 'Ậ' => 'ậ',
        'Ă' => 'ă', 'Ắ' => 'ắ', 'Ằ' => 'ằ', 'Ẳ' => 'ẳ', 'Ẵ' => 'ẵ', 'Ặ' => 'ặ',
        'E' => 'e', 'É' => 'é', 'È' => 'è', 'Ẻ' => 'ẻ', 'Ẽ' => 'ẽ', 'Ẹ' => 'ẹ',
        'Ê' => 'ê', 'Ế' => 'ế', 'Ề' => 'ề', 'Ể' => 'ể', 'Ễ' => 'ễ', 'Ệ' => 'ệ',
        'I' => 'i', 'Í' => 'í', 'Ì' => 'ì', 'Ỉ' => 'ỉ', 'Ĩ' => 'ĩ', 'Ị' => 'ị',
        'O' => 'o', 'Ó' => 'ó', 'Ò' => 'ò', 'Ỏ' => 'ỏ', 'Õ' => 'õ', 'Ọ' => 'ọ',
        'Ô' => 'ô', 'Ố' => 'ố', 'Ồ' => 'ồ', 'Ổ' => 'ổ', 'Ỗ' => 'ỗ', 'Ộ' => 'ộ',
        'Ơ' => 'ơ', 'Ớ' => 'ớ', 'Ờ' => 'ờ', 'Ở' => 'ở', 'Ỡ' => 'ỡ', 'Ợ' => 'ợ',
        'U' => 'u', 'Ú' => 'ú', 'Ù' => 'ù', 'Ủ' => 'ủ', 'Ũ' => 'ũ', 'Ụ' => 'ụ',
        'Ư' => 'ư', 'Ứ' => 'ứ', 'Ừ' => 'ừ', 'Ử' => 'ử', 'Ữ' => 'ữ', 'Ự' => 'ự',
        'Y' => 'y', 'Ý' => 'ý', 'Ỳ' => 'ỳ', 'Ỷ' => 'ỷ', 'Ỹ' => 'ỹ', 'Ỵ' => 'ỵ',
        'Đ' => 'đ',
        other => other.to_ascii_lowercase(),
    }
}
