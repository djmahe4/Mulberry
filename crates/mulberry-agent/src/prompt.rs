/// Pre-built prompt templates for different agent roles.
pub struct PromptTemplates;

impl PromptTemplates {
    /// System prompt for a composer agent.
    pub fn composer_system_prompt() -> String {
        "You are an AI music composer assistant for the Mulberry DAW. \
         You specialize in generating musical patterns, melodies, chord progressions, \
         and arrangements. When asked, output MIDI-compatible note data as JSON arrays \
         with fields: note (MIDI number 0-127), velocity (0.0-1.0), start (beats), \
         and duration (beats). Always respect the requested key and scale."
            .into()
    }

    /// System prompt for a mix engineer agent.
    pub fn mix_engineer_system_prompt() -> String {
        "You are an AI mix engineer assistant for the Mulberry DAW. \
         You provide advice on EQ, compression, reverb, panning, and gain staging. \
         When suggesting parameter changes, output them as JSON objects with fields: \
         target (track or plugin name), param (parameter name), and value (numeric). \
         Always explain the reasoning behind your suggestions."
            .into()
    }

    /// System prompt for a live-coding agent.
    pub fn live_coder_system_prompt() -> String {
        "You are an AI live-coding assistant for the Mulberry DAW. \
         You generate rhythmic and melodic patterns using the Mulberry live-coding \
         syntax. Output valid pattern expressions that can be evaluated directly. \
         Be creative with polyrhythms, euclidean rhythms, and generative techniques."
            .into()
    }

    /// System prompt for a music theory tutor agent.
    pub fn tutor_system_prompt() -> String {
        "You are a music theory tutor integrated into the Mulberry DAW. \
         Explain concepts clearly with practical examples. When relevant, \
         reference MIDI note numbers and standard music notation. Cover topics \
         such as scales, modes, chord construction, voice leading, counterpoint, \
         and harmonic analysis. Keep explanations concise but accurate."
            .into()
    }

    /// User prompt requesting the generation of a musical pattern.
    pub fn pattern_generation_prompt(key: &str, scale: &str, bars: u32) -> String {
        format!(
            "Generate a {bars}-bar melodic pattern in the key of {key} {scale}. \
             Output the result as a JSON array of note events with fields: \
             note (MIDI number), velocity (0.0-1.0), start (in beats), \
             and duration (in beats). Assume 4/4 time."
        )
    }

    /// User prompt requesting mixing advice.
    pub fn mixing_advice_prompt(description: &str) -> String {
        format!(
            "I have the following mix situation: {description}\n\n\
             Please suggest specific parameter adjustments to improve the mix. \
             Output your suggestions as a JSON array of objects with fields: \
             target, param, value, and a brief reason for each change."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompts_are_non_empty() {
        assert!(!PromptTemplates::composer_system_prompt().is_empty());
        assert!(!PromptTemplates::mix_engineer_system_prompt().is_empty());
        assert!(!PromptTemplates::live_coder_system_prompt().is_empty());
        assert!(!PromptTemplates::tutor_system_prompt().is_empty());
    }

    #[test]
    fn pattern_prompt_contains_parameters() {
        let prompt = PromptTemplates::pattern_generation_prompt("C", "minor", 4);
        assert!(prompt.contains("C"));
        assert!(prompt.contains("minor"));
        assert!(prompt.contains("4-bar"));
    }

    #[test]
    fn mixing_prompt_contains_description() {
        let prompt = PromptTemplates::mixing_advice_prompt("muddy low end");
        assert!(prompt.contains("muddy low end"));
    }
}
