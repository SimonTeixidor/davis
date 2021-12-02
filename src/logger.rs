use log::{LevelFilter, Metadata, Record};
use std::io::Write;

pub struct Logger(pub bool);

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        self.0
    }

    fn log(&self, record: &Record) {
        eprintln!("{}", record.args());
    }

    fn flush(&self) {
        if let Err(e) = std::io::stderr().flush() {
            log::trace!("Failed to flush stderr: {}", e);
        }
    }
}

impl Logger {
    pub fn init(self) {
        log::set_max_level(if self.0 {
            LevelFilter::Trace
        } else {
            LevelFilter::Off
        });
        let _ = log::set_logger(Box::leak(Box::new(self)));
    }
}
