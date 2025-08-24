use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum TranscendiaError {
    CannotLoadConfig,
    CannotSaveConfig,
    CannotGetMonitors,
}
