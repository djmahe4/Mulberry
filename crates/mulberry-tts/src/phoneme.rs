/// Basic phoneme types for formant synthesis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phoneme {
    // Vowels (with formant frequencies in Hz: F1, F2)
    /// "ah" as in "father"
    A,
    /// "eh" as in "bed"
    E,
    /// "ee" as in "see"
    I,
    /// "oh" as in "go"
    O,
    /// "oo" as in "blue"
    U,
    /// Silence / pause
    Silence,
}

impl Phoneme {
    /// Get formant frequencies (F1, F2) for this phoneme.
    pub fn formants(&self) -> (f32, f32) {
        match self {
            Phoneme::A => (730.0, 1090.0),
            Phoneme::E => (530.0, 1840.0),
            Phoneme::I => (270.0, 2290.0),
            Phoneme::O => (570.0, 840.0),
            Phoneme::U => (300.0, 870.0),
            Phoneme::Silence => (0.0, 0.0),
        }
    }
}

/// Convert text to a simple phoneme sequence.
///
/// This is a very basic approximation — a real TTS system would use a proper
/// grapheme-to-phoneme (G2P) model.
pub fn text_to_phonemes(text: &str) -> Vec<Phoneme> {
    let mut phonemes = Vec::new();
    for ch in text.to_lowercase().chars() {
        match ch {
            'a' => phonemes.push(Phoneme::A),
            'e' => phonemes.push(Phoneme::E),
            'i' => phonemes.push(Phoneme::I),
            'o' => phonemes.push(Phoneme::O),
            'u' => phonemes.push(Phoneme::U),
            ' ' => phonemes.push(Phoneme::Silence),
            _ => {} // Skip consonants in this basic version
        }
    }
    if phonemes.is_empty() {
        phonemes.push(Phoneme::Silence);
    }
    phonemes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_to_phonemes_vowels() {
        let phonemes = text_to_phonemes("aeiou");
        assert_eq!(
            phonemes,
            vec![Phoneme::A, Phoneme::E, Phoneme::I, Phoneme::O, Phoneme::U]
        );
    }

    #[test]
    fn test_text_to_phonemes_with_spaces() {
        let phonemes = text_to_phonemes("a e");
        assert_eq!(phonemes, vec![Phoneme::A, Phoneme::Silence, Phoneme::E]);
    }

    #[test]
    fn test_text_to_phonemes_empty() {
        let phonemes = text_to_phonemes("");
        assert_eq!(phonemes, vec![Phoneme::Silence]);
    }

    #[test]
    fn test_text_to_phonemes_consonants_only() {
        let phonemes = text_to_phonemes("bcd");
        assert_eq!(phonemes, vec![Phoneme::Silence]);
    }

    #[test]
    fn test_formants() {
        let (f1, f2) = Phoneme::A.formants();
        assert!(f1 > 0.0);
        assert!(f2 > f1);

        let (f1, f2) = Phoneme::Silence.formants();
        assert_eq!(f1, 0.0);
        assert_eq!(f2, 0.0);
    }
}
