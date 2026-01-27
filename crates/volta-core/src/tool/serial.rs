use std::cmp::Ordering;

use super::ToolSpec;
use crate::error::{ErrorKind, Fallible};
use crate::version::{Tag, VersionSpec};
use once_cell::sync::Lazy;
use regex::Regex;
use validate_npm_package_name::{validate, Validity};

static TOOL_SPEC_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new("^(?P<name>(?:@([^/]+?)[/])?([^/]+?))(@(?P<version>.+))?$").expect("regex is valid")
});
static HAS_VERSION: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^\s]+@").expect("regex is valid"));

/// Methods for parsing a `ToolSpec` out of string values
impl ToolSpec {
    #[must_use]
    pub fn from_str_and_version(tool_name: &str, version: VersionSpec) -> Self {
        match tool_name {
            "node" => Self::Node(version),
            "npm" => Self::Npm(version),
            "pnpm" => Self::Pnpm(version),
            "yarn" => Self::Yarn(version),
            package => Self::Package(package.to_string(), version),
        }
    }

    /// Try to parse a tool and version from a string like `<tool>[@<version>]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool spec cannot be parsed.
    pub fn try_from_str(tool_spec: &str) -> Fallible<Self> {
        let captures =
            TOOL_SPEC_PATTERN
                .captures(tool_spec)
                .ok_or_else(|| ErrorKind::ParseToolSpecError {
                    tool_spec: tool_spec.into(),
                })?;

        // Validate that the captured name is a valid NPM package name.
        let name = &captures["name"];
        if let Validity::Invalid { errors, .. } = validate(name) {
            return Err(ErrorKind::InvalidToolName {
                name: name.into(),
                errors,
            }
            .into());
        }

        let version = captures
            .name("version")
            .map(|version| version.as_str().parse())
            .transpose()?
            .unwrap_or_default();

        Ok(match name {
            "node" => Self::Node(version),
            "npm" => Self::Npm(version),
            "pnpm" => Self::Pnpm(version),
            "yarn" => Self::Yarn(version),
            package => Self::Package(package.into(), version),
        })
    }

    /// Get a valid, sorted `Vec<ToolSpec>` given a `Vec<String>`.
    ///
    /// Accounts for the following error conditions:
    ///
    /// - `volta install node 12`, where the user intended to install `node@12`
    ///   but used syntax like in nodenv or nvm
    /// - invalid version specs
    ///
    /// Returns a listed sorted so that if `node` is included in the list, it is
    /// always first.
    ///
    /// # Errors
    ///
    /// Returns an error if any tool spec cannot be parsed.
    pub fn from_strings<T>(tool_strs: &[T], action: &str) -> Fallible<Vec<Self>>
    where
        T: AsRef<str>,
    {
        Self::check_args(tool_strs, action)?;

        let mut tools = tool_strs
            .iter()
            .map(|arg| Self::try_from_str(arg.as_ref()))
            .collect::<Fallible<Vec<Self>>>()?;

        tools.sort_by(Self::sort_comparator);
        Ok(tools)
    }

    /// Check the args for the bad patterns of
    /// - `volta install <number>`
    /// - `volta install <tool> <number>`
    fn check_args<T>(args: &[T], action: &str) -> Fallible<()>
    where
        T: AsRef<str>,
    {
        let mut args = args.iter();

        match (args.next(), args.next(), args.next()) {
            // The case we are concerned with here is where we have `<number>`.
            // That is, exactly one argument, which is a valid version specifier.
            //
            // - `volta install node@12` is allowed.
            // - `volta install 12` is an error.
            // - `volta install lts` is an error.
            (Some(maybe_version), None, None) if is_version_like(maybe_version.as_ref()) => {
                Err(ErrorKind::InvalidInvocationOfBareVersion {
                    action: action.to_string(),
                    version: maybe_version.as_ref().to_string(),
                }
                .into())
            }
            // The case we are concerned with here is where we have `<tool> <number>`.
            // This is only interesting if there are exactly two args. Then we care
            // whether the two items are a bare name (with no `@version`), followed
            // by a valid version specifier (ignoring custom tags). That is:
            //
            // - `volta install node@lts latest` is allowed.
            // - `volta install node latest` is an error.
            // - `volta install node latest yarn` is allowed.
            (Some(name), Some(maybe_version), None)
                if !HAS_VERSION.is_match(name.as_ref())
                    && is_version_like(maybe_version.as_ref()) =>
            {
                Err(ErrorKind::InvalidInvocation {
                    action: action.to_string(),
                    name: name.as_ref().to_string(),
                    version: maybe_version.as_ref().to_string(),
                }
                .into())
            }
            _ => Ok(()),
        }
    }

