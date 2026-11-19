use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Replaying,
    Paused,
    SpeakingGap,
    BreathingGap,
    CardComplete,
    Stopped,
}

#[derive(Debug, Clone)]
pub enum ShadowLoopPhase {
    Stopped,
    PlayingReference {
        repetition: u32,
    },
    SpeakingGap {
        repetition: u32,
        remaining: Duration,
        total: Duration,
        started_at: Instant,
    },
    PlayingReplay {
        repetition: u32,
    },
    BreathingGap {
        repetition: u32,
        remaining: Duration,
        total: Duration,
        started_at: Instant,
    },
    CardCompleteBreak {
        remaining: Duration,
        total: Duration,
        started_at: Instant,
    },
    Paused {
        prev: Box<ShadowLoopPhase>,
    },
}

impl ShadowLoopPhase {
    pub fn pause_countdown(self) -> Self {
        match self {
            Self::SpeakingGap {
                repetition,
                remaining,
                total,
                started_at,
            } => {
                let actual = remaining.saturating_sub(started_at.elapsed());
                Self::Paused {
                    prev: Box::new(Self::SpeakingGap {
                        repetition,
                        remaining: actual,
                        total,
                        started_at: Instant::now(),
                    }),
                }
            }
            Self::BreathingGap {
                repetition,
                remaining,
                total,
                started_at,
            } => {
                let actual = remaining.saturating_sub(started_at.elapsed());
                Self::Paused {
                    prev: Box::new(Self::BreathingGap {
                        repetition,
                        remaining: actual,
                        total,
                        started_at: Instant::now(),
                    }),
                }
            }
            Self::CardCompleteBreak {
                remaining,
                total,
                started_at,
            } => {
                let actual = remaining.saturating_sub(started_at.elapsed());
                Self::Paused {
                    prev: Box::new(Self::CardCompleteBreak {
                        remaining: actual,
                        total,
                        started_at: Instant::now(),
                    }),
                }
            }
            other => other,
        }
    }

    pub fn resume_from_paused(self) -> Self {
        match self {
            Self::SpeakingGap {
                repetition,
                remaining,
                total,
                ..
            } => Self::SpeakingGap {
                repetition,
                remaining,
                total,
                started_at: Instant::now(),
            },
            Self::BreathingGap {
                repetition,
                remaining,
                total,
                ..
            } => Self::BreathingGap {
                repetition,
                remaining,
                total,
                started_at: Instant::now(),
            },
            Self::CardCompleteBreak {
                remaining, total, ..
            } => Self::CardCompleteBreak {
                remaining,
                total,
                started_at: Instant::now(),
            },
            other => other,
        }
    }
}
