"""
Mulberry Web-Music IDE
A browser-based music interaction environment using Web Audio API for real-time synthesis.

This Streamlit application provides:
- Real-time tone generation with oscillator nodes
- ADSR envelope control (Attack, Decay, Sustain, Release)
- Multiple waveform types (sine, square, sawtooth, triangle)
- C-major scale demo
- Interactive UI controls for frequency and waveform selection
"""

import streamlit as st
import streamlit.components.v1 as components

# Page configuration
st.set_page_config(
    page_title="Mulberry Web-Music IDE",
    page_icon="🎵",
    layout="wide"
)

# Title and description
st.title("🎵 Mulberry Web-Music IDE")
st.markdown("""
Welcome to the Mulberry Web-Music IDE! This application uses the Web Audio API 
to generate sounds in real-time. No pre-recorded audio files—everything is 
synthesized on the fly in your browser.
""")

# Sidebar controls
st.sidebar.header("Audio Controls")
st.sidebar.markdown("### Tone Generator Settings")

# Frequency slider (200-800 Hz, default 440 Hz - A4 note)
frequency = st.sidebar.slider(
    "Frequency (Hz)",
    min_value=200,
    max_value=800,
    value=440,
    step=1,
    help="Adjust the frequency of the tone. 440 Hz is the standard A4 note."
)

# Waveform selection dropdown
waveform = st.sidebar.selectbox(
    "Waveform Type",
    options=["sine", "square", "sawtooth", "triangle"],
    index=0,
    help="Select the waveform type for the oscillator."
)

# ADSR envelope controls
st.sidebar.markdown("### ADSR Envelope")
st.sidebar.markdown("Control the amplitude envelope of the sound:")

attack = st.sidebar.slider(
    "Attack (s)",
    min_value=0.01,
    max_value=2.0,
    value=0.1,
    step=0.01,
    help="Time for the sound to reach full volume"
)

decay = st.sidebar.slider(
    "Decay (s)",
    min_value=0.01,
    max_value=2.0,
    value=0.2,
    step=0.01,
    help="Time for the sound to drop to sustain level"
)

sustain = st.sidebar.slider(
    "Sustain Level",
    min_value=0.0,
    max_value=1.0,
    value=0.5,
    step=0.01,
    help="The level at which the sound holds during the note"
)

release = st.sidebar.slider(
    "Release (s)",
    min_value=0.01,
    max_value=3.0,
    value=0.3,
    step=0.01,
    help="Time for the sound to fade out after note release"
)

# Main content area
col1, col2 = st.columns([2, 1])

with col1:
    st.markdown("### Sound Generation")
    st.markdown("""
    Use the controls below to generate tones. The Web Audio API creates sounds 
    in real-time using your browser's audio capabilities.
    """)

with col2:
    st.markdown("### Quick Info")
    st.info(f"""
    **Current Settings:**
    - Frequency: {frequency} Hz
    - Waveform: {waveform}
    - Attack: {attack}s
    - Decay: {decay}s
    - Sustain: {sustain}
    - Release: {release}s
    """)

