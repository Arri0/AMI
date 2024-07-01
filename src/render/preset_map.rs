use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PresetMap {
    banks: BTreeMap<u16, BTreeMap<u8, Preset>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Preset {
    pub name: String,
    pub notes: Vec<bool>,
}

impl PresetMap {
    pub fn new() -> Self {
        Self {
            banks: Default::default(),
        }
    }

    pub fn add_preset(&mut self, bank: u16, preset_id: u8, preset: Preset) {
        match self.banks.get_mut(&bank) {
            Some(presets) => {
                presets.insert(preset_id, preset);
            },
            None => {
                let mut presets = BTreeMap::new();
                presets.insert(preset_id, preset);
                self.banks.insert(bank, presets);
            },
        }
    }

    pub fn has_preset(&self, bank: u16, preset_id: u8) -> bool {
        if !self.banks.contains_key(&bank) {
            false
        } else {
            self.banks[&bank].contains_key(&preset_id)
        }
    }

    pub fn first_available_preset(&self) -> Option<(u16,u8)> {
        let bank0 = self.banks.first_key_value()?;
        return bank0.1.keys().next().map(|preset_id| (*bank0.0, *preset_id))
    }
}

impl Default for PresetMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Preset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            notes: vec![false; 128],
        }
    }

    pub fn add_note_range(&mut self, min: u8, max: u8) {
        for i in min..=max {
            self.notes[i as usize] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset() {
        let mut preset = Preset::new("Test");
        assert_eq!(preset.name, "Test");

        preset.add_note_range(0, 10);
        preset.add_note_range(20, 30);
        assert_eq!(preset.notes[0..=10], vec![true; 11]);
        assert_eq!(preset.notes[11..20], vec![false; 9]);
        assert_eq!(preset.notes[20..=30], vec![true; 11]);
        assert_eq!(preset.notes[31..], vec![false; 97]);
    }
}
