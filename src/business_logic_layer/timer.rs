use crate::entities::{CommandPacket, Exception, StatePacket};
use std::time::{Duration, Instant};
use std::thread;

pub struct SleepTimer {
    time: Duration,
    instant: Instant,
}

impl SleepTimer {
    pub fn new(sleep_in_millis: u64) -> SleepTimer {
        SleepTimer {
            time: Duration::from_millis(sleep_in_millis),
            instant: Instant::now(),
        }
    }

    pub fn sleep(&mut self) {
        let elapsed = self.instant.elapsed();
        self.time
            .checked_sub(elapsed)
            .and_then(|d| {
                if d == Duration::new(0, 0) {
                    None
                } else {
                    Some(d)
                }
            })
            .map(thread::sleep);
        self.instant = Instant::now();
    }
}

pub struct WaitTimer {
    time: Duration,
    instant: Instant,
}

impl WaitTimer {
    pub fn new(millis: u64) -> WaitTimer {
        WaitTimer {
            time: Duration::from_millis(millis),
            instant: Instant::now(),
        }
    }
    pub fn continue_execution(&mut self) -> bool {
        if self.instant.elapsed() > self.time {
            self.instant = Instant::now();
            true
        } else {
            false
        }
    }
}

pub struct ElapsedTimer {
    time: Duration,
    instant: Instant,
}

impl ElapsedTimer {
    pub fn new() -> ElapsedTimer {
        ElapsedTimer {
            time: Duration::new(0, 0),
            instant: Instant::now(),
        }
    }
    pub fn elapsed(&mut self) -> Duration {
        let elapsed = self.instant.elapsed();
        let res = elapsed - self.time;
        self.time = elapsed;
        res
    }
}
