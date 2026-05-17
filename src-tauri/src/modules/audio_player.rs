use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tauri::{AppHandle, Emitter};
use web_audio_api::context::{
    AudioContext, AudioContextLatencyCategory, AudioContextOptions, BaseAudioContext,
};
use web_audio_api::node::AudioNode;
use web_audio_api::MediaElement;

#[derive(Clone, Serialize)]
pub struct AudioState {
    pub is_playing: bool,
    pub is_paused: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: f32,
    pub current_path: Option<String>,
}

pub struct AudioPlayer {
    ctx: AudioContext,
    gain: web_audio_api::node::GainNode,
    media: Option<MediaElement>,
    source: Option<web_audio_api::node::MediaElementAudioSourceNode>,

    is_playing: bool,
    is_paused: bool,
    duration: f64,
    volume: f32,
    current_path: Option<String>,

    /// Flag set by streaming decoder when track naturally reaches end
    ended: Arc<AtomicBool>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let ctx = AudioContext::new(AudioContextOptions {
            latency_hint: AudioContextLatencyCategory::Playback,
            ..AudioContextOptions::default()
        });
        let gain = ctx.create_gain();
        gain.connect(&ctx.destination());

        AudioPlayer {
            ctx,
            gain,
            media: None,
            source: None,
            is_playing: false,
            is_paused: false,
            duration: 0.0,
            volume: 1.0,
            current_path: None,
            ended: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn play_file(&mut self, path: &str) -> Result<AudioState, String> {
        self.stop_internal();

        let duration = get_audio_duration(path)?;

        let mut media =
            MediaElement::new(path).map_err(|e| format!("打开音频文件失败: {}", e))?;

        let source = self.ctx.create_media_element_source(&mut media);
        source.connect(&self.gain);

        self.ended.store(false, Ordering::SeqCst);

        media.play();

        self.media = Some(media);
        self.source = Some(source);
        self.is_playing = true;
        self.is_paused = false;
        self.duration = duration;
        self.current_path = Some(path.to_string());

        Ok(self.get_state())
    }

    pub fn pause(&mut self) -> Result<AudioState, String> {
        if let Some(ref media) = self.media {
            media.pause();
        }
        self.is_playing = false;
        self.is_paused = true;
        Ok(self.get_state())
    }

    pub fn resume(&mut self) -> Result<AudioState, String> {
        if let Some(ref media) = self.media {
            if self.duration > 0.0 && media.current_time() >= self.duration - 0.05 {
                media.set_current_time(0.0);
            }
            media.play();
        }
        self.is_playing = true;
        self.is_paused = false;
        Ok(self.get_state())
    }

    pub fn seek(&mut self, position: f64) -> Result<AudioState, String> {
        if let Some(ref media) = self.media {
            let pos = position.max(0.0).min(self.duration);
            media.set_current_time(pos);
            self.ended.store(false, Ordering::SeqCst);
        }
        Ok(self.get_state())
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = (vol / 100.0).clamp(0.0, 1.0);
        self.gain.gain().set_value(self.volume);
    }

    pub fn get_state(&self) -> AudioState {
        let position = self
            .media
            .as_ref()
            .map(|m| m.current_time().min(self.duration))
            .unwrap_or(0.0);

        AudioState {
            is_playing: self.is_playing,
            is_paused: self.is_paused,
            position,
            duration: self.duration,
            volume: (self.volume * 100.0).round(),
            current_path: self.current_path.clone(),
        }
    }

    pub fn poll_track_ended(&self) -> bool {
        if self.duration <= 0.0 || !self.is_playing {
            return false;
        }
        if let Some(ref media) = self.media {
            let pos = media.current_time();
            pos >= self.duration - 0.12
        } else {
            false
        }
    }

    fn stop_internal(&mut self) {
        self.source = None;
        if let Some(ref media) = self.media {
            media.pause();
        }
        self.media = None;
        self.is_playing = false;
        self.is_paused = false;
        self.duration = 0.0;
        self.current_path = None;
    }
}

fn get_audio_duration(path: &str) -> Result<f64, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("打开音频文件失败: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let meta_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &meta_opts)
        .map_err(|e| format!("探测音频格式失败: {}", e))?;

    let track = probed
        .format
        .tracks()
        .first()
        .ok_or("音频文件中没有轨道")?;

    let codec_params = &track.codec_params;
    let duration = codec_params
        .time_base
        .and_then(|tb| codec_params.n_frames.map(|n| tb.calc_time(n)))
        .map(|t| t.seconds as f64 + t.frac)
        .unwrap_or(0.0);

    Ok(duration)
}

// ─── Tauri Commands ───────────────────────────────────────────

#[tauri::command]
pub fn play_audio(
    path: String,
    state: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<AudioState, String> {
    let mut player = state.lock().map_err(|e| format!("{}", e))?;
    player.play_file(&path)
}

#[tauri::command]
pub fn pause_audio(
    state: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<AudioState, String> {
    let mut player = state.lock().map_err(|e| format!("{}", e))?;
    player.pause()
}

#[tauri::command]
pub fn resume_audio(
    state: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<AudioState, String> {
    let mut player = state.lock().map_err(|e| format!("{}", e))?;
    player.resume()
}

#[tauri::command]
pub fn seek_audio(
    position: f64,
    state: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<AudioState, String> {
    let mut player = state.lock().map_err(|e| format!("{}", e))?;
    player.seek(position)
}

#[tauri::command]
pub fn stop_audio(
    state: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<(), String> {
    let mut player = state.lock().map_err(|e| format!("{}", e))?;
    player.stop_internal();
    Ok(())
}

#[tauri::command]
pub fn set_audio_volume(
    volume: f32,
    state: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<(), String> {
    let mut player = state.lock().map_err(|e| format!("{}", e))?;
    player.set_volume(volume);
    Ok(())
}

#[tauri::command]
pub fn get_audio_state(
    state: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<AudioState, String> {
    let player = state.lock().map_err(|e| format!("{}", e))?;
    Ok(player.get_state())
}

/// Start a background timer that periodically emits progress events.
/// Called once during app setup.
pub fn start_progress_timer(app: &AppHandle, player: Arc<Mutex<AudioPlayer>>) {
    let app_clone = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(200));

        let mut track_ended = false;
        {
            let p = match player.lock() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let state = p.get_state();
            if state.duration <= 0.0 {
                continue;
            }

            let _ = app_clone.emit("audio-progress", serde_json::json!({
                "position": state.position,
                "duration": state.duration,
                "is_playing": state.is_playing,
                "is_paused": state.is_paused,
            }));

            if state.is_playing && p.poll_track_ended() {
                track_ended = true;
            }
        }

        if track_ended {
            if let Ok(mut p) = player.lock() {
                p.stop_internal();
            }
            let _ = app_clone.emit("audio-ended", serde_json::json!({}));
        }
    });
}
