//! Provides resolution of Node requirements into specific versions, using the `NodeJS` index

use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

use super::super::registry_fetch_error;
use super::metadata::{NodeEntry, NodeIndex, RawNodeIndex};
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::{create_staging_file, read_file};
use crate::hook::ToolHooks;
use crate::layout::volta_home;
use crate::session::Session;
use crate::style::progress_spinner;
use crate::tool::Node;
use crate::version::{Tag, VersionSpec};
use attohttpc::header::HeaderMap;
use attohttpc::Response;
use cfg_if::cfg_if;
use fs_utils::ensure_containing_dir_exists;
use headers::{CacheControl, Expires, HeaderMapExt};
use log::debug;
use nodejs_semver::{Range, Version};

// ISSUE (#86): Move public repository URLs to config file
cfg_if! {
    if #[cfg(feature = "mock-network")] {
        // TODO: We need to reconsider our mocking strategy in light of mockito deprecating the
        // SERVER_URL constant: Since our acceptance tests run the binary in a separate process,
        // we can't use `mockito::server_url()`, which relies on shared memory.
        #[allow(deprecated)]
        const SERVER_URL: &str = mockito::SERVER_URL;
        fn public_node_version_index() -> String {
            format!("{}/node-dist/index.json", SERVER_URL)
        }
    } else {
        /// Returns the URL of the index of available Node versions on the public Node server.
        fn public_node_version_index() -> String {
            "https://nodejs.org/dist/index.json".to_string()
        }
    }
}

/// # Errors
///
/// Returns an error if the version cannot be resolved.
pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.node();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(&requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None | VersionSpec::Tag(Tag::Lts) => resolve_lts(hooks),
        VersionSpec::Tag(Tag::Latest) => resolve_latest(hooks),
        // Node doesn't have "tagged" versions (apart from 'latest' and 'lts'), so custom tags will always be an error
        VersionSpec::Tag(Tag::Custom(tag)) => {
            Err(ErrorKind::NodeVersionNotFound { matching: tag }.into())
        }
    }
}

fn resolve_latest(hooks: Option<&ToolHooks<Node>>) -> Fallible<Version> {
    // NOTE: This assumes the registry always produces a list in sorted order
    //       from newest to oldest. This should be specified as a requirement
    //       when we document the plugin API.
    let url = match hooks {
        Some(&ToolHooks {
            latest: Some(ref hook),
            ..
        }) => {
            debug!("Using node.latest hook to determine node index URL");
            hook.resolve("index.json")?
        }
        _ => public_node_version_index(),
    };
    let version_opt = match_node_version(&url, |_| true)?;

    version_opt.map_or_else(
        || {
            Err(ErrorKind::NodeVersionNotFound {
                matching: "latest".into(),
            }
            .into())
        },
        |version| {
            debug!("Found latest node version ({version}) from {url}");
            Ok(version)
        },
    )
}

fn resolve_lts(hooks: Option<&ToolHooks<Node>>) -> Fallible<Version> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using node.index hook to determine node index URL");
            hook.resolve("index.json")?
        }
        _ => public_node_version_index(),
    };
    let version_opt = match_node_version(&url, |&NodeEntry { lts, .. }| lts)?;

    version_opt.map_or_else(
        || {
            Err(ErrorKind::NodeVersionNotFound {
                matching: "lts".into(),
            }
            .into())
        },
        |version| {
            debug!("Found newest LTS node version ({version}) from {url}");
            Ok(version)
        },
    )
}

fn resolve_semver(matching: &Range, hooks: Option<&ToolHooks<Node>>) -> Fallible<Version> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using node.index hook to determine node index URL");
            hook.resolve("index.json")?
        }
        _ => public_node_version_index(),
    };
    let version_opt = match_node_version(&url, |NodeEntry { version, .. }| {
        matching.satisfies(version)
    })?;

    version_opt.map_or_else(
        || {
            Err(ErrorKind::NodeVersionNotFound {
                matching: matching.to_string(),
            }
            .into())
        },
        |version| {
            debug!("Found node@{version} matching requirement '{matching}' from {url}");
            Ok(version)
        },
    )
}

