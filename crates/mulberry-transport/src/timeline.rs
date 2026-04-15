//! Timeline — track and region management.
//!
//! The [`Timeline`] holds an ordered list of [`Track`]s, each containing
//! [`Region`]s that represent clips of audio or MIDI placed at specific
//! musical positions.

use mulberry_core::time::MusicalPosition;
use serde::{Deserialize, Serialize};

// ── Region ──────────────────────────────────────────────────────────────────

/// A contiguous block of content placed on a [`Track`].
///
/// Regions reference audio or MIDI data and are positioned on the timeline
/// using a [`MusicalPosition`] start point and a duration in beats.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    /// Unique identifier for this region.
    pub id: u64,
    /// Human-readable region name.
    pub name: String,
    /// Position on the timeline where the region begins.
    pub start: MusicalPosition,
    /// Length of the region measured in beats.
    pub duration_beats: f64,
    /// Index of the track this region belongs to.
    pub track_index: usize,
}

// ── Track ───────────────────────────────────────────────────────────────────

/// A single track in the timeline.
///
/// Tracks contain zero or more [`Region`]s and carry mute/solo state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    /// Unique identifier for this track.
    pub id: u64,
    /// Human-readable track name.
    pub name: String,
    /// Regions placed on this track.
    pub regions: Vec<Region>,
    /// Whether the track is muted.
    pub muted: bool,
    /// Whether the track is soloed.
    pub solo: bool,
}

// ── Timeline ────────────────────────────────────────────────────────────────

// CONTEXT7 REVIEW:
// Issue: Region overlap queries require converting MusicalPosition to a
//        linear beat count, which depends on the time signature.
// Resolution: `regions_at_position` uses a default 4/4 assumption
//             (4 beats per bar) for the linear conversion.
// Why: Timeline does not own a TimeSignature; callers that need
//      time-signature-aware queries should perform the conversion externally.

/// The timeline manages all tracks and their regions.
///
/// It provides CRUD operations for tracks and regions and supports querying
/// which regions overlap a given musical position.
pub struct Timeline {
    tracks: Vec<Track>,
    next_id: u64,
}

impl Timeline {
    /// Creates an empty [`Timeline`].
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            next_id: 1,
        }
    }

    /// Adds a new track with the given name and returns its unique id.
    pub fn add_track(&mut self, name: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tracks.push(Track {
            id,
            name: name.to_string(),
            regions: Vec::new(),
            muted: false,
            solo: false,
        });
        id
    }

    /// Removes the track with the given id. Returns `true` if found.
    pub fn remove_track(&mut self, id: u64) -> bool {
        let len = self.tracks.len();
        self.tracks.retain(|t| t.id != id);
        self.tracks.len() != len
    }

    /// Adds a region to the specified track. Returns the region id on success,
    /// or `None` if the track was not found.
    pub fn add_region(
        &mut self,
        track_id: u64,
        name: &str,
        start: MusicalPosition,
        duration_beats: f64,
    ) -> Option<u64> {
        let track_index = self.tracks.iter().position(|t| t.id == track_id)?;
        let id = self.next_id;
        self.next_id += 1;
        self.tracks[track_index].regions.push(Region {
            id,
            name: name.to_string(),
            start,
            duration_beats,
            track_index,
        });
        Some(id)
    }

    /// Removes a region from the specified track. Returns `true` if found.
    pub fn remove_region(&mut self, track_id: u64, region_id: u64) -> bool {
        if let Some(track) = self.tracks.iter_mut().find(|t| t.id == track_id) {
            let len = track.regions.len();
            track.regions.retain(|r| r.id != region_id);
            return track.regions.len() != len;
        }
        false
    }

    /// Returns a reference to the track with the given id, if it exists.
    pub fn get_track(&self, id: u64) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    /// Returns a mutable reference to the track with the given id, if it exists.
    pub fn get_track_mut(&mut self, id: u64) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }

    /// Returns a slice of all tracks.
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// Sets the mute state on the track with the given id.
    pub fn mute_track(&mut self, id: u64, mute: bool) {
        if let Some(track) = self.get_track_mut(id) {
            track.muted = mute;
        }
    }

    /// Sets the solo state on the track with the given id.
    pub fn solo_track(&mut self, id: u64, solo: bool) {
        if let Some(track) = self.get_track_mut(id) {
            track.solo = solo;
        }
    }

    /// Returns references to all regions that overlap the given position.
    ///
    /// **Note:** This method assumes 4 beats per bar when converting
    /// [`MusicalPosition`] to a linear beat count. For non-4/4 time
    /// signatures, callers should perform their own conversion.
    pub fn regions_at_position(&self, pos: &MusicalPosition) -> Vec<&Region> {
        const ASSUMED_BEATS_PER_BAR: f64 = 4.0;

        let pos_linear = pos.bar as f64 * ASSUMED_BEATS_PER_BAR + pos.beat;

        self.tracks
            .iter()
            .flat_map(|t| t.regions.iter())
            .filter(|r| {
                let r_start = r.start.bar as f64 * ASSUMED_BEATS_PER_BAR + r.start.beat;
                let r_end = r_start + r.duration_beats;
                pos_linear >= r_start && pos_linear < r_end
            })
            .collect()
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}
