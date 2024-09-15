use godot::prelude::*;
use log::{Level, Metadata, Record};

/// To make it easier to work with rust dependencies, a custom implementation of a `log` sink
/// is implemented for godot files. Instead of using special functions, every type of message
/// will work.

pub struct GodotLogSink;

impl log::Log for GodotLogSink {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        match record.level() {
            Level::Debug => godot_print!("[color=darkgray]\\[DEBUG - {}\\] {}[/color]",
                record.target(), record.args()),
            Level::Info => godot_print!("\\[INFO - {}\\] {}",
                record.target(), record.args()),
            Level::Warn => godot_warn!("\\[WARN - {}\\] {}",
                record.target(), record.args()),
            Level::Error => godot_error!("\\[ERR - {} \\] {}",
                record.target(), record.args()),
            _ => {}
        }
    }

    fn flush(&self) {
        //NOOP
    }
}