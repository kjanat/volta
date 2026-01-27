use std::fmt;
use std::process::Output;
use std::str;

use crate::process::Builder;

use hamcrest2::core::{MatchResult, Matcher};
use serde_json::{self, Value};

#[derive(Clone)]
pub struct Execs {
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: Option<i32>,
    stdout_contains: Vec<String>,
    stderr_contains: Vec<String>,
    either_contains: Vec<String>,
    stdout_contains_n: Vec<(String, usize)>,
    stdout_not_contains: Vec<String>,
    stderr_not_contains: Vec<String>,
    stderr_unordered: Vec<String>,
    neither_contains: Vec<String>,
    json: Option<Vec<Value>>,
}

impl Execs {
    /// Verify that stdout is equal to the given lines.
    ///
    /// See `lines_match` for supported patterns.
    #[must_use]
    pub fn with_stdout(mut self, expected: &str) -> Self {
        self.stdout = Some(expected.to_string());
        self
    }

    /// Verify that stderr is equal to the given lines.
    ///
    /// See `lines_match` for supported patterns.
    #[must_use]
    pub fn with_stderr(mut self, expected: &str) -> Self {
        self.stderr = Some(expected.to_string());
        self
    }

    /// Verify the exit code from the process.
    #[must_use]
    pub const fn with_status(mut self, expected: i32) -> Self {
        self.exit_code = Some(expected);
        self
    }

    /// Verify that stdout contains the given contiguous lines somewhere in
    /// its output.
    ///
    /// See `lines_match` for supported patterns.
    #[must_use]
    pub fn with_stdout_contains(mut self, expected: &str) -> Self {
        self.stdout_contains.push(expected.to_string());
        self
    }

    /// Verify that stderr contains the given contiguous lines somewhere in
    /// its output.
    ///
    /// See `lines_match` for supported patterns.
    #[must_use]
    pub fn with_stderr_contains(mut self, expected: &str) -> Self {
        self.stderr_contains.push(expected.to_string());
        self
    }

    /// Verify that either stdout or stderr contains the given contiguous
    /// lines somewhere in its output.
    ///
    /// See `lines_match` for supported patterns.
    #[must_use]
    pub fn with_either_contains(mut self, expected: &str) -> Self {
        self.either_contains.push(expected.to_string());
        self
    }

    /// Verify that stdout contains the given contiguous lines somewhere in
    /// its output, and should be repeated `number` times.
    ///
    /// See `lines_match` for supported patterns.
    #[must_use]
    pub fn with_stdout_contains_n(mut self, expected: &str, number: usize) -> Self {
        self.stdout_contains_n.push((expected.to_string(), number));
        self
    }

    /// Verify that stdout does not contain the given contiguous lines.
    ///
    /// See `lines_match` for supported patterns.
    /// See note on `with_stderr_does_not_contain`.
    #[must_use]
    pub fn with_stdout_does_not_contain(mut self, expected: &str) -> Self {
        self.stdout_not_contains.push(expected.to_string());
        self
    }

    /// Verify that stderr does not contain the given contiguous lines.
    ///
    /// See `lines_match` for supported patterns.
    ///
    /// Care should be taken when using this method because there is a
    /// limitless number of possible things that *won't* appear.  A typo means
    /// your test will pass without verifying the correct behavior. If
    /// possible, write the test first so that it fails, and then implement
    /// your fix/feature to make it pass.
    #[must_use]
    pub fn with_stderr_does_not_contain(mut self, expected: &str) -> Self {
        self.stderr_not_contains.push(expected.to_string());
        self
    }

    /// Verify that all of the stderr output is equal to the given lines,
    /// ignoring the order of the lines.
    ///
    /// See `lines_match` for supported patterns.
    /// This is useful when checking the output of `cargo build -v` since
    /// the order of the output is not always deterministic.
    /// Recommend use `with_stderr_contains` instead unless you really want to
    /// check *every* line of output.
    ///
    /// Be careful when using patterns such as `[..]`, because you may end up
    /// with multiple lines that might match, and this is not smart enough to
    /// do anything like longest-match.  For example, avoid something like:
    ///     `[RUNNING]` `rustc [..]`
    ///     `[RUNNING]` `rustc --crate-name foo [..]`
    /// This will randomly fail if the other crate name is `bar`, and the
    /// order changes.
    #[must_use]
    pub fn with_stderr_unordered(mut self, expected: &str) -> Self {
        self.stderr_unordered.push(expected.to_string());
        self
    }

