use std::{thread, time};

pub struct FramerateThrottle {
    minimum_frame_length: time::Duration,
    next_frame: time::Instant,
}

impl FramerateThrottle {
    pub fn new(minimum_frame_length: time::Duration) -> Self {
        Self {
            minimum_frame_length,
            next_frame: time::Instant::now(),
        }
    }

    pub fn throttle(&mut self) {
        let now = time::Instant::now();

        if now >= self.next_frame {
            self.next_frame = now + self.minimum_frame_length;
        } else {
            Self::safe_wait(self.next_frame);
            self.next_frame += self.minimum_frame_length;
        }
    }

    fn safe_wait(target: time::Instant) {
        let mut cur_time = time::Instant::now();
        while cur_time < target {
            cur_time = time::Instant::now();

            if target - cur_time > time::Duration::from_millis(3) {
                thread::sleep(time::Duration::from_millis(1));
            }
        }
    }
}
