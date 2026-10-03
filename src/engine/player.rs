use crate::domain::config::{AppConfig, PlaybackPreset};
use crate::domain::stats::PracticeStats;
use crate::engine::queue::SentenceQueue;
use crate::error::Result;
use crate::infra::audio::AudioEngine;
use std::time::{Duration, Instant};

pub const MIN_SHADOW_GAP_SECS: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Replaying,
    Paused,
    WaitingGap,
    Stopped,
}

#[derive(Debug, Clone)]
pub enum ShadowLoopPhase {
    Stopped,
    PlayingReference {
        repetition: u32,
    },
    ShadowGap {
        repetition: u32,
        start: Instant,
        duration: Duration,
    },
    PlayingReplay {
        repetition: u32,
    },
    PostReplayGap {
        repetition: u32,
        start: Instant,
        duration: Duration,
    },
    Paused {
        prev: Box<ShadowLoopPhase>,
    },
}

pub struct ShadowPlayer {
    audio: AudioEngine,
    phase: ShadowLoopPhase,
    config: AppConfig,
}

impl ShadowPlayer {
    pub fn new(audio: AudioEngine, config: AppConfig) -> Self {
        Self {
            audio,
            phase: ShadowLoopPhase::Stopped,
            config,
        }
    }

    pub fn phase(&self) -> &ShadowLoopPhase {
        &self.phase
    }

    pub fn status(&self) -> PlaybackStatus {
        match &self.phase {
            ShadowLoopPhase::Stopped => PlaybackStatus::Stopped,
            ShadowLoopPhase::PlayingReference { .. } => {
                if self.audio.is_paused() {
                    PlaybackStatus::Paused
                } else {
                    PlaybackStatus::Playing
                }
            }
            ShadowLoopPhase::PlayingReplay { .. } => {
                if self.audio.is_paused() {
                    PlaybackStatus::Paused
                } else {
                    PlaybackStatus::Replaying
                }
            }
            ShadowLoopPhase::ShadowGap { .. } | ShadowLoopPhase::PostReplayGap { .. } => {
                PlaybackStatus::WaitingGap
            }
            ShadowLoopPhase::Paused { .. } => PlaybackStatus::Paused,
        }
    }

    pub fn current_repetition(&self) -> u32 {
        match &self.phase {
            ShadowLoopPhase::PlayingReference { repetition }
            | ShadowLoopPhase::ShadowGap { repetition, .. }
            | ShadowLoopPhase::PlayingReplay { repetition }
            | ShadowLoopPhase::PostReplayGap { repetition, .. } => *repetition,
            ShadowLoopPhase::Paused { prev } => match **prev {
                ShadowLoopPhase::PlayingReference { repetition }
                | ShadowLoopPhase::ShadowGap { repetition, .. }
                | ShadowLoopPhase::PlayingReplay { repetition }
                | ShadowLoopPhase::PostReplayGap { repetition, .. } => repetition,
                _ => 1,
            },
            ShadowLoopPhase::Stopped => 1,
        }
    }

    pub fn play_current(&mut self, queue: &mut SentenceQueue) -> Result<()> {
        let Some(sentence) = queue.current_mut() else {
            self.stop();
            return Ok(());
        };

        sentence.play_count += 1;
        if let Err(e) = self.audio.play_file(&sentence.audio_path) {
            self.stop();
            return Err(e);
        }
        self.phase = ShadowLoopPhase::PlayingReference { repetition: 1 };
        Ok(())
    }

    pub fn toggle_play(&mut self, queue: &mut SentenceQueue) -> Result<()> {
        match self.phase.clone() {
            ShadowLoopPhase::Stopped => self.play_current(queue),
            ShadowLoopPhase::PlayingReference { .. } | ShadowLoopPhase::PlayingReplay { .. } => {
                self.audio.toggle_pause();
                Ok(())
            }
            ShadowLoopPhase::ShadowGap { .. } | ShadowLoopPhase::PostReplayGap { .. } => {
                self.phase = ShadowLoopPhase::Paused {
                    prev: Box::new(self.phase.clone()),
                };
                Ok(())
            }
            ShadowLoopPhase::Paused { prev } => {
                self.phase = *prev;
                Ok(())
            }
        }
    }

    pub fn replay(&mut self, queue: &mut SentenceQueue) -> Result<()> {
        self.play_current(queue)
    }

    pub fn next(&mut self, queue: &mut SentenceQueue) -> Result<()> {
        queue.advance();
        self.play_current(queue)
    }

