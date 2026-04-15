# Mulberry Music Theory Guide

This guide covers the music theory fundamentals you need to work with Mulberry, from basic concepts to synthesis and production techniques. Code examples use Mulberry's live-coding pattern syntax.

---

## Fundamentals

### Sound and Frequency

**Sound** is a vibration that propagates through a medium (usually air) as a pressure wave. When these pressure waves reach your ears, your brain interprets them as sound.

Key properties of a sound wave:

| Property | Description | Unit |
|----------|-------------|------|
| **Frequency** | Number of wave cycles per second | Hertz (Hz) |
| **Amplitude** | Height of the wave (loudness) | Decibels (dB) |
| **Wavelength** | Physical distance of one cycle | Meters (m) |
| **Phase** | Position in the wave cycle | Degrees (°) or radians |

**Frequency and Pitch:**
- Higher frequency = higher pitch
- Lower frequency = lower pitch
- Human hearing range: approximately 20 Hz – 20,000 Hz

**Standard Tuning:**
- **A4 = 440 Hz** is the international standard tuning reference
- Every note's frequency can be calculated from this reference

**Octave Relationship:**
- An **octave** is a doubling (or halving) of frequency
- A3 = 220 Hz, A4 = 440 Hz, A5 = 880 Hz
- Notes an octave apart sound "the same" but higher or lower

```
-- Play A in three octaves to hear the relationship
d1 $ "a3 a4 a5" | sound "sine"
```

### Musical Notes

Western music divides the octave into **12 semitones** (half steps). The seven natural note names are:

```
C  D  E  F  G  A  B
```

