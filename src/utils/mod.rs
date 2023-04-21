use log::{Level, Metadata, Record};
use chrono::prelude::*;

pub static DEFAULT_LOGGER: DefaultLogger = DefaultLogger;

pub struct DefaultLogger;

impl log::Log for DefaultLogger {
    /// Method for check if the log is enabled
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata to check
    ///
    /// # Returns
    ///
    /// * `bool` - If the log is enabled
    ///
    fn enabled(&self, metadata: &Metadata) -> bool { metadata.level() < Level::Trace }

    /// Method for log the message
    ///
    /// # Arguments
    ///
    /// * `record` - The record to log
    ///
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{} - {}]: {}", Utc::now().timestamp_millis(), record.level(), record.args());
        }
    }

    /// Method for flush the log
    fn flush(&self) {}
}

