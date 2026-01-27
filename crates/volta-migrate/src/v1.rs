use std::fs::File;
#[cfg(unix)]
use std::fs::remove_file;
use std::path::PathBuf;

use super::empty::Empty;
use super::v0::V0;
use log::debug;
use volta_core::error::{Context, ErrorKind, Fallible, FilesystemError, VoltaError};
#[cfg(unix)]
use volta_core::fs::{read_dir_eager, remove_file_if_exists};
use volta_layout::v1;

/// Represents a V1 Volta Layout (used by Volta v0.7.0 - v0.7.2)
///
/// Holds a reference to the V1 layout struct to support potential future migrations
pub struct V1 {
    pub home: v1::VoltaHome,
}

impl V1 {
    pub fn new(home: PathBuf) -> Self {
        Self {
            home: v1::VoltaHome::new(home),
        }
    }

    /// Write the layout file to mark migration to V1 as complete
    ///
    /// Should only be called once all other migration steps are finished, so that we don't
    /// accidentally mark an incomplete migration as completed
    fn complete_migration(home: v1::VoltaHome) -> Fallible<Self> {
        debug!("Writing layout marker file");
        File::create(home.layout_file()).with_context(|| {
            ErrorKind::Filesystem(FilesystemError::CreateLayoutFile {
                file: home.layout_file().to_owned(),
            })
        })?;

        Ok(Self { home })
    }
}

impl TryFrom<Empty> for V1 {
    type Error = VoltaError;

    fn try_from(old: Empty) -> Fallible<Self> {
        debug!("New Volta installation detected, creating fresh layout");

        let home = v1::VoltaHome::new(old.home);
        home.create().with_context(|| {
            ErrorKind::Filesystem(FilesystemError::CreateDir {
                dir: home.root().to_owned(),
            })
        })?;

        Self::complete_migration(home)
    }
}

impl TryFrom<V0> for V1 {
    type Error = VoltaError;

    fn try_from(old: V0) -> Fallible<Self> {
        debug!("Existing Volta installation detected, migrating from V0 layout");

        let new_home = v1::VoltaHome::new(old.home.root().to_owned());
        new_home.create().with_context(|| {
            ErrorKind::Filesystem(FilesystemError::CreateDir {
                dir: new_home.root().to_owned(),
            })
        })?;

        #[cfg(unix)]
        {
            debug!("Removing unnecessary 'load.*' files");
            let root_contents = read_dir_eager(new_home.root()).with_context(|| {
                ErrorKind::Filesystem(FilesystemError::ReadDir {
                    dir: new_home.root().to_owned(),
                })
            })?;
            for (entry, _) in root_contents {
                let path = entry.path();
                if let Some(stem) = path.file_stem()
                    && stem == "load"
                    && path.is_file()
                {
                    remove_file(&path).with_context(|| {
                        ErrorKind::Filesystem(FilesystemError::DeleteFile { file: path })
                    })?;
                }
            }

            debug!("Removing old Volta binaries");

            let old_volta_bin = new_home.root().join("volta");
            remove_file_if_exists(old_volta_bin)?;

            let old_shim_bin = new_home.root().join("shim");
            remove_file_if_exists(old_shim_bin)?;
        }

        Self::complete_migration(new_home)
    }
}
