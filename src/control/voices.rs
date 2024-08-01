use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    InvalidVoiceIndex,
    InvalidSlotIndex,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Voice {
    pub name: String,
    pub instrument_index: Option<usize>,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
    slots: Vec<bool>,
}

impl Voice {
    pub fn slots(&self) -> &Vec<bool> {
        &self.slots
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Voices {
    num_slots: usize,
    voices: Vec<Voice>,
}

impl Voices {
    pub fn voices(&self) -> &Vec<Voice> {
        &self.voices
    }

    pub fn set_num_slots(&mut self, num_slots: usize) {
        let prev_num_slots = self.num_slots;
        self.num_slots = num_slots;
        self.update_slots(prev_num_slots);
    }

    pub fn add_voice(&mut self) {
        self.voices.push(Voice {
            name: String::new(),
            instrument_index: None,
            channel: 9,
            note: 0,
            velocity: 127,
            slots: vec![false; self.num_slots],
        });
    }

    pub fn remove_voice(&mut self, index: usize) -> Result<(), Error> {
        if index < self.voices.len() {
            self.voices.remove(index);
            Ok(())
        } else {
            Err(Error::InvalidVoiceIndex)
        }
    }

    pub fn clear(&mut self) {
        self.voices.clear();
    }

    pub fn set_voice_name(&mut self, voice_index: usize, name: String) -> Result<(), Error> {
        if voice_index < self.voices.len() {
            self.voices[voice_index].name = name;
            Ok(())
        } else {
            Err(Error::InvalidVoiceIndex)
        }
    }

    pub fn set_voice_instrument(
        &mut self,
        voice_index: usize,
        instrument_index: Option<usize>,
    ) -> Result<(), Error> {
        if voice_index < self.voices.len() {
            self.voices[voice_index].instrument_index = instrument_index;
            Ok(())
        } else {
            Err(Error::InvalidVoiceIndex)
        }
    }

    pub fn set_voice_note(&mut self, voice_index: usize, note: u8) -> Result<(), Error> {
        if voice_index < self.voices.len() {
            self.voices[voice_index].note = note;
            Ok(())
        } else {
            Err(Error::InvalidVoiceIndex)
        }
    }

    pub fn set_voice_velocity(&mut self, voice_index: usize, velocity: u8) -> Result<(), Error> {
        if voice_index < self.voices.len() {
            self.voices[voice_index].velocity = velocity;
            Ok(())
        } else {
            Err(Error::InvalidVoiceIndex)
        }
    }

    pub fn set_voice_channel(&mut self, voice_index: usize, channel: u8) -> Result<(), Error> {
        if voice_index < self.voices.len() {
            self.voices[voice_index].channel = channel;
            Ok(())
        } else {
            Err(Error::InvalidVoiceIndex)
        }
    }

    pub fn set_slot(
        &mut self,
        voice_index: usize,
        slot_index: usize,
        enabled: bool,
    ) -> Result<(), Error> {
        if voice_index < self.voices.len() {
            let voice = &mut self.voices[voice_index];
            if slot_index < voice.slots.len() {
                voice.slots[slot_index] = enabled;
                Ok(())
            } else {
                Err(Error::InvalidSlotIndex)
            }
        } else {
            Err(Error::InvalidVoiceIndex)
        }
    }

    pub fn set_all_to_silence(&mut self) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.instrument_index = None);
    }

    pub fn reindex_instruments(&mut self, removed_index: usize) {
        self.voices
            .iter_mut()
            .for_each(|voice| match voice.instrument_index {
                Some(instr_index) if instr_index == removed_index => voice.instrument_index = None,
                Some(instr_index) if instr_index > removed_index => {
                    voice.instrument_index = Some(instr_index - 1);
                }
                _ => {}
            });
    }

    fn update_slots(&mut self, prev_num_slots: usize) {
        let num_slots = self.num_slots;
        if prev_num_slots == 0 || num_slots == 0 {
            self.update_slots_resize(num_slots);
        } else if num_slots > prev_num_slots {
            if num_slots % prev_num_slots == 0 {
                self.update_slots_interleave(num_slots / prev_num_slots);
            } else {
                self.update_slots_append(num_slots - prev_num_slots)
            }
        } else if num_slots < prev_num_slots {
            if prev_num_slots % num_slots == 0 {
                self.update_slots_decimate(prev_num_slots / num_slots);
            } else {
                //FIXME: attempt to subtract with overflow
                self.update_slots_cut_out(prev_num_slots - num_slots)
            }
        }
    }

    fn update_slots_interleave(&mut self, factor: usize) {
        self.voices
            .iter_mut()
            .for_each(|voice| interpolate_slots(voice, factor));
    }

    fn update_slots_append(&mut self, number: usize) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.slots.resize(voice.slots.len() + number, false));
    }

    fn update_slots_decimate(&mut self, factor: usize) {
        self.voices
            .iter_mut()
            .for_each(|voice| decimate_slots(voice, factor));
    }

    fn update_slots_cut_out(&mut self, number: usize) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.slots.resize(voice.slots.len() - number, false));
    }

    fn update_slots_resize(&mut self, size: usize) {
        self.voices
            .iter_mut()
            .for_each(|voice| voice.slots.resize(size, false));
    }
}

fn interpolate_slots(voice: &mut Voice, factor: usize) {
    let mut interpolated = Vec::with_capacity(voice.slots.len() * factor);
    for item in voice.slots.iter() {
        interpolated.push(*item);
        interpolated.extend(std::iter::repeat(false).take(factor - 1));
    }
    voice.slots = interpolated;
}

fn decimate_slots(voice: &mut Voice, factor: usize) {
    let mut decimated = Vec::with_capacity(voice.slots.len() / factor);
    for item in voice.slots.iter().step_by(factor) {
        decimated.push(*item);
    }
    voice.slots = decimated;
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn interpolate_decimate_slots() {
        //TODO: write new test

        // let v1 = DrumMachineNoise {
        //     instrument_index: 0,
        //     note: 0,
        //     velocity: 0,
        // };

        // let v2 = DrumMachineNoise {
        //     instrument_index: 1,
        //     note: 0,
        //     velocity: 0,
        // };

        // let values = vec![Some(v1.clone()), Some(v2.clone())];
        // let interpolated_values = super::interpolate_slots(&values, 2);
        // let decimated_values = super::decimate_slots(&values, 2);

        // assert_eq!(
        //     interpolated_values,
        //     vec![Some(v1.clone()), None, Some(v2.clone()), None,]
        // );

        // assert_eq!(decimated_values, vec![Some(v1.clone())]);
    }

    #[test]
    pub fn reindex_slots() {
        //TODO: write new test

        // let v1 = DrumMachineNoise {
        //     instrument_index: 0,
        //     note: 0,
        //     velocity: 0,
        // };

        // let v2 = DrumMachineNoise {
        //     instrument_index: 1,
        //     note: 0,
        //     velocity: 0,
        // };

        // let v3 = DrumMachineNoise {
        //     instrument_index: 2,
        //     note: 0,
        //     velocity: 0,
        // };

        // let values = vec![Some(v1.clone()), Some(v2.clone()), Some(v3.clone())];
        // assert_eq!(
        //     super::reindex_slots(&values, 0),
        //     vec![None, Some(v1.clone()), Some(v2.clone())]
        // );
    }
}
