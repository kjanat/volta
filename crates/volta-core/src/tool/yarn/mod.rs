use std::fmt::{self, Display};

use super::{
    FetchStatus, Fetchable, Installable, Pinnable, check_fetched, check_shim_reachable,
    debug_already_fetched, info_fetched, info_installed, info_pinned, info_project_version,
};
use crate::error::{ErrorKind, Fallible, PlatformError};
use crate::inventory::yarn_available;
use crate::session::Session;
use crate::style::tool_version;
use crate::sync::VoltaLock;
use nodejs_semver::Version;

mod fetch;
mod metadata;
mod resolve;

pub use resolve::resolve;

/// The Tool implementation for fetching and installing Yarn
pub struct Yarn {
    pub(super) version: Version,
}

impl Yarn {
    #[must_use]
    pub const fn new(version: Version) -> Self {
        Self { version }
    }

    #[must_use]
    pub fn archive_basename(version: &str) -> String {
        format!("yarn-v{version}")
    }

    #[must_use]
    pub fn archive_filename(version: &str) -> String {
        format!("{}.tar.gz", Self::archive_basename(version))
    }

    pub(crate) fn ensure_fetched(&self, session: &Session) -> Fallible<()> {
        match check_fetched(|| yarn_available(&self.version))? {
            FetchStatus::AlreadyFetched => {
                debug_already_fetched(self);
                Ok(())
            }
            FetchStatus::FetchNeeded(_lock) => fetch::fetch(&self.version, session.hooks()?.yarn()),
        }
    }
}

impl Fetchable for Yarn {
    fn fetch(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        info_fetched(self);
        Ok(())
    }
}

impl Installable for Yarn {
    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        // Acquire a lock on the Volta directory, if possible, to prevent concurrent changes
        let _lock = VoltaLock::acquire();
        self.ensure_fetched(session)?;

        session
            .toolchain_mut()?
            .set_active_yarn(Some(self.version.clone()))?;

        info_installed(&self);
        check_shim_reachable("yarn");

        if let Ok(Some(project)) = session.project_platform()
            && let Some(yarn) = &project.yarn
        {
            info_project_version(tool_version("yarn", yarn), &self);
        }
        Ok(())
    }
}

impl Pinnable for Yarn {
    fn pin(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        if session.project()?.is_some() {
            self.ensure_fetched(session)?;

            // Note: We know this will succeed, since we checked above
            let project = session.project_mut()?.unwrap();
            project.pin_yarn(Some(self.version.clone()))?;

            info_pinned(self);
            Ok(())
        } else {
            Err(ErrorKind::Platform(PlatformError::NotInPackage).into())
        }
    }
}

impl Display for Yarn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("yarn", &self.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yarn_archive_basename() {
        assert_eq!(Yarn::archive_basename("1.2.3"), "yarn-v1.2.3");
    }

    #[test]
    fn test_yarn_archive_filename() {
        assert_eq!(Yarn::archive_filename("1.2.3"), "yarn-v1.2.3.tar.gz");
    }
}