    /// Compare `ToolSpec`s for sorting when converting from strings
    ///
    /// We want to preserve the original order as much as possible, so we treat tools in
    /// the same tool category as equal. We still need to pull Node to the front of the
    /// list, followed by Npm, pnpm, Yarn, and then Packages last.
    const fn sort_comparator(left: &Self, right: &Self) -> Ordering {
        match (left, right) {
            (Self::Node(_), Self::Node(_))
            | (Self::Npm(_), Self::Npm(_))
            | (Self::Pnpm(_), Self::Pnpm(_))
            | (Self::Yarn(_), Self::Yarn(_))
            | (Self::Package(_, _), Self::Package(_, _)) => Ordering::Equal,
            (Self::Node(_), _) => Ordering::Less,
            (_, Self::Node(_)) => Ordering::Greater,
            (Self::Npm(_), _) => Ordering::Less,
            (_, Self::Npm(_)) => Ordering::Greater,
            (Self::Pnpm(_), _) => Ordering::Less,
            (_, Self::Pnpm(_)) => Ordering::Greater,
            (Self::Yarn(_), _) => Ordering::Less,
            (_, Self::Yarn(_)) => Ordering::Greater,
        }
    }
}

/// Determine if a given string is "version-like".
///
/// This means it is either 'latest', 'lts', a Version, or a Version Range.
fn is_version_like(value: &str) -> bool {
    matches!(
        value.parse(),
        Ok(VersionSpec::Exact(_)
            | VersionSpec::Semver(_)
            | VersionSpec::Tag(Tag::Latest | Tag::Lts))
    )
}

#[cfg(test)]
mod tests {
    mod try_from_str {
        use std::str::FromStr as _;

        use super::super::super::ToolSpec;
        use crate::version::{Tag, VersionSpec};

        const LTS: &str = "lts";
        const LATEST: &str = "latest";
        const MAJOR: &str = "3";
        const MINOR: &str = "3.0";
        const PATCH: &str = "3.0.0";
        const BETA: &str = "beta";

        /// Convenience macro for generating the <tool>@<version> string.
        macro_rules! versioned_tool {
            ($tool:expr, $version:expr) => {
                format!("{}@{}", $tool, $version)
            };
        }

        #[test]
        fn parses_bare_node() {
            assert_eq!(
                ToolSpec::try_from_str("node").expect("succeeds"),
                ToolSpec::Node(VersionSpec::default())
            );
        }