fn match_node_version(
    url: &str,
    predicate: impl Fn(&NodeEntry) -> bool,
) -> Fallible<Option<Version>> {
    let index: NodeIndex = resolve_node_versions(url)?.into();
    let mut entries = index.entries.into_iter();
    Ok(entries
        .find(predicate)
        .map(|NodeEntry { version, .. }| version))
}

/// Reads a public index from the Node cache, if it exists and hasn't expired.
fn read_cached_opt(url: &str) -> Fallible<Option<RawNodeIndex>> {
    let expiry_file = volta_home()?.node_index_expiry_file();
    let expiry = read_file(expiry_file).with_context(|| ErrorKind::ReadNodeIndexExpiryError {
        file: expiry_file.to_owned(),
    })?;

    if expiry
        .map(|date| httpdate::parse_http_date(&date))
        .transpose()
        .with_context(|| ErrorKind::ParseNodeIndexExpiryError)?
        .is_none_or(|expiry_date| SystemTime::now() >= expiry_date)
    {
        return Ok(None);
    }

    let index_file = volta_home()?.node_index_file();
    let cached = read_file(index_file).with_context(|| ErrorKind::ReadNodeIndexCacheError {
        file: index_file.to_owned(),
    })?;

    let Some(json) = cached
        .as_ref()
        .and_then(|content| content.strip_prefix(url))
    else {
        return Ok(None);
    };

    serde_json::de::from_str(json).with_context(|| ErrorKind::ParseNodeIndexCacheError)
}

/// Get the cache max-age of an HTTP response.
fn max_age(headers: &HeaderMap) -> Duration {
    const FOUR_HOURS: Duration = Duration::from_secs(4 * 60 * 60);
    headers
        .typed_get::<CacheControl>()
        .and_then(|cache_control| cache_control.max_age())
        .unwrap_or(FOUR_HOURS)
}

fn resolve_node_versions(url: &str) -> Fallible<RawNodeIndex> {
    if let Some(serial) = read_cached_opt(url)? {
        debug!("Found valid cache of Node version index");
        Ok(serial)
    } else {
        debug!("Node index cache was not found or was invalid");
        let spinner = progress_spinner(format!("Fetching public registry: {url}"));

        let (_, headers, response) = attohttpc::get(url)
            .send()
            .and_then(Response::error_for_status)
            .with_context(registry_fetch_error("Node", url))?
            .split();

        let expires = headers
            .typed_get::<Expires>()
            .map_or_else(|| SystemTime::now() + max_age(&headers), SystemTime::from);

        let response_text = response
            .text()
            .with_context(registry_fetch_error("Node", url))?;

        let index: RawNodeIndex = serde_json::de::from_str(&response_text).with_context(|| {
            ErrorKind::ParseNodeIndexError {
                from_url: url.to_string(),
            }
        })?;

        let cached = create_staging_file()?;

        let mut cached_file: &File = cached.as_file();
        writeln!(cached_file, "{url}")
            .and_then(|()| cached_file.write(response_text.as_bytes()))
            .with_context(|| ErrorKind::WriteNodeIndexCacheError {
                file: cached.path().to_path_buf(),
            })?;

        let index_cache_file = volta_home()?.node_index_file();
        ensure_containing_dir_exists(&index_cache_file).with_context(|| {
            ErrorKind::ContainingDirError {
                path: index_cache_file.to_owned(),
            }
        })?;
        cached
            .persist(index_cache_file)
            .with_context(|| ErrorKind::WriteNodeIndexCacheError {
                file: index_cache_file.to_owned(),
            })?;

        let expiry = create_staging_file()?;
        let mut expiry_file: &File = expiry.as_file();

        write!(expiry_file, "{}", httpdate::fmt_http_date(expires)).with_context(|| {
            ErrorKind::WriteNodeIndexExpiryError {
                file: expiry.path().to_path_buf(),
            }
        })?;

        let index_expiry_file = volta_home()?.node_index_expiry_file();
        ensure_containing_dir_exists(&index_expiry_file).with_context(|| {
            ErrorKind::ContainingDirError {
                path: index_expiry_file.to_owned(),
            }
        })?;
        expiry.persist(index_expiry_file).with_context(|| {
            ErrorKind::WriteNodeIndexExpiryError {
                file: index_expiry_file.to_owned(),
            }
        })?;

        spinner.finish_and_clear();
        Ok(index)
    }
}
