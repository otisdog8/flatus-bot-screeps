use fern;
use log;
use screeps;

pub use log::LevelFilter::*;

struct JsLog;
struct JsNotify;

impl log::Log for JsLog {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        let message = format!("{}", record.args());
        js! {
            console.log(@{message});
        }
    }
    fn flush(&self) {}
}
impl log::Log for JsNotify {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        let message = format!("{}", record.args());
        js! {
            Game.notify(@{message});
        }
    }
    fn flush(&self) {}
}

pub fn setup_logging(verbosity: log::LevelFilter) {
    fern::Dispatch::new()
        .level(verbosity)
        .format(|out, message, record| {
            out.finish(format_args!(
                "({}) {}: {}",
                record.level(),
                record.target(),
                message
            ))
        }).chain(Box::new(JsLog) as Box<log::Log>)
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Warn)
                .format(|out, message, _record| {
                    let time = screeps::game::time();
                    out.finish(format_args!("[{}] {}", time, message))
                }).chain(Box::new(JsNotify) as Box<log::Log>),
        ).apply()
        .expect("expected setup_logging to only ever be called once per instance");
}
