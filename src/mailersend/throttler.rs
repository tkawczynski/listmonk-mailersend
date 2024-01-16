use std::time::Duration;

use chrono::Timelike;
use chrono::Utc;

pub struct Throttler {
    per_minute: u32,
    current_count: u32,
}

impl Throttler {
    pub fn new(per_minute: u32) -> Self {
        Throttler {
            per_minute,
            current_count: 0,
        }
    }
    pub fn try_blocking(&mut self, timeout: Duration) -> bool {
        let mut elapsed_time = Duration::new(0, 0);
        let mut current_minute = Utc::now().minute();
        while elapsed_time < timeout {
            if self.current_count < self.per_minute {
                self.current_count += 1;
                return true;
            }
            if Utc::now().minute() != current_minute {
                self.current_count = 0;
                current_minute = Utc::now().minute();
            }
            elapsed_time += Duration::from_millis(100);
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_should_block() {
        let mut throttler = Throttler::new(1);
        assert!(throttler.try_blocking(Duration::from_secs(1)));
        assert!(!throttler.try_blocking(Duration::from_secs(1)));
    }
}
