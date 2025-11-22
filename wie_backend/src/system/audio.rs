use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use smaf_player::{SmafEvent, parse_smaf};

use crate::{System, audio_sink::AudioSink};

pub type AudioHandle = u32;

#[derive(Debug)]
pub enum AudioError {
    InvalidHandle,
    InvalidAudio,
    NotPlaying,
}

enum AudioFile {
    Smaf(Vec<u8>),
}

#[derive(Clone)]
struct PlaybackControl {
    is_playing: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    should_loop: Arc<AtomicBool>,
    volume: Arc<AtomicU8>,
}

impl PlaybackControl {
    fn new(should_loop: bool) -> Self {
        Self {
            is_playing: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            should_loop: Arc::new(AtomicBool::new(should_loop)),
            volume: Arc::new(AtomicU8::new(100)),
        }
    }

    fn start(&self) {
        self.is_playing.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
    }

    fn stop(&self) {
        self.is_playing.store(false, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
    }

    fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }

    fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    fn should_loop(&self) -> bool {
        self.should_loop.load(Ordering::SeqCst)
    }

    fn set_loop(&self, should_loop: bool) {
        self.should_loop.store(should_loop, Ordering::SeqCst);
    }

    fn set_volume(&self, volume: u8) {
        self.volume.store(volume.min(100), Ordering::SeqCst);
    }

    fn get_volume(&self) -> u8 {
        self.volume.load(Ordering::SeqCst)
    }
}

pub struct Audio {
    sink: Arc<Box<dyn AudioSink>>,
    files: BTreeMap<AudioHandle, AudioFile>,
    playback_controls: BTreeMap<AudioHandle, PlaybackControl>,
    last_audio_handle: AudioHandle,
    global_volume: Arc<AtomicU8>,
}

impl Audio {
    pub fn new(sink: Box<dyn AudioSink>) -> Self {
        Self {
            sink: Arc::new(sink),
            files: BTreeMap::new(),
            playback_controls: BTreeMap::new(),
            last_audio_handle: 0,
            global_volume: Arc::new(AtomicU8::new(100)),
        }
    }

    pub fn load_smaf(&mut self, data: &[u8]) -> Result<AudioHandle, AudioError> {
        let audio_handle = self.last_audio_handle;

        self.last_audio_handle += 1;
        self.files.insert(audio_handle, AudioFile::Smaf(data.to_vec()));

        Ok(audio_handle)
    }

    pub fn play(&mut self, system: &System, audio_handle: AudioHandle) -> Result<(), AudioError> {
        self.play_with_loop(system, audio_handle, false)
    }

    pub fn play_with_loop(&mut self, system: &System, audio_handle: AudioHandle, should_loop: bool) -> Result<(), AudioError> {
        match self.files.get(&audio_handle) {
            Some(AudioFile::Smaf(data)) => {
                // Stop existing playback if any
                if let Some(control) = self.playback_controls.get(&audio_handle) {
                    control.stop();
                }

                // Create new playback control
                let control = PlaybackControl::new(should_loop);
                control.start();
                self.playback_controls.insert(audio_handle, control.clone());

                let player = SmafPlayer::new(data);
                let mut system_clone = system.clone();
                let sink_clone = self.sink.clone();

                system.spawn(async move || {
                    loop {
                        player.play(&mut system_clone, &**sink_clone, &control).await;

                        if !control.should_loop() || !control.is_playing() {
                            break;
                        }
                    }

                    control.stop();
                    Ok(())
                });
            }
            None => return Err(AudioError::InvalidHandle),
        }

        Ok(())
    }

    pub fn stop(&mut self, _system: &System, audio_handle: AudioHandle) -> Result<(), AudioError> {
        match self.playback_controls.get(&audio_handle) {
            Some(control) => {
                control.stop();
                Ok(())
            }
            None => Err(AudioError::InvalidHandle),
        }
    }

    pub fn pause(&self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        match self.playback_controls.get(&audio_handle) {
            Some(control) => {
                if !control.is_playing() {
                    return Err(AudioError::NotPlaying);
                }
                control.pause();
                Ok(())
            }
            None => Err(AudioError::InvalidHandle),
        }
    }

    pub fn resume(&self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        match self.playback_controls.get(&audio_handle) {
            Some(control) => {
                if !control.is_playing() {
                    return Err(AudioError::NotPlaying);
                }
                control.resume();
                Ok(())
            }
            None => Err(AudioError::InvalidHandle),
        }
    }

    pub fn set_volume(&self, audio_handle: AudioHandle, volume: u8) -> Result<(), AudioError> {
        match self.playback_controls.get(&audio_handle) {
            Some(control) => {
                control.set_volume(volume);
                self.sink.set_volume(volume);
                Ok(())
            }
            None => Err(AudioError::InvalidHandle),
        }
    }

    pub fn get_volume(&self, audio_handle: AudioHandle) -> Result<u8, AudioError> {
        match self.playback_controls.get(&audio_handle) {
            Some(control) => Ok(control.get_volume()),
            None => Err(AudioError::InvalidHandle),
        }
    }

    pub fn set_global_volume(&self, volume: u8) {
        self.global_volume.store(volume.min(100), Ordering::SeqCst);
        self.sink.set_volume(volume);
    }

    pub fn get_global_volume(&self) -> u8 {
        self.global_volume.load(Ordering::SeqCst)
    }

    pub fn close(&mut self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        // Stop playback if active
        if let Some(control) = self.playback_controls.get(&audio_handle) {
            control.stop();
        }

        self.playback_controls.remove(&audio_handle);

        if self.files.remove(&audio_handle).is_none() {
            return Err(AudioError::InvalidHandle);
        }

        Ok(())
    }
}

pub struct SmafPlayer {
    events: Vec<(usize, SmafEvent)>,
}

impl SmafPlayer {
    pub fn new(data: &[u8]) -> Self {
        Self { events: parse_smaf(data) }
    }

    pub async fn play(&self, system: &mut System, sink: &dyn AudioSink, control: &PlaybackControl) {
        let mut play_time = 0;
        for (time, event) in &self.events {
            // Check if playback was stopped
            if !control.is_playing() {
                break;
            }

            // Wait while paused
            while control.is_paused() && control.is_playing() {
                system.sleep(10).await; // Small delay to avoid busy-waiting
            }

            // Sleep for the time difference
            system.sleep((time - play_time) as _).await;

            // Check again after sleeping
            if !control.is_playing() {
                break;
            }

            match event {
                SmafEvent::Wave {
                    channel,
                    sampling_rate,
                    data,
                } => {
                    sink.play_wave(*channel, *sampling_rate, data);
                }
                SmafEvent::MidiNoteOn { channel, note, velocity } => {
                    sink.midi_note_on(*channel, *note, *velocity);
                }
                SmafEvent::MidiNoteOff { channel, note, velocity } => {
                    sink.midi_note_off(*channel, *note, *velocity);
                }
                SmafEvent::MidiProgramChange { channel, program } => {
                    sink.midi_program_change(*channel, *program);
                }
                SmafEvent::MidiControlChange { channel, control, value } => {
                    sink.midi_control_change(*channel, *control, *value);
                }
                SmafEvent::End => {}
            }

            play_time = *time;
        }
    }
}
