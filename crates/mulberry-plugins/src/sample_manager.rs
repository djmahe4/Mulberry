//! Sample manager: stores and retrieves normalised mono audio samples.

/// A loaded audio sample.
#[derive(Debug, Clone)]
pub struct SampleEntry {
    pub id: u32,
    pub name: String,
    pub data: Vec<f32>,
    pub sample_rate: u32,
    pub duration_secs: f32,
}

/// Manages a collection of loaded audio samples.
pub struct SampleManager {
    samples: Vec<SampleEntry>,
    next_id: u32,
}

impl SampleManager {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a sample and return its assigned id.
    pub fn add_sample(&mut self, name: &str, data: Vec<f32>, sample_rate: u32) -> u32 {
        let duration_secs = if sample_rate > 0 {
            data.len() as f32 / sample_rate as f32
        } else {
            0.0
        };
        let id = self.next_id;
        self.next_id += 1;
        self.samples.push(SampleEntry {
            id,
            name: name.to_string(),
            data,
            sample_rate,
            duration_secs,
        });
        id
    }

    /// Remove a sample by id. Returns `true` if it was found.
    pub fn remove_sample(&mut self, id: u32) -> bool {
        if let Some(pos) = self.samples.iter().position(|s| s.id == id) {
            self.samples.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_sample(&self, id: u32) -> Option<&SampleEntry> {
        self.samples.iter().find(|s| s.id == id)
    }

    pub fn list_samples(&self) -> Vec<(u32, &str)> {
        self.samples.iter().map(|s| (s.id, s.name.as_str())).collect()
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

impl Default for SampleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get_remove() {
        let mut mgr = SampleManager::new();
        let data = vec![0.1_f32, 0.2, 0.3];
        let id = mgr.add_sample("kick", data.clone(), 44100);
        assert_eq!(mgr.sample_count(), 1);

        let entry = mgr.get_sample(id).expect("should find sample");
        assert_eq!(entry.name, "kick");
        assert_eq!(entry.data, data);
        assert_eq!(entry.sample_rate, 44100);

        assert!(mgr.remove_sample(id));
        assert_eq!(mgr.sample_count(), 0);
        assert!(mgr.get_sample(id).is_none());
    }

    #[test]
    fn list_samples_returns_ids_and_names() {
        let mut mgr = SampleManager::new();
        let id1 = mgr.add_sample("snare", vec![0.0; 100], 44100);
        let id2 = mgr.add_sample("hat", vec![0.0; 50], 44100);
        let list = mgr.list_samples();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&(id1, "snare")));
        assert!(list.contains(&(id2, "hat")));
    }
}