    pub fn previous(&mut self, queue: &mut SentenceQueue) -> Result<()> {
        queue.previous();
        self.play_current(queue)
    }

    pub fn stop(&mut self) {
        self.audio.stop();
        self.phase = ShadowLoopPhase::Stopped;
    }

    pub fn tick(&mut self, queue: &mut SentenceQueue, stats: &mut PracticeStats) -> Result<()> {
        match self.phase.clone() {
            ShadowLoopPhase::PlayingReference { repetition } => {
                if self.audio.is_done() {
                    let is_shadow = self.config.preset == PlaybackPreset::Shadow;
                    stats.record_playback(self.audio.elapsed().as_secs_f64(), is_shadow);
                    self.handle_reference_done(repetition, queue, stats)?;
                }
            }
            ShadowLoopPhase::ShadowGap {
                repetition,
                start,
                duration,
            } => {
                if start.elapsed() >= duration {
                    self.start_replay(repetition, queue)?;
                }
            }
            ShadowLoopPhase::PlayingReplay { repetition } => {
                if self.audio.is_done() {
                    self.handle_replay_done(repetition, queue, stats)?;
                }
            }
            ShadowLoopPhase::PostReplayGap {
                repetition,
                start,
                duration,
            } => {
                if start.elapsed() >= duration {
                    self.finish_repetition_or_advance(repetition, queue, stats)?;
                }
            }
            ShadowLoopPhase::Stopped | ShadowLoopPhase::Paused { .. } => {}
        }
        Ok(())
    }

    fn handle_reference_done(
        &mut self,
        repetition: u32,
        queue: &mut SentenceQueue,
        stats: &mut PracticeStats,
    ) -> Result<()> {
        match self.config.preset {
            PlaybackPreset::Listen => self.finish_repetition_or_advance(repetition, queue, stats),
            PlaybackPreset::Repeat => {
                let gap = self.effective_shadow_gap();
                self.phase = ShadowLoopPhase::PostReplayGap {
                    repetition,
                    start: Instant::now(),
                    duration: gap,
                };
                Ok(())
            }
            PlaybackPreset::Shadow => {
                let gap = self.effective_shadow_gap();
                self.phase = ShadowLoopPhase::ShadowGap {
                    repetition,
                    start: Instant::now(),
                    duration: gap,
                };
                Ok(())
            }
        }
    }

    pub fn effective_shadow_gap(&self) -> Duration {
        let min = Duration::from_secs_f32(MIN_SHADOW_GAP_SECS);
        self.audio.duration().max(min)
    }

    fn start_replay(&mut self, repetition: u32, queue: &mut SentenceQueue) -> Result<()> {
        let Some(sentence) = queue.current_mut() else {
            self.stop();
            return Ok(());
        };
        sentence.shadow_count += 1;
        self.audio.play_file(&sentence.audio_path)?;
        self.phase = ShadowLoopPhase::PlayingReplay { repetition };
        Ok(())
    }

    fn handle_replay_done(
        &mut self,
        repetition: u32,
        queue: &mut SentenceQueue,
        stats: &mut PracticeStats,
    ) -> Result<()> {
        if self.config.pause_after_replay_secs > 0.0 {
            let post_gap = Duration::from_secs_f32(self.config.pause_after_replay_secs);
            self.phase = ShadowLoopPhase::PostReplayGap {
                repetition,
                start: Instant::now(),
                duration: post_gap,
            };
            Ok(())
        } else {
            self.finish_repetition_or_advance(repetition, queue, stats)
        }
    }

    fn finish_repetition_or_advance(
        &mut self,
        repetition: u32,
        queue: &mut SentenceQueue,
        stats: &mut PracticeStats,
    ) -> Result<()> {
        if repetition < self.config.repeat_count {
            let next_rep = repetition + 1;
            let Some(sentence) = queue.current() else {
                self.stop();
                return Ok(());
            };
            self.audio.play_file(&sentence.audio_path)?;
            self.phase = ShadowLoopPhase::PlayingReference {
                repetition: next_rep,
            };
            Ok(())
        } else {
            stats.record_sentence_completed();
            if self.config.auto_advance {
                queue.advance();
                self.play_current(queue)
            } else {
                self.stop();
                Ok(())
            }
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn audio(&self) -> &AudioEngine {
        &self.audio
    }

    pub fn audio_mut(&mut self) -> &mut AudioEngine {
        &mut self.audio
    }
}
