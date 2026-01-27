use std::path::PathBuf;

use super::volta_home;
use crate::error::{EnvironmentError, ErrorKind, Fallible};

pub(super) fn default_home_dir() -> Fallible<PathBuf> {
    let mut home = dirs::home_dir().ok_or_else(|| ErrorKind::from(EnvironmentError::NoHome))?;
    home.push(".volta");
    Ok(home)
}

/// # Errors
///
/// Returns an error if the Volta home directory cannot be determined.
pub fn env_paths() -> Fallible<Vec<PathBuf>> {
    let home = volta_home()?;
    Ok(vec![home.shim_dir().to_owned()])
}
