//! Optional display-only reports; no simulator, renderer or image dependency in the core.
mod error;
mod protocol;
mod assets;
mod html;
mod output;
mod snapshot;
mod combat_replay;
mod actions;
mod run_capture;
pub use actions::describe_run_action;
pub use snapshot::run_snapshot;
pub use run_capture::{RunRecorder,CaptureError,CaptureRecovery};
pub use snapshot::legacy_snapshot;
pub use combat_replay::capture_combat;

pub use error::ReportError;
pub use protocol::{ReportWriter, read_report};
pub use assets::{AssetCatalog, embed_referenced, load_assets};
pub use html::{export_html, json_for_html};
pub use output::{OutputReservation, create_trace};
