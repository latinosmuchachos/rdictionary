mod settings;
mod store;

use std::path::PathBuf;

pub use settings::Settings;
pub use store::Store;

pub fn get_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}
