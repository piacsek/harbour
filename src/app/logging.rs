use super::App;
use crate::log::Log;

#[derive(Default)]
pub(super) struct Logging {
    log: Log,
    started: bool,
}

impl App {
    pub fn with_log(mut self, log: Log) -> App {
        self.logging.log = log;
        self
    }

    pub(super) fn log_start(&mut self) {
        if std::mem::replace(&mut self.logging.started, true) {
            return;
        }
        self.logging.log.write(|| {
            format!(
                "start version={} size={}x{} command={:?} picker={} editor={}",
                env!("CARGO_PKG_VERSION"),
                self.screen.width,
                self.screen.height,
                self.command.first().map_or("", String::as_str),
                self.picker
                    .as_ref()
                    .map_or("none".to_string(), |picker| format!(
                        "{:?}",
                        picker.program()
                    )),
                self.editor
                    .as_ref()
                    .map_or("none".to_string(), |editor| format!("{:?}", editor.name())),
            )
        });
    }
}