    /// Verify the JSON output matches the given JSON.
    ///
    /// Typically used when testing cargo commands that emit JSON.
    /// Each separate JSON object should be separated by a blank line.
    /// Example:
    ///     `assert_that`(
    ///         p.cargo("metadata"),
    ///         `execs().with_json(r`#"
    ///             {"example": "abc"}
    ///             {"example": "def"}
    ///         "#)
    ///      );
    /// Objects should match in the order given.
    /// The order of arrays is ignored.
    /// Strings support patterns described in `lines_match`.
    /// Use `{...}` to match any object.
    ///
    /// # Panics
    ///
    /// Panics if `expected` contains invalid JSON.
    #[must_use]
    pub fn with_json(mut self, expected: &str) -> Self {
        self.json = Some(
            expected
                .split("\n\n")
                .map(|obj| obj.parse().unwrap())
                .collect(),
        );
        self
    }

    fn match_output(&self, actual: &Output) -> MatchResult {
        self.match_status(actual)
            .and_then(|()| self.match_stdout(actual))
            .and_then(|()| self.match_stderr(actual))
    }

    fn match_status(&self, actual: &Output) -> MatchResult {
        match self.exit_code {
            None => Ok(()),
            Some(code) if actual.status.code() == Some(code) => Ok(()),
            Some(_) => Err(format!(
                "exited with {}\n--- stdout\n{}\n--- stderr\n{}",
                actual.status,
                String::from_utf8_lossy(&actual.stdout),
                String::from_utf8_lossy(&actual.stderr)
            )),
        }
    }

    fn match_stdout(&self, actual: &Output) -> MatchResult {
        match_std(
            self.stdout.as_ref(),
            &actual.stdout,
            "stdout",
            &actual.stderr,
            MatchKind::Exact,
        )?;
        self.match_contains_checks(actual)?;
        self.match_not_contains_checks(actual)?;
        self.match_either_neither_checks(actual)?;
        self.match_json_output(actual)
    }

    fn match_contains_checks(&self, actual: &Output) -> MatchResult {
        for expect in &self.stdout_contains {
            match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stderr,
                MatchKind::Partial,
            )?;
        }
        for expect in &self.stderr_contains {
            match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stdout,
                MatchKind::Partial,
            )?;
        }
        for (expect, number) in &self.stdout_contains_n {
            match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stderr,
                MatchKind::PartialN(*number),
            )?;
        }
        for expect in &self.stderr_unordered {
            match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stdout,
                MatchKind::Unordered,
            )?;
        }
        Ok(())
    }

    fn match_not_contains_checks(&self, actual: &Output) -> MatchResult {
        for expect in &self.stdout_not_contains {
            match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stderr,
                MatchKind::NotPresent,
            )?;
        }
        for expect in &self.stderr_not_contains {
            match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stdout,
                MatchKind::NotPresent,
            )?;
        }
        Ok(())
    }

    fn match_either_neither_checks(&self, actual: &Output) -> MatchResult {
        for expect in &self.neither_contains {
            match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stdout,
                MatchKind::NotPresent,
            )?;
            match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stderr,
                MatchKind::NotPresent,
            )?;
        }
        for expect in &self.either_contains {
            let stdout_result = match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stdout,
                MatchKind::Partial,
            );
            let stderr_result = match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stderr,
                MatchKind::Partial,
            );
            if let (Err(_), Err(_)) = (stdout_result, stderr_result) {
                return Err(format!(
                    "expected to find:\n\
                     {expect}\n\n\
                     did not find in either output."
                ));
            }
        }
        Ok(())
    }

    fn match_json_output(&self, actual: &Output) -> MatchResult {
        let Some(ref objects) = self.json else {
            return Ok(());
        };
        let stdout =
            str::from_utf8(&actual.stdout).map_err(|_| "stdout was not utf8 encoded".to_owned())?;
        let lines = stdout
            .lines()
            .filter(|line| line.starts_with('{'))
            .collect::<Vec<_>>();
        if lines.len() != objects.len() {
            return Err(format!(
                "expected {} json lines, got {}, stdout:\n{}",
                objects.len(),
                lines.len(),
                stdout
            ));
        }
        for (obj, line) in objects.iter().zip(lines) {
            match_json(obj, line)?;
        }
        Ok(())
    }

    fn match_stderr(&self, actual: &Output) -> MatchResult {
        match_std(
            self.stderr.as_ref(),
            &actual.stderr,
            "stderr",
            &actual.stdout,
            MatchKind::Exact,
        )
    }
}

