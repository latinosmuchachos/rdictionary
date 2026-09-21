mod settings;
mod store;

use std::path::PathBuf;

pub use settings::Settings;
pub use store::Store;

/// Returns the data directory located next to the project's `Cargo.toml`.
pub fn get_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}