# Web Audio implementation using HTML/JavaScript
# This component will handle all the audio processing
audio_component_html = f"""
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: transparent;
        }}
        .controls {{
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
            flex-wrap: wrap;
        }}
        button {{
            padding: 12px 24px;
            font-size: 16px;
            cursor: pointer;
            border: none;
            border-radius: 8px;
            transition: all 0.3s ease;
            font-weight: 600;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .play-tone {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }}
        .play-tone:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
        }}
        .play-tone:active {{
            transform: translateY(0);
        }}
        .play-scale {{
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            color: white;
        }}
        .play-scale:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(245, 87, 108, 0.4);
        }}
        .play-scale:active {{
            transform: translateY(0);
        }}
        button:disabled {{
            opacity: 0.5;
            cursor: not-allowed;
            transform: none !important;
        }}
        .info {{
            margin-top: 20px;
            padding: 15px;
            background-color: #f0f4ff;
            border-left: 4px solid #667eea;
            border-radius: 4px;
        }}
        .info h3 {{
            margin-top: 0;
            color: #667eea;
        }}
        .status {{
            margin-top: 10px;
            padding: 10px;
            background-color: #e8f5e9;
            border-radius: 4px;
            font-family: monospace;
            display: none;
        }}
        .status.active {{
            display: block;
        }}
    </style>
</head>
<body>
    <div class="controls">
        <button class="play-tone" onclick="playTone()">▶️ Play Tone</button>
        <button class="play-scale" onclick="playCMajorScale()">🎼 Play C-Major Scale</button>
    </div>
    
    <div class="status" id="status"></div>
    
    <div class="info">
        <h3>🎵 How the Web Audio API Works</h3>
        <p><strong>Audio Context:</strong> The browser's audio processing engine that manages all sound generation.</p>
        <p><strong>Oscillator Node:</strong> Generates the raw waveform at a specific frequency.</p>
        <p><strong>Gain Node:</strong> Controls volume and implements the ADSR envelope.</p>
        <p><strong>Audio Graph:</strong> Oscillator → Gain → Destination (speakers)</p>
        <br>
        <p><strong>ADSR Envelope:</strong></p>
        <ul>
            <li><strong>Attack:</strong> How quickly the sound reaches full volume</li>
            <li><strong>Decay:</strong> How quickly it drops to sustain level</li>
            <li><strong>Sustain:</strong> The steady volume level while held</li>
            <li><strong>Release:</strong> How quickly it fades to silence after release</li>
        </ul>
    </div>

    <script>
        // Global audio context - created once and reused
        // The AudioContext is the foundation of Web Audio API
        let audioContext = null;

        // Parameters from Streamlit (passed via template)
        const frequency = {frequency};
        const waveform = "{waveform}";
        const attack = {attack};
        const decay = {decay};
        const sustain = {sustain};
        const release = {release};

        /**
         * Initialize Audio Context
         * Must be called after a user gesture (button click) to comply with browser autoplay policies
         */
        function initAudioContext() {{
            if (!audioContext) {{
                // Create the AudioContext - this is our audio processing engine
                audioContext = new (window.AudioContext || window.webkitAudioContext)();
                showStatus('Audio context initialized');
            }}
            // Resume if suspended (browser autoplay policy)
            if (audioContext.state === 'suspended') {{
                audioContext.resume();
            }}
        }}

        /**
         * Play a single tone with ADSR envelope
         * This demonstrates the core Web Audio API pattern:
         * 1. Create oscillator (sound source)
         * 2. Create gain node (volume control)
         * 3. Connect nodes: oscillator → gain → destination
         * 4. Apply ADSR envelope to gain
         * 5. Start and stop oscillator
         */
        function playTone() {{
            // Initialize audio context on first user interaction
            initAudioContext();
            
            // Get current time from audio context for precise timing
            const now = audioContext.currentTime;
            
            // Create oscillator node - this generates the actual sound wave
            const oscillator = audioContext.createOscillator();
            oscillator.type = waveform; // Set waveform type (sine, square, etc.)
            oscillator.frequency.setValueAtTime(frequency, now); // Set frequency in Hz
            
            // Create gain node - controls volume (amplitude)
            const gainNode = audioContext.createGain();
            
            // Connect the audio graph: oscillator → gain → destination (speakers)
            oscillator.connect(gainNode);
            gainNode.connect(audioContext.destination);
            
            // Calculate total note duration
            const noteDuration = attack + decay + 0.5; // Hold for 0.5s at sustain level
            
            // Apply ADSR envelope to gain node
            // This creates the volume shape over time
            
            // Start at 0 (silence)
            gainNode.gain.setValueAtTime(0, now);
            
            // ATTACK: Ramp up to full volume (1.0)
            gainNode.gain.linearRampToValueAtTime(1.0, now + attack);
            
            // DECAY: Ramp down to sustain level
            gainNode.gain.linearRampToValueAtTime(sustain, now + attack + decay);
            
            // SUSTAIN: Hold at sustain level (implicit - just wait)
            // The sustain value is maintained automatically
            
            // RELEASE: Ramp down to silence after note duration
            gainNode.gain.linearRampToValueAtTime(sustain, now + noteDuration);
            gainNode.gain.linearRampToValueAtTime(0, now + noteDuration + release);
            
            // Start the oscillator
            oscillator.start(now);
            
            // Stop the oscillator after release phase completes
            oscillator.stop(now + noteDuration + release);
            
            // Show status
            showStatus(`Playing {{frequency}}Hz {{waveform}} tone with ADSR envelope`);
            
            // Hide status after sound completes
            setTimeout(() => {{
                hideStatus();
            }}, (noteDuration + release) * 1000 + 500);
        }}

        /**
         * Play C-Major scale (C, D, E, F, G, A, B, C)
         * Demonstrates sequencing multiple tones
         * Frequencies are based on equal temperament tuning starting from C4 (261.63 Hz)
         */
        function playCMajorScale() {{
            initAudioContext();
            
            // C-Major scale frequencies (C4 to C5)
            // These are calculated using the 12-tone equal temperament formula:
            // f(n) = f0 * 2^(n/12), where f0 is a reference frequency
            const cMajorScale = [
                261.63, // C4
                293.66, // D4
                329.63, // E4
                349.23, // F4
                392.00, // G4
                440.00, // A4
                493.88, // B4
                523.25  // C5 (octave)
            ];
            
            const noteNames = ['C4', 'D4', 'E4', 'F4', 'G4', 'A4', 'B4', 'C5'];
            const noteDuration = 0.3; // Duration of each note in seconds
            
            showStatus('Playing C-Major scale...');
            
            // Schedule each note in the scale
            cMajorScale.forEach((freq, index) => {{
                // Use setTimeout to schedule each note
                // We add a small delay (index * noteDuration * 1000) for each note
                setTimeout(() => {{
                    playNoteInScale(freq, noteDuration, noteNames[index]);
                }}, index * noteDuration * 1000);
            }});
            
            // Hide status after scale completes
            const totalDuration = cMajorScale.length * noteDuration;
            setTimeout(() => {{
                hideStatus();
            }}, totalDuration * 1000 + 500);
        }}

        /**
         * Play a single note in a scale
         * Similar to playTone but with fixed duration and simpler envelope
         */
        function playNoteInScale(freq, duration, noteName) {{
            const now = audioContext.currentTime;
            
            // Create oscillator with the specified frequency
            const oscillator = audioContext.createOscillator();
            oscillator.type = waveform;
            oscillator.frequency.setValueAtTime(freq, now);
            
            // Create gain node with simplified envelope for scale
            const gainNode = audioContext.createGain();
            oscillator.connect(gainNode);
            gainNode.connect(audioContext.destination);
            
            // Simple envelope for scale notes
            const quickAttack = 0.01; // Quick attack for crisp notes
            const quickRelease = 0.05; // Quick release
            
            gainNode.gain.setValueAtTime(0, now);
            gainNode.gain.linearRampToValueAtTime(0.5, now + quickAttack);
            gainNode.gain.linearRampToValueAtTime(0.5, now + duration - quickRelease);
            gainNode.gain.linearRampToValueAtTime(0, now + duration);
            
            oscillator.start(now);
            oscillator.stop(now + duration);
            
            // Update status with current note
            updateStatus(`Playing {{noteName}} ({{freq.toFixed(2)}}Hz)`);
        }}

        /**
         * Utility functions for status display
         */
        function showStatus(message) {{
            const status = document.getElementById('status');
            status.textContent = message;
            status.classList.add('active');
        }}

        function updateStatus(message) {{
            const status = document.getElementById('status');
            if (status.classList.contains('active')) {{
                status.textContent = message;
            }}
        }}

        function hideStatus() {{
            const status = document.getElementById('status');
            status.classList.remove('active');
        }}

        // Log initialization
        console.log('Mulberry Web-Music IDE loaded');
        console.log('Settings:', {{ frequency, waveform, attack, decay, sustain, release }});
    </script>
</body>
</html>
"""