The full **chromatic scale** includes sharps (#) and flats (b):

```
C  C#  D  D#  E  F  F#  G  G#  A  A#  B
     Db      Eb          Gb      Ab      Bb
```

**Enharmonic equivalents:** C# and Db are the same pitch, just named differently depending on context.

**MIDI Note Numbers:**

MIDI assigns an integer (0–127) to every note. This is how digital audio systems represent pitch.

| Note | MIDI Number | Frequency (Hz) |
|------|-------------|-----------------|
| C3 | 48 | 130.81 |
| C4 (Middle C) | 60 | 261.63 |
| A4 | 69 | 440.00 |
| C5 | 72 | 523.25 |

**Formula:** `frequency = 440 × 2^((midi_note - 69) / 12)`

In Mulberry, you can use either note names or MIDI numbers:

```
-- These are equivalent
d1 $ "c4 e4 g4" | sound "piano"
d1 $ "60 64 67" | sound "piano"
```

### Scales

A **scale** is a set of notes arranged in ascending order by pitch. Scales define the "palette" of notes for a piece of music.

#### Major Scale

The major scale follows the pattern of **Whole** and **Half** steps:

```
W  W  H  W  W  W  H
```

**C Major Scale** (no sharps or flats):

```
C  D  E  F  G  A  B  C
 W  W  H  W  W  W  H
```

```
-- C Major scale ascending
d1 $ "c4 d4 e4 f4 g4 a4 b4 c5" | sound "piano" | slow 2
```

#### Minor Scales

**Natural Minor** — the "sad" scale. Formula: `W H W W H W W`

```
-- A Natural Minor (same notes as C Major, starting on A)
d1 $ "a3 b3 c4 d4 e4 f4 g4 a4" | sound "piano" | slow 2
```

**Harmonic Minor** — raised 7th degree for a stronger pull to the tonic:

```
-- A Harmonic Minor
d1 $ "a3 b3 c4 d4 e4 f4 g#4 a4" | sound "piano" | slow 2
```

**Melodic Minor** — raised 6th and 7th ascending, natural minor descending:

```
-- A Melodic Minor (ascending form)
d1 $ "a3 b3 c4 d4 e4 f#4 g#4 a4" | sound "piano" | slow 2
```

#### Pentatonic Scale

A five-note scale that sounds good in almost any context. It's the major scale without the 4th and 7th degrees.

**C Major Pentatonic:** C D E G A

```
-- Pentatonic melody
d1 $ "c4 d4 e4 g4 a4 c5 a4 g4" | sound "piano"
```

**A Minor Pentatonic:** A C D E G (the blues scale foundation)

```
-- Minor pentatonic riff
d1 $ "a3 c4 d4 e4 g4 e4 d4 c4" | sound "guitar"
```

### Intervals

An **interval** is the distance between two notes, measured in semitones.

| Interval | Semitones | Example (from C) | Sound Quality |
|----------|-----------|-------------------|---------------|
| Unison | 0 | C → C | Same note |
| Minor 2nd | 1 | C → Db | Tense, dissonant |
| Major 2nd | 2 | C → D | Bright step |
| Minor 3rd | 3 | C → Eb | Sad, dark |
| Major 3rd | 4 | C → E | Happy, bright |
| Perfect 4th | 5 | C → F | Open, neutral |
| Tritone | 6 | C → F# | Very tense |
| Perfect 5th | 7 | C → G | Strong, stable |
| Minor 6th | 8 | C → Ab | Bittersweet |
| Major 6th | 9 | C → A | Warm |
| Minor 7th | 10 | C → Bb | Bluesy |
| Major 7th | 11 | C → B | Dreamy, tense |
| Octave | 12 | C → C | Same note, higher |

**Perfect intervals** (unison, 4th, 5th, octave) are called "perfect" because they sound especially consonant and stable.

```
-- Listen to intervals from C4
d1 $ "c4 e4"  | sound "sine"   -- major 3rd (happy)
d2 $ "c4 eb4" | sound "sine"   -- minor 3rd (sad)
d3 $ "c4 g4"  | sound "sine"   -- perfect 5th (strong)
```

### Chords

A **chord** is three or more notes played simultaneously. Chords are built by stacking intervals.

#### Major Triads

Built from: **root + major 3rd + perfect 5th** (0, 4, 7 semitones)

```
C Major:  C  E  G
F Major:  F  A  C
G Major:  G  B  D
```

```
-- C Major chord
d1 $ < "c4" , "e4" , "g4" > | sound "piano"
```

#### Minor Triads

Built from: **root + minor 3rd + perfect 5th** (0, 3, 7 semitones)

```
A Minor:  A  C  E
D Minor:  D  F  A
E Minor:  E  G  B
```

```
-- A Minor chord
d1 $ < "a3" , "c4" , "e4" > | sound "piano"
```

#### 7th Chords

Add a 7th on top of a triad for richer harmony:

| Type | Formula | Example |
|------|---------|---------|
| Major 7th | 0, 4, 7, 11 | Cmaj7: C E G B |
| Dominant 7th | 0, 4, 7, 10 | G7: G B D F |
| Minor 7th | 0, 3, 7, 10 | Am7: A C E G |
| Diminished 7th | 0, 3, 6, 9 | Bdim7: B D F Ab |

```
-- Cmaj7 chord
d1 $ < "c4" , "e4" , "g4" , "b4" > | sound "piano"

-- G7 chord
d2 $ < "g3" , "b3" , "d4" , "f4" > | sound "piano"
```

#### Chord Progressions

Common progressions using Roman numeral notation (relative to the key):

**I – IV – V – I** (the most fundamental progression):
```
-- In C Major: C → F → G → C
d1 $ < "c4 f4 g4 c4" , "e4 a4 b4 e4" , "g4 c5 d5 g4" > | sound "piano" | slow 4
```

**ii – V – I** (jazz standard):
```
-- In C Major: Dm → G → C
d1 $ < "d4 g3 c4" , "f4 b3 e4" , "a4 d4 g4" > | sound "piano" | slow 3
```

**I – V – vi – IV** (pop progression):
```
-- In C Major: C → G → Am → F
d1 $ < "c4 g4 a4 f4" , "e4 b4 c5 a4" , "g4 d5 e5 c5" > | sound "piano" | slow 4
```

### Rhythm

#### Time Signatures

A **time signature** tells you how many beats are in a bar and which note value gets one beat.

| Time Signature | Meaning | Feel |
|----------------|---------|------|
| **4/4** | 4 quarter-note beats per bar | "Common time" — most popular |
| **3/4** | 3 quarter-note beats per bar | Waltz feel |
| **6/8** | 6 eighth-note beats per bar | Compound duple — swinging feel |

#### Note Durations

In 4/4 time:

| Duration | Beats | Name |
|----------|-------|------|
| Whole note | 4 | Semibreve |
| Half note | 2 | Minim |
| Quarter note | 1 | Crotchet |
| Eighth note | ½ | Quaver |
| Sixteenth note | ¼ | Semiquaver |

#### BPM (Beats Per Minute)

BPM defines tempo — how fast the music plays.

| BPM Range | Feel |
|-----------|------|
| 60–80 | Slow (ballads, ambient) |
| 80–110 | Moderate (hip-hop, R&B) |
| 110–130 | Upbeat (pop, house) |
| 130–150 | Fast (techno, drum & bass) |
| 150+ | Very fast (jungle, speedcore) |

#### Subdivisions

Dividing beats into smaller parts creates rhythmic complexity:

```
-- Quarter notes (4 per cycle)
d1 $ "bd bd bd bd" | sound "drums"

-- Eighth notes using subdivision (8 per cycle)
d1 $ "[bd bd] [bd bd] [bd bd] [bd bd]" | sound "drums"

-- Sixteenth notes
d1 $ "bd*4 bd*4 bd*4 bd*4" | sound "drums"

-- Mixed subdivisions for a realistic beat
d1 $ "bd [~ hh] sd [hh hh]" | sound "drums"
```

---

## Synthesis

### Waveforms

Synthesizers generate sound from basic **waveforms**, each with a distinct tonal character determined by its harmonic content.

#### Sine Wave

- **Harmonics:** Fundamental frequency only (no overtones)
- **Sound:** Pure, clean, flute-like
- **Use:** Sub-bass, pure tones, FM synthesis carrier

```
d1 $ "c4 e4 g4 c5" | sound "sine"
```

#### Square Wave

- **Harmonics:** Odd harmonics only (1st, 3rd, 5th, 7th…)
- **Sound:** Hollow, clarinet-like, retro/chiptune
- **Use:** Leads, bass, 8-bit sounds

```
d1 $ "c4 e4 g4 c5" | sound "square"
```

#### Sawtooth Wave

- **Harmonics:** All harmonics (1st, 2nd, 3rd, 4th…), each at 1/n amplitude
- **Sound:** Bright, brassy, rich
- **Use:** Pads, leads, brass sounds, supersaw

```
d1 $ "c4 e4 g4 c5" | sound "saw"
```

#### Triangle Wave

- **Harmonics:** Odd harmonics only, but amplitude falls off as 1/n²
- **Sound:** Mellow, softer than square, woody
- **Use:** Bass, mellow leads, sub oscillator

```
d1 $ "c4 e4 g4 c5" | sound "triangle"
```

#### Comparison

```
-- Compare all four waveforms playing the same note
d1 $ "c4" | sound "sine"
d2 $ "c4" | sound "square"
d3 $ "c4" | sound "saw"
d4 $ "c4" | sound "triangle"
```

### ADSR Envelope

An **envelope** shapes how a sound evolves over time. The most common type is **ADSR**:

```
Amplitude
    │
    │   /\
    │  / │\___________
    │ /  │            \
    │/   │             \
    └────┴──────────────┴───── Time
    A    D    S          R
```

| Stage | Description | Typical Values |
|-------|-------------|----------------|
| **Attack** | Time from silence to peak amplitude | 0–500 ms |
| **Decay** | Time from peak to sustain level | 0–1000 ms |
| **Sustain** | Steady-state amplitude level (not a time!) | 0.0–1.0 |
| **Release** | Time from note-off to silence | 0–5000 ms |

**Sound design examples:**

| Sound | Attack | Decay | Sustain | Release |
|-------|--------|-------|---------|---------|
| Pluck/stab | 0 ms | 100 ms | 0.0 | 50 ms |
| Piano | 5 ms | 500 ms | 0.3 | 300 ms |
| Pad | 500 ms | 200 ms | 0.8 | 2000 ms |
| Organ | 10 ms | 0 ms | 1.0 | 10 ms |

```
-- Short, plucky sound
d1 $ "c4 e4 g4 c5" | sound "saw" | attack 0 | decay 0.1 | sustain 0 | release 0.05

-- Long, evolving pad
d1 $ "c4 e4 g4 c5" | sound "saw" | attack 0.5 | decay 0.2 | sustain 0.8 | release 2.0 | slow 4
```

### Filters

Filters shape the **frequency content** (timbre) of a sound by boosting or attenuating certain frequencies.

#### Low-Pass Filter (LPF)

Passes frequencies **below** the cutoff, removes high frequencies. This is the most commonly used filter in synthesis.

- **Effect:** Darkens/warms the sound
- **Use:** Removing harshness, creating "underwater" effects, filter sweeps

```
d1 $ "c4 e4 g4 c5" | sound "saw" | lpf 800
```

#### High-Pass Filter (HPF)

Passes frequencies **above** the cutoff, removes low frequencies.

- **Effect:** Thins out the sound, removes muddiness
- **Use:** Clearing low-end from non-bass instruments, creating "tinny" effects

```
d1 $ "c4 e4 g4 c5" | sound "saw" | hpf 2000
```

#### Band-Pass Filter (BPF)

Passes only a **band** of frequencies around the center frequency.

- **Effect:** Isolates a frequency range, like a "wah" or vocal formant
- **Use:** Wah effects, radio/telephone simulation

```
d1 $ "c4 e4 g4 c5" | sound "saw" | bpf 1000
```

#### Notch Filter (Band-Reject)

Removes a narrow **band** of frequencies, passes everything else.

- **Effect:** Scoops out a frequency range
- **Use:** Removing resonances, phaser effects

#### Cutoff and Resonance

Every filter has two key parameters:

- **Cutoff Frequency:** The frequency at which the filter starts working (in Hz)
- **Resonance (Q):** Boosts frequencies near the cutoff, creating a peak. High resonance creates a ringing, whistling quality. At extreme values, the filter self-oscillates.

```
-- Filter sweep: cutoff moves from 200 Hz to 8000 Hz
d1 $ "c3*8" | sound "saw" | lpf (range 200 8000 $ slow 4 sine)
```

---

## Production

### Signal Flow

Audio signals follow a path from creation to output:

```
Source (oscillator/sample)
    │
    ▼
Insert Effects (per-track)
    │  EQ → Compressor → Saturation
    ▼
Channel Strip
    │  Volume → Pan
    ▼
Send Effects (shared)
    │  Reverb, Delay (via aux bus)
    ▼
Mixer (summing all tracks)
    │
    ▼
Master Bus
    │  Master EQ → Limiter
    ▼
Output (DAC → speakers/headphones)
```

**Insert vs Send Effects:**

| Type | Description | Use Case |
|------|-------------|----------|
| **Insert** | Processes the entire signal on a single track | EQ, compression, distortion |
| **Send** | Routes a copy of the signal to a shared effect bus | Reverb, delay (shared by many tracks) |

**Gain Staging:** Keep levels consistent through the signal chain. Aim for peaks around -6 dB on individual tracks to leave headroom for mixing.

### Mixing

#### Volume Balancing

The foundation of a good mix. Start with all faders down and bring up one element at a time:

1. **Drums/rhythm** — the foundation
2. **Bass** — lock in with the kick
3. **Harmony** (chords, pads) — fill out the mid-range
4. **Melody/lead** — sits on top
5. **Effects/texture** — add space and interest

#### Panning (Stereo Placement)

Distribute elements across the stereo field:

| Element | Typical Position |
|---------|-----------------|
| Kick, bass, lead vocal | Center |
| Snare | Center or slightly off |
| Hi-hats | Slightly left or right |
| Guitars, keys | Left/right pair |
| Pads, strings | Wide stereo |
| Effects | Various |

```
d1 $ "bd ~ sd ~" | sound "drums" | pan 0.5       -- center
d2 $ "hh*8" | sound "drums" | pan 0.3             -- left of center
d3 $ "c4 e4 g4 c5" | sound "piano" | pan 0.7      -- right of center
```

#### EQ (Equalization)

Shape the frequency balance of each track:

| Frequency Range | Name | Character |
|----------------|------|-----------|
| 20–60 Hz | Sub-bass | Felt more than heard |
| 60–250 Hz | Bass | Warmth, body |
| 250–2000 Hz | Midrange | Body, presence |
| 2000–6000 Hz | Upper mids | Presence, clarity |
| 6000–20000 Hz | Highs | Air, brilliance |

**Tips:**
- Cut before you boost (removing problem frequencies is cleaner than adding good ones)
- Use high-pass filters on everything except bass and kick to clear mud
- Small, narrow cuts for problem frequencies; broad, gentle boosts for character

#### Compression

Reduces the **dynamic range** (difference between loud and quiet) of a signal.

| Parameter | Description |
|-----------|-------------|
| **Threshold** | Level above which compression kicks in |
| **Ratio** | How much to reduce (4:1 means 4 dB over threshold → 1 dB output) |
| **Attack** | How fast the compressor reacts |
| **Release** | How fast it stops compressing |
| **Makeup Gain** | Boost to compensate for volume reduction |

**Use cases:**
- Taming vocal dynamics
- Adding punch to drums (slow attack lets transient through)
- Gluing a mix together (bus compression)

#### Reverb and Delay

**Reverb** simulates acoustic spaces:

| Type | Sound |
|------|-------|
| Room | Small, tight, natural |
| Hall | Large, spacious, orchestral |
| Plate | Bright, dense, vintage |
| Spring | Metallic, twangy, surf rock |

**Delay** creates echoes:

| Type | Sound |
|------|-------|
| Slapback | Single short echo (rockabilly) |
| Ping-pong | Alternates left and right |
| Tape | Warm, degrading repeats |
| Dotted eighth | "The Edge" guitar sound |

```
-- Add reverb and delay
d1 $ "c4 e4 g4 c5" | sound "piano" | room 0.8 | delay 0.3 | delaytime 0.375
```

### Arrangement

#### Tracks and Regions

A **track** is a horizontal lane containing audio or pattern data. A **region** (or clip) is a block of content placed on a track's timeline.

```
-- Define multiple tracks for a full arrangement
d1 $ "bd ~ sd ~" | sound "drums"                   -- drums
d2 $ "c3 ~ g3 ~" | sound "bass"                    -- bass
d3 $ "c4 e4 g4 c5" | sound "piano" | slow 2        -- harmony
d4 $ "~ ~ c5 ~" | sound "lead"                     -- melody
```

#### Song Structure

Common sections in popular music:

| Section | Purpose | Typical Length |
|---------|---------|----------------|
| **Intro** | Sets the mood, draws listener in | 4–8 bars |
| **Verse** | Tells the story, lower energy | 8–16 bars |
| **Chorus** | Main hook, highest energy | 8–16 bars |
| **Bridge** | Contrast section, breaks repetition | 4–8 bars |
| **Outro** | Winds down, closes the song | 4–8 bars |

A common pop structure:

```
Intro → Verse → Chorus → Verse → Chorus → Bridge → Chorus → Outro
```

Electronic music often uses a build-based structure:

```
Intro → Buildup → Drop → Breakdown → Buildup → Drop → Outro
```

#### Automation

**Automation** is the recording of parameter changes over time. Instead of a static value, a parameter follows a curve.

Common automation targets:
- Volume (fade-ins, fade-outs, swells)
- Filter cutoff (filter sweeps)
- Pan (movement across stereo field)
- Effect sends (reverb builds)
- BPM (tempo changes)

```
-- Simulating a filter sweep with pattern transforms
d1 $ "c3*8" | sound "saw" | lpf (range 200 8000 $ slow 8 sine)

-- Volume swell
d1 $ "c4 e4 g4 c5" | sound "pad" | gain (range 0 1 $ slow 4 sine)
```

---

## Quick Reference

### Note-to-Frequency Table

| Octave | C | C# | D | D# | E | F | F# | G | G# | A | A# | B |
|--------|---|----|---|----|---|---|----|---|----|---|----|---|
| 2 | 65 | 69 | 73 | 78 | 82 | 87 | 93 | 98 | 104 | 110 | 117 | 123 |
| 3 | 131 | 139 | 147 | 156 | 165 | 175 | 185 | 196 | 208 | 220 | 233 | 247 |
| 4 | 262 | 277 | 294 | 311 | 330 | 349 | 370 | 392 | 415 | 440 | 466 | 494 |
| 5 | 523 | 554 | 587 | 622 | 659 | 698 | 740 | 784 | 831 | 880 | 932 | 988 |

*(Frequencies rounded to nearest integer, in Hz)*

### Common Scales (from C)

| Scale | Notes |
|-------|-------|
| C Major | C D E F G A B |
| C Natural Minor | C D Eb F G Ab Bb |
| C Harmonic Minor | C D Eb F G Ab B |
| C Major Pentatonic | C D E G A |
| C Minor Pentatonic | C Eb F G Bb |
| C Blues | C Eb F F# G Bb |
| C Dorian | C D Eb F G A Bb |
| C Mixolydian | C D E F G A Bb |

### Common Chord Types

| Type | Formula (semitones) | Example from C |
|------|-------------------|----------------|
| Major | 0, 4, 7 | C E G |
| Minor | 0, 3, 7 | C Eb G |
| Diminished | 0, 3, 6 | C Eb Gb |
| Augmented | 0, 4, 8 | C E G# |
| Major 7th | 0, 4, 7, 11 | C E G B |
| Minor 7th | 0, 3, 7, 10 | C Eb G Bb |
| Dominant 7th | 0, 4, 7, 10 | C E G Bb |
| Suspended 2nd | 0, 2, 7 | C D G |
| Suspended 4th | 0, 5, 7 | C F G |