        #[test]
        fn parses_node_with_valid_versions() {
            let tool = "node";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MAJOR)).expect("succeeds"),
                ToolSpec::Node(
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MINOR)).expect("succeeds"),
                ToolSpec::Node(
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, PATCH)).expect("succeeds"),
                ToolSpec::Node(
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, LATEST)).expect("succeeds"),
                ToolSpec::Node(VersionSpec::Tag(Tag::Latest))
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, LTS)).expect("succeeds"),
                ToolSpec::Node(VersionSpec::Tag(Tag::Lts))
            );
        }

        #[test]
        fn parses_bare_yarn() {
            assert_eq!(
                ToolSpec::try_from_str("yarn").expect("succeeds"),
                ToolSpec::Yarn(VersionSpec::default())
            );
        }

        #[test]
        fn parses_yarn_with_valid_versions() {
            let tool = "yarn";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MAJOR)).expect("succeeds"),
                ToolSpec::Yarn(
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, MINOR)).expect("succeeds"),
                ToolSpec::Yarn(
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, PATCH)).expect("succeeds"),
                ToolSpec::Yarn(
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(tool, LATEST)).expect("succeeds"),
                ToolSpec::Yarn(VersionSpec::Tag(Tag::Latest))
            );
        }

        #[test]
        fn parses_bare_packages() {
            let package = "ember-cli";
            assert_eq!(
                ToolSpec::try_from_str(package).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::default())
            );
        }

        #[test]
        fn parses_namespaced_packages() {
            let package = "@types/lodash";
            assert_eq!(
                ToolSpec::try_from_str(package).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::default())
            );
        }

        #[test]
        fn parses_bare_packages_with_valid_versions() {
            let package = "something-awesome";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MAJOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MINOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, PATCH)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LATEST)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Tag(Tag::Latest))
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LTS)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Tag(Tag::Lts))
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, BETA)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Tag(Tag::Custom(BETA.into())))
            );
        }

        #[test]
        fn parses_namespaced_packages_with_valid_versions() {
            let package = "@something/awesome";

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MAJOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, MINOR)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, PATCH)).expect("succeeds"),
                ToolSpec::Package(
                    package.into(),
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LATEST)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Tag(Tag::Latest))
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, LTS)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Tag(Tag::Lts))
            );

            assert_eq!(
                ToolSpec::try_from_str(&versioned_tool!(package, BETA)).expect("succeeds"),
                ToolSpec::Package(package.into(), VersionSpec::Tag(Tag::Custom(BETA.into())))
            );
        }
    }

    mod from_strings {
        use super::super::*;
        use std::str::FromStr;

        static PIN: &str = "pin";

        #[test]
        fn special_cases_just_number() {
            let version = "1.2.3";
            let args: Vec<String> = vec![version.into()];

            let err = ToolSpec::from_strings(&args, PIN).unwrap_err();

            assert_eq!(
                err.kind(),
                &ErrorKind::InvalidInvocationOfBareVersion {
                    action: PIN.into(),
                    version: version.into()
                },
                "`volta <action> number` results in the correct error"
            );
        }

        #[test]
        fn special_cases_tool_space_number() {
            let name = "potato";
            let version = "1.2.3";
            let args: Vec<String> = vec![name.into(), version.into()];

            let err = ToolSpec::from_strings(&args, PIN).unwrap_err();

            assert_eq!(
                err.kind(),
                &ErrorKind::InvalidInvocation {
                    action: PIN.into(),
                    name: name.into(),
                    version: version.into()
                },
                "`volta <action> tool number` results in the correct error"
            );
        }

        #[test]
        fn leaves_other_scenarios_alone() {
            let empty: Vec<&str> = Vec::new();
            assert_eq!(
                ToolSpec::from_strings(&empty, PIN).expect("is ok").len(),
                empty.len(),
                "when there are no args"
            );

            let only_one = ["node".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&only_one, PIN).expect("is ok").len(),
                only_one.len(),
                "when there is only one arg"
            );

            let one_with_explicit_verson = ["10@latest".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&one_with_explicit_verson, PIN)
                    .expect("is ok")
                    .len(),
                only_one.len(),
                "when the sole arg is version-like but has an explicit version"
            );

            let two_but_unmistakable = ["12".to_owned(), "node".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&two_but_unmistakable, PIN)
                    .expect("is ok")
                    .len(),
                two_but_unmistakable.len(),
                "when there are two args but the order is not likely to be a mistake"
            );

            let two_but_valid_first = ["node@lts".to_owned(), "12".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&two_but_valid_first, PIN)
                    .expect("is ok")
                    .len(),
                two_but_valid_first.len(),
                "when there are two args but the first is a valid tool spec"
            );

            let more_than_two_tools = ["node".to_owned(), "12".to_owned(), "yarn".to_owned()];
            assert_eq!(
                ToolSpec::from_strings(&more_than_two_tools, PIN)
                    .expect("is ok")
                    .len(),
                more_than_two_tools.len(),
                "when there are more than two args"
            );
        }

        #[test]
        fn sorts_node_npm_yarn_to_front() {
            let multiple = [
                "ember-cli@3".to_owned(),
                "yarn".to_owned(),
                "npm@5".to_owned(),
                "node@latest".to_owned(),
            ];
            let expected = [
                ToolSpec::Node(VersionSpec::Tag(Tag::Latest)),
                ToolSpec::Npm(VersionSpec::from_str("5").expect("requirement is valid")),
                ToolSpec::Yarn(VersionSpec::default()),
                ToolSpec::Package(
                    "ember-cli".to_owned(),
                    VersionSpec::from_str("3").expect("requirement is valid"),
                ),
            ];
            assert_eq!(
                ToolSpec::from_strings(&multiple, PIN).expect("is ok"),
                expected
            );
        }

        #[test]
        fn keeps_package_order_unchanged() {
            let packages_with_node = ["typescript@latest", "ember-cli@3", "node@lts", "mocha"];
            let expected = [
                ToolSpec::Node(VersionSpec::Tag(Tag::Lts)),
                ToolSpec::Package("typescript".to_owned(), VersionSpec::Tag(Tag::Latest)),
                ToolSpec::Package(
                    "ember-cli".to_owned(),
                    VersionSpec::from_str("3").expect("requirement is valid"),
                ),
                ToolSpec::Package("mocha".to_owned(), VersionSpec::default()),
            ];

            assert_eq!(
                ToolSpec::from_strings(&packages_with_node, PIN).expect("is ok"),
                expected
            );
        }
    }
}
