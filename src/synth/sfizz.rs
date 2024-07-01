#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::path::PathBuf;

mod bind {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/sfizz_bindings.rs"));
}

#[derive(Debug)]
pub struct FailedToLoadFileError {
    pub file_path: PathBuf,
}

impl std::fmt::Display for FailedToLoadFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to load file '{}'", self.file_path.display())
    }
}

pub enum OversamplingFactor {
    X1 = 1,
    X2 = 2,
    X4 = 4,
    X8 = 8,
}

pub enum ProcessingMode {
    Live = 0,
    Freewheeling = 1,
}

pub struct Synth {
    c_synth: *mut bind::sfizz_synth_t,
    sample_rate: Option<u32>,
    num_frames: Option<usize>,
}

unsafe impl Send for Synth {}

impl Synth {
    pub fn set_oversampling_factor(&mut self, factor: OversamplingFactor) {
        unsafe {
            bind::sfizz_set_oversampling_factor(
                self.c_synth,
                factor as bind::sfizz_oversampling_factor_t,
            );
        }
    }

    pub fn set_preload_size(&mut self, size: u32) {
        unsafe {
            bind::sfizz_set_preload_size(self.c_synth, size);
        }
    }

    pub fn set_sample_quality(&mut self, pm: ProcessingMode, quality: u8) {
        unsafe {
            bind::sfizz_set_sample_quality(
                self.c_synth,
                pm as bind::sfizz_process_mode_t,
                quality as i32,
            );
        }
    }

    pub fn set_num_voices(&mut self, num_voices: u16) {
        unsafe {
            bind::sfizz_set_num_voices(self.c_synth, num_voices as i32);
        }
    }

    pub fn get_num_active_voices(&self) -> u16 {
        unsafe { bind::sfizz_get_num_active_voices(self.c_synth) as u16 }
    }

    pub fn send_note_on(&mut self, note_number: u8, velocity: u8) {
        unsafe {
            bind::sfizz_send_note_on(self.c_synth, 0, note_number as i32, velocity as i32);
        }
    }

    pub fn send_note_off(&mut self, note_number: u8, velocity: u8) {
        unsafe {
            bind::sfizz_send_note_off(self.c_synth, 0, note_number as i32, velocity as i32);
        }
    }

    pub fn send_polyphonic_aftertouch(&mut self, note_number: u8, pressure: u8) {
        unsafe {
            bind::sfizz_send_poly_aftertouch(self.c_synth, 0, note_number as i32, pressure as i32);
        }
    }

    pub fn send_cc(&mut self, cmd: u8, value: u8) {
        unsafe {
            bind::sfizz_send_cc(self.c_synth, 0, cmd as i32, value as i32);
        }
    }

    pub fn send_channel_aftertouch(&mut self, pressure: u8) {
        unsafe {
            bind::sfizz_send_channel_aftertouch(self.c_synth, 0, pressure as i32);
        }
    }

    pub fn send_pitch_wheel(&mut self, value: i16) {
        unsafe {
            bind::sfizz_send_pitch_wheel(self.c_synth, 0, value as i32);
        }
    }

    pub fn load_file(&mut self, path: &std::path::Path) -> Result<(), FailedToLoadFileError> {
        let path_c = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        let result = unsafe { bind::sfizz_load_file(self.c_synth, path_c.as_ptr()) };
        if result {
            Ok(())
        } else {
            Err(FailedToLoadFileError {
                file_path: path.to_owned(),
            })
        }
    }

    pub fn silence(&mut self) {
        unsafe {
            bind::sfizz_all_sound_off(self.c_synth);
        }
    }

    pub fn render_block(&mut self, lbuf: &mut [f32], rbuf: &mut [f32]) {
        if self.sample_rate.is_some() && self.num_frames.is_some() {
            let mut channels = [lbuf.as_mut_ptr(), rbuf.as_mut_ptr()];
            let channels = channels.as_mut_ptr();
            let num_frames = lbuf.len().min(rbuf.len()); // can be actually less than the num frames set by fn
            unsafe {
                const num_channels: i32 = 2;
                bind::sfizz_render_block(self.c_synth, channels, num_channels, num_frames as i32);
            }
        }
    }

    pub fn num_frames(&self) -> Option<usize> {
        self.num_frames
    }

    pub fn set_num_frames(&mut self, num_frames: usize) {
        self.num_frames = Some(num_frames);
        unsafe {
            bind::sfizz_set_samples_per_block(self.c_synth, num_frames as i32);
        }
    }

    pub fn sample_rate(&self) -> Option<u32> {
        self.sample_rate
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = Some(sample_rate);
        unsafe {
            bind::sfizz_set_sample_rate(self.c_synth, sample_rate as f32);
        }
    }

    // _synth.setOversamplingFactor(1);
    // _synth.setPreloadSize(65536);
    // _synth.setSamplesPerBlock(audioCfg.numBufferFrames);
    // _synth.setSampleRate(audioCfg.sampleRate);
    // _synth.setSampleQuality(sfz::Sfizz::ProcessMode::ProcessLive, 2);
    // _synth.setNumVoices(64);
}

impl Default for Synth {
    fn default() -> Self {
        let mut synth = Self {
            c_synth: unsafe { bind::sfizz_create_synth() },
            sample_rate: None,
            num_frames: None,
        };
        synth.set_oversampling_factor(OversamplingFactor::X1);
        synth.set_preload_size(65536);
        synth.set_sample_quality(ProcessingMode::Live, 2);
        synth.set_num_voices(64);
        // sfizz_set_samples_per_block(synth, BUFFER_SIZE);
        // sfizz_set_sample_rate(synth, SAMPLE_RATE);
        synth
    }
}

impl Drop for Synth {
    fn drop(&mut self) {
        unsafe { bind::sfizz_free(self.c_synth) };
    }
}

impl Clone for Synth {
    fn clone(&self) -> Self {
        let mut new_obj = Synth::default();
        if let Some(sr) = self.sample_rate {
            new_obj.set_sample_rate(sr)
        }
        if let Some(nf) = self.num_frames {
            new_obj.set_num_frames(nf)
        }
        //TODO: implement CC chache?
        new_obj
    }
}