fn match_std(
    expected: Option<&String>,
    actual: &[u8],
    description: &str,
    extra: &[u8],
    kind: MatchKind,
) -> MatchResult {
    let Some(out) = expected else {
        return Ok(());
    };
    let Ok(actual) = str::from_utf8(actual) else {
        return Err(format!("{description} was not utf8 encoded"));
    };
    // Let's not deal with \r\n vs \n on windows...
    let actual = actual.replace('\r', "");
    let actual = actual.replace('\t', "<tab>");

    match kind {
        MatchKind::Exact => match_exact(out, &actual, extra),
        MatchKind::Partial => match_partial(out, &actual),
        MatchKind::PartialN(number) => match_partial_n(out, &actual, number),
        MatchKind::NotPresent => match_not_present(out, &actual),
        MatchKind::Unordered => match_unordered(out, &actual),
    }
}

fn match_exact(out: &str, actual: &str, extra: &[u8]) -> MatchResult {
    let a = actual.lines();
    let e = out.lines();
    let diffs = diff_lines(a, e, false);
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "differences:\n\
             {}\n\n\
             other output:\n\
             `{}`",
            diffs.join("\n"),
            String::from_utf8_lossy(extra)
        ))
    }
}

fn match_partial(out: &str, actual: &str) -> MatchResult {
    let mut a = actual.lines();
    let e = out.lines();
    let mut diffs = diff_lines(a.clone(), e.clone(), true);
    while a.next().is_some() {
        let new_diffs = diff_lines(a.clone(), e.clone(), true);
        if new_diffs.len() < diffs.len() {
            diffs = new_diffs;
        }
    }
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "expected to find:\n\
             {out}\n\n\
             did not find in output:\n\
             {actual}"
        ))
    }
}

fn match_partial_n(out: &str, actual: &str, number: usize) -> MatchResult {
    let mut a = actual.lines();
    let e = out.lines();
    let mut matches = 0;
    loop {
        if diff_lines(a.clone(), e.clone(), true).is_empty() {
            matches += 1;
        }
        if a.next().is_none() {
            break;
        }
    }
    if matches == number {
        Ok(())
    } else {
        Err(format!(
            "expected to find {number} occurrences:\n\
             {out}\n\n\
             did not find in output:\n\
             {actual}"
        ))
    }
}

fn match_not_present(out: &str, actual: &str) -> MatchResult {
    let mut a = actual.lines();
    let e = out.lines();
    let mut diffs = diff_lines(a.clone(), e.clone(), true);
    while a.next().is_some() {
        let new_diffs = diff_lines(a.clone(), e.clone(), true);
        if new_diffs.len() < diffs.len() {
            diffs = new_diffs;
        }
    }
    if diffs.is_empty() {
        Err(format!(
            "expected not to find:\n\
             {out}\n\n\
             but found in output:\n\
             {actual}"
        ))
    } else {
        Ok(())
    }
}

fn match_unordered(out: &str, actual: &str) -> MatchResult {
    let mut a = actual.lines().collect::<Vec<_>>();
    let e = out.lines();
    for e_line in e {
        match a.iter().position(|a_line| lines_match(e_line, a_line)) {
            Some(index) => {
                a.remove(index);
            }
            None => {
                return Err(format!(
                    "Did not find expected line:\n\
                     {}\n\
                     Remaining available output:\n\
                     {}\n",
                    e_line,
                    a.join("\n")
                ));
            }
        }
    }
    if a.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Output included extra lines:\n\
             {}\n",
            a.join("\n")
        ))
    }
}