# Render the audio component
st.markdown("---")
components.html(audio_component_html, height=500, scrolling=True)

# Additional information section
st.markdown("---")
st.markdown("### 📚 Learning Resources")

col1, col2, col3 = st.columns(3)

with col1:
    st.markdown("""
    **Web Audio API Concepts**
    - AudioContext: The audio engine
    - OscillatorNode: Sound generator
    - GainNode: Volume control
    - Audio Graph: Node connections
    """)

with col2:
    st.markdown("""
    **Waveform Types**
    - Sine: Pure, smooth tone
    - Square: Hollow, clarinet-like
    - Sawtooth: Bright, brass-like
    - Triangle: Mellow, flute-like
    """)

with col3:
    st.markdown("""
    **Musical Notes**
    - A4 = 440 Hz (standard tuning)
    - Octave = 2x frequency
    - Semitone = 2^(1/12) ≈ 1.059
    - C-Major: No sharps/flats
    """)

# Footer
st.markdown("---")
st.markdown("""
<div style='text-align: center; color: #666; padding: 20px;'>
    <p>🎵 Mulberry Web-Music IDE - Built with Streamlit and Web Audio API</p>
    <p>Generate sounds in real-time with no pre-recorded audio files!</p>
</div>
""", unsafe_allow_html=True)
