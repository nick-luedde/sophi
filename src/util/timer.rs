use colored::Colorize;
use std::time::SystemTime;

pub struct SophiTimer {
    start: SystemTime,
    duration_ms: u128,
}

impl SophiTimer {
    pub fn new() -> SophiTimer {
        SophiTimer {
            start: SystemTime::now(),
            duration_ms: 0,
        }
    }

    pub fn stop(&mut self) -> &mut SophiTimer {
        self.duration_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - self
                .start
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

        return self;
    }

    pub fn print_line(&mut self) -> &mut SophiTimer {
        let fmt = format!("Completed in {} ms", self.duration_ms).bright_black();

        println!("{}", fmt);

        return self;
    }
}