fn match_json(expected: &Value, line: &str) -> MatchResult {
    let Ok(actual) = line.parse() else {
        return Err(format!("invalid json:\n`{line}`"));
    };

    match find_mismatch(expected, &actual) {
        Some((expected_part, actual_part)) => Err(format!(
            "JSON mismatch\nExpected:\n{}\nWas:\n{}\nExpected part:\n{}\nActual part:\n{}\n",
            serde_json::to_string_pretty(expected).unwrap(),
            serde_json::to_string_pretty(&actual).unwrap(),
            serde_json::to_string_pretty(expected_part).unwrap(),
            serde_json::to_string_pretty(actual_part).unwrap(),
        )),
        None => Ok(()),
    }
}

fn diff_lines<'a>(actual: str::Lines<'a>, expected: str::Lines<'a>, partial: bool) -> Vec<String> {
    let actual = actual.take(if partial {
        expected.clone().count()
    } else {
        usize::MAX
    });
    zip_all(actual, expected)
        .enumerate()
        .filter_map(|(i, (a, e))| match (a, e) {
            (Some(a), Some(e)) => {
                if lines_match(e, a) {
                    None
                } else {
                    Some(format!("{i:3} - |{e}|\n    + |{a}|\n"))
                }
            }
            (Some(a), None) => Some(format!("{i:3} -\n    + |{a}|\n")),
            (None, Some(e)) => Some(format!("{i:3} - |{e}|\n    +\n")),
            (None, None) => panic!("Cannot get here"),
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MatchKind {
    Exact,
    Partial,
    PartialN(usize),
    NotPresent,
    Unordered,
}

/// Compare a line with an expected pattern.
/// - Use `[..]` as a wildcard to match 0 or more characters on the same line
///   (similar to `.*` in a regex).
/// - Use `[EXE]` to optionally add `.exe` on Windows (empty string on other
///   platforms).
/// - There is a wide range of macros (such as `[COMPILING]` or `[WARNING]`)
///   to match cargo's "status" output and allows you to ignore the alignment.
///   See `substitute_macros` for a complete list of macros.
#[must_use]
pub fn lines_match(expected: &str, actual: &str) -> bool {
    // Let's not deal with / vs \ (windows...)
    let expected = expected.replace('\\', "/");
    let mut actual: &str = &actual.replace('\\', "/");
    let expected = substitute_macros(&expected);
    for (i, part) in expected.split("[..]").enumerate() {
        match actual.find(part) {
            Some(j) => {
                if i == 0 && j != 0 {
                    return false;
                }
                actual = &actual[j + part.len()..];
            }
            None => return false,
        }
    }
    actual.is_empty() || expected.ends_with("[..]")
}

#[test]
fn lines_match_works() {
    assert!(lines_match("a b", "a b"));
    assert!(lines_match("a[..]b", "a b"));
    assert!(lines_match("a[..]", "a b"));
    assert!(lines_match("[..]", "a b"));
    assert!(lines_match("[..]b", "a b"));

    assert!(!lines_match("[..]b", "c"));
    assert!(!lines_match("b", "c"));
    assert!(!lines_match("b", "cb"));
}

// Compares JSON object for approximate equality.
// You can use `[..]` wildcard in strings (useful for OS dependent things such
// as paths).  You can use a `"{...}"` string literal as a wildcard for
// arbitrary nested JSON (useful for parts of object emitted by other programs
// (e.g. rustc) rather than Cargo itself).  Arrays are sorted before comparison.
fn find_mismatch<'a>(expected: &'a Value, actual: &'a Value) -> Option<(&'a Value, &'a Value)> {
    use serde_json::Value::{Array, Bool, Null, Number, Object, String};
    match (expected, actual) {
        (Number(l), Number(r)) if l == r => None,
        (Bool(l), Bool(r)) if l == r => None,
        (String(l), String(r)) if lines_match(l, r) => None,
        (Array(l), Array(r)) => {
            if l.len() != r.len() {
                return Some((expected, actual));
            }

            let mut l = l.iter().collect::<Vec<_>>();
            let mut r = r.iter().collect::<Vec<_>>();

            l.retain(|l| {
                r.iter()
                    .position(|r| find_mismatch(l, r).is_none())
                    .is_none_or(|i| {
                        r.remove(i);
                        false
                    })
            });

            if l.is_empty() {
                assert_eq!(r.len(), 0);
                None
            } else {
                assert!(!r.is_empty());
                Some((l[0], r[0]))
            }
        }
        (Object(l), Object(r)) => {
            let same_keys = l.len() == r.len() && l.keys().all(|k| r.contains_key(k));
            if !same_keys {
                return Some((expected, actual));
            }

            l.values()
                .zip(r.values())
                .find_map(|(l, r)| find_mismatch(l, r))
        }
        (Null, Null) => None,
        // magic string literal "{...}" acts as wildcard for any sub-JSON
        (String(l), _) if l == "{...}" => None,
        _ => Some((expected, actual)),
    }
}

struct ZipAll<I1: Iterator, I2: Iterator> {
    first: I1,
    second: I2,
}

impl<T, I1: Iterator<Item = T>, I2: Iterator<Item = T>> Iterator for ZipAll<I1, I2> {
    type Item = (Option<T>, Option<T>);
    fn next(&mut self) -> Option<(Option<T>, Option<T>)> {
        let first = self.first.next();
        let second = self.second.next();

        match (first, second) {
            (None, None) => None,
            (a, b) => Some((a, b)),
        }
    }
}

const fn zip_all<T, I1: Iterator<Item = T>, I2: Iterator<Item = T>>(
    a: I1,
    b: I2,
) -> ZipAll<I1, I2> {
    ZipAll {
        first: a,
        second: b,
    }
}

impl fmt::Display for Execs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "execs")
    }
}

