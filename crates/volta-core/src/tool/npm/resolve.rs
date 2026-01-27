//! Provides resolution of npm Version requirements into specific versions

use super::super::registry::{
    PackageDetails, PackageIndex, fetch_npm_registry, public_registry_index,
};
use crate::error::{ErrorKind, Fallible};
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::tool::Npm;
use crate::version::{Tag, VersionSpec};
use log::debug;
use nodejs_semver::{Range, Version};

/// # Errors
///
/// Returns an error if the version cannot be resolved.
pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Option<Version>> {
    let hooks = session.hooks()?.npm();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(&requirement, hooks).map(Some),
        VersionSpec::Exact(version) => Ok(Some(version)),
        VersionSpec::None | VersionSpec::Tag(Tag::Latest) => resolve_tag("latest", hooks).map(Some),
        VersionSpec::Tag(Tag::Custom(tag)) if tag == "bundled" => Ok(None),
        VersionSpec::Tag(tag) => resolve_tag(&tag.to_string(), hooks).map(Some),
    }
}

fn fetch_npm_index(hooks: Option<&ToolHooks<Npm>>) -> Fallible<(String, PackageIndex)> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using npm.index hook to determine npm index URL");
            hook.resolve("npm")?
        }
        _ => public_registry_index("npm"),
    };

    fetch_npm_registry(url, "npm")
}

fn resolve_tag(tag: &str, hooks: Option<&ToolHooks<Npm>>) -> Fallible<Version> {
    let (url, mut index) = fetch_npm_index(hooks)?;

    index.tags.remove(tag).map_or_else(
        || {
            Err(ErrorKind::NpmVersionNotFound {
                matching: tag.into(),
            }
            .into())
        },
        |version| {
            debug!("Found npm@{version} matching tag '{tag}' from {url}");
            Ok(version)
        },
    )
}

fn resolve_semver(matching: &Range, hooks: Option<&ToolHooks<Npm>>) -> Fallible<Version> {
    let (url, index) = fetch_npm_index(hooks)?;

    let details_opt = index
        .entries
        .into_iter()
        .find(|PackageDetails { version, .. }| matching.satisfies(version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found npm@{} matching requirement '{}' from {}",
                details.version, matching, url
            );
            Ok(details.version)
        }
        None => Err(ErrorKind::NpmVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}