impl fmt::Debug for Execs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "execs")
    }
}

impl Matcher<Builder> for Execs {
    fn matches(&self, mut process: Builder) -> MatchResult {
        self.matches(&mut process)
    }
}

impl<'a> Matcher<&'a mut Builder> for Execs {
    fn matches(&self, process: &'a mut Builder) -> MatchResult {
        println!("running {process}");
        let res = process.exec_with_output();

        match res {
            Ok(out) => self.match_output(&out),
            Err(err) => {
                if let Some(out) = &err.output {
                    return self.match_output(out);
                }
                Err(format!("could not exec process {process}: {err}"))
            }
        }
    }
}

impl Matcher<Output> for Execs {
    fn matches(&self, output: Output) -> MatchResult {
        self.match_output(&output)
    }
}

#[must_use]
pub const fn execs() -> Execs {
    Execs {
        stdout: None,
        stderr: None,
        exit_code: Some(0),
        stdout_contains: Vec::new(),
        stderr_contains: Vec::new(),
        either_contains: Vec::new(),
        stdout_contains_n: Vec::new(),
        stdout_not_contains: Vec::new(),
        stderr_not_contains: Vec::new(),
        stderr_unordered: Vec::new(),
        neither_contains: Vec::new(),
        json: None,
    }
}

fn substitute_macros(input: &str) -> String {
    let macros = [
        ("[RUNNING]", "     Running"),
        ("[COMPILING]", "   Compiling"),
        ("[CHECKING]", "    Checking"),
        ("[CREATED]", "     Created"),
        ("[FINISHED]", "    Finished"),
        ("[ERROR]", "error:"),
        ("[WARNING]", "warning:"),
        ("[DOCUMENTING]", " Documenting"),
        ("[FRESH]", "       Fresh"),
        ("[UPDATING]", "    Updating"),
        ("[ADDING]", "      Adding"),
        ("[REMOVING]", "    Removing"),
        ("[DOCTEST]", "   Doc-tests"),
        ("[PACKAGING]", "   Packaging"),
        ("[DOWNLOADING]", " Downloading"),
        ("[UPLOADING]", "   Uploading"),
        ("[VERIFYING]", "   Verifying"),
        ("[ARCHIVING]", "   Archiving"),
        ("[INSTALLING]", "  Installing"),
        ("[REPLACING]", "   Replacing"),
        ("[UNPACKING]", "   Unpacking"),
        ("[SUMMARY]", "     Summary"),
        ("[FIXING]", "      Fixing"),
        ("[EXE]", if cfg!(windows) { ".exe" } else { "" }),
    ];
    let mut result = input.to_owned();
    for &(pat, subst) in &macros {
        result = result.replace(pat, subst);
    }
    result
}
