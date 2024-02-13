//! Unified diff parsing/metadata extraction library for Rust
//!
//! # Examples
//!
//! ```
//! extern crate unidiff;
//!
//! use unidiff::PatchSet;
//!
//! fn main() {
//!     let diff_str = "diff --git a/added_file b/added_file
//! new file mode 100644
//! index 0000000..9b710f3
//! --- /dev/null
//! +++ b/added_file
//! @@ -0,0 +1,4 @@
//! +This was missing!
//! +Adding it now.
//! +
//! +Only for testing purposes.";
//!     let mut patch = PatchSet::new();
//!     patch.parse(diff_str).ok().expect("Error parsing diff");
//! }
//! ```
use lazy_static::lazy_static;

use std::error;
use std::fmt;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use regex::Regex;

lazy_static! {
    static ref RE_SOURCE_FILENAME: Regex = Regex::new(r"^--- (?P<filename>[^\t\n]+)(?:\t(?P<timestamp>[^\n]+))?").unwrap();
    static ref RE_TARGET_FILENAME: Regex = Regex::new(r"^\+\+\+ (?P<filename>[^\t\n]+)(?:\t(?P<timestamp>[^\n]+))?").unwrap();
    static ref RE_HUNK_HEADER: Regex = Regex::new(r"^@@ -(?P<source_start>\d+)(?:,(?P<source_length>\d+))? \+(?P<target_start>\d+)(?:,(?P<target_length>\d+))? @@[ ]?(?P<section_header>.*)").unwrap();
    static ref RE_HUNK_BODY_LINE: Regex = Regex::new(r"^(?P<line_type>[- \n\+\\]?)(?P<value>.*)").unwrap();
}

/// Diff line is added
pub const LINE_TYPE_ADDED: &'static str = "+";
/// Diff line is removed
pub const LINE_TYPE_REMOVED: &'static str = "-";
/// Diff line is context
pub const LINE_TYPE_CONTEXT: &'static str = " ";
/// Diff line is empty
pub const LINE_TYPE_EMPTY: &'static str = "\n";

/// Error type
#[derive(Debug, Clone)]
pub enum Error {
    /// Target without source
    TargetWithoutSource(String),
    /// Unexpected hunk found
    UnexpectedHunk(String),
    /// Hunk line expected
    ExpectLine(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::TargetWithoutSource(ref l) => write!(f, "Target without source: {}", l),
            Error::UnexpectedHunk(ref l) => write!(f, "Unexpected hunk found: {}", l),
            Error::ExpectLine(ref l) => write!(f, "Hunk line expected: {}", l),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::TargetWithoutSource(..) => "Target without source",
            Error::UnexpectedHunk(..) => "Unexpected hunk found",
            Error::ExpectLine(..) => "Hunk line expected",
        }
    }
}

/// `unidiff::parse` result type
pub type Result<T> = ::std::result::Result<T, Error>;

/// A diff line
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Line {
    /// Source file line number
    pub source_line_no: Option<usize>,
    /// Target file line number
    pub target_line_no: Option<usize>,
    /// Diff file line number
    pub diff_line_no: usize,
    /// Diff line type
    pub line_type: String,
    /// Diff line content value
    pub value: String,
}

impl Line {
    pub fn new<T: Into<String>>(value: T, line_type: T) -> Line {
        Line {
            source_line_no: Some(0usize),
            target_line_no: Some(0usize),
            diff_line_no: 0usize,
            line_type: line_type.into(),
            value: value.into(),
        }
    }

    /// Diff line type is added
    pub fn is_added(&self) -> bool {
        LINE_TYPE_ADDED == &self.line_type
    }

    /// Diff line type is removed
    pub fn is_removed(&self) -> bool {
        LINE_TYPE_REMOVED == &self.line_type
    }

    /// Diff line type is context
    pub fn is_context(&self) -> bool {
        LINE_TYPE_CONTEXT == &self.line_type
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.line_type, self.value)
    }
}

/// Each of the modified blocks of a file
///
/// You can iterate over it to get ``Line``s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hunk {
    /// Count of lines added
    added: usize,
    /// Count of lines removed
    removed: usize,
    /// Source file starting line number
    pub source_start: usize,
    /// Source file changes length
    pub source_length: usize,
    /// Target file starting line number
    pub target_start: usize,
    /// Target file changes length
    pub target_length: usize,
    /// Section header
    pub section_header: String,
    lines: Vec<Line>,
    source: Vec<String>,
    target: Vec<String>,
}

impl Hunk {
    pub fn new<T: Into<String>>(
        source_start: usize,
        source_length: usize,
        target_start: usize,
        target_length: usize,
        section_header: T,
    ) -> Hunk {
        Hunk {
            added: 0usize,
            removed: 0usize,
            source_start: source_start,
            source_length: source_length,
            target_start: target_start,
            target_length: target_length,
            section_header: section_header.into(),
            lines: vec![],
            source: vec![],
            target: vec![],
        }
    }

    /// Count of lines added
    pub fn added(&self) -> usize {
        self.added
    }

    /// Count of lines removed
    pub fn removed(&self) -> usize {
        self.removed
    }

    /// Is this hunk valid
    pub fn is_valid(&self) -> bool {
        self.source.len() == self.source_length && self.target.len() == self.target_length
    }

    /// Lines from source file
    pub fn source_lines(&self) -> Vec<Line> {
        self.lines
            .iter()
            .cloned()
            .filter(|l| l.is_context() || l.is_removed())
            .collect()
    }

    /// Lines from target file
    pub fn target_lines(&self) -> Vec<Line> {
        self.lines
            .iter()
            .cloned()
            .filter(|l| l.is_context() || l.is_added())
            .collect()
    }

    /// Append new line into hunk
    pub fn append(&mut self, line: Line) {
        if line.is_added() {
            self.added = self.added + 1;
            self.target
                .push(format!("{}{}", line.line_type, line.value));
        } else if line.is_removed() {
            self.removed = self.removed + 1;
            self.source
                .push(format!("{}{}", line.line_type, line.value));
        } else if line.is_context() {
            self.source
                .push(format!("{}{}", line.line_type, line.value));
            self.target
                .push(format!("{}{}", line.line_type, line.value));
        }
        self.lines.push(line);
    }

    /// Count of lines in this hunk
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Is this hunk empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Lines in this hunk
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn lines_mut(&mut self) -> &mut [Line] {
        &mut self.lines
    }
}

impl fmt::Display for Hunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = format!(
            "@@ -{},{} +{},{} @@ {}\n",
            self.source_start,
            self.source_length,
            self.target_start,
            self.target_length,
            self.section_header
        );
        let content = self
            .lines
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}{}", header, content)
    }
}

impl IntoIterator for Hunk {
    type Item = Line;
    type IntoIter = ::std::vec::IntoIter<Line>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

impl Index<usize> for Hunk {
    type Output = Line;

    fn index(&self, idx: usize) -> &Line {
        &self.lines[idx]
    }
}

impl IndexMut<usize> for Hunk {
    fn index_mut(&mut self, index: usize) -> &mut Line {
        &mut self.lines[index]
    }
}

/// Patch updated file, contains a list of Hunks
///
/// You can iterate over it to get ``Hunk``s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatchedFile {
    /// Source file name
    pub source_file: String,
    /// Source file timestamp
    pub source_timestamp: Option<String>,
    /// Target file name
    pub target_file: String,
    /// Target file timestamp
    pub target_timestamp: Option<String>,
    hunks: Vec<Hunk>,
}

impl PatchedFile {
    /// Initialize a new PatchedFile instance
    pub fn new<T: Into<String>>(source_file: T, target_file: T) -> PatchedFile {
        PatchedFile {
            source_file: source_file.into(),
            target_file: target_file.into(),
            source_timestamp: None,
            target_timestamp: None,
            hunks: vec![],
        }
    }

    /// Initialize a new PatchedFile instance with hunks
    pub fn with_hunks<T: Into<String>>(
        source_file: T,
        target_file: T,
        hunks: Vec<Hunk>,
    ) -> PatchedFile {
        PatchedFile {
            source_file: source_file.into(),
            target_file: target_file.into(),
            source_timestamp: None,
            target_timestamp: None,
            hunks: hunks,
        }
    }

    /// Patched file relative path
    pub fn path(&self) -> String {
        if self.source_file.starts_with("a/") && self.target_file.starts_with("b/") {
            return self.source_file[2..].to_owned();
        }
        if self.source_file.starts_with("a/") && "/dev/null" == &self.target_file {
            return self.source_file[2..].to_owned();
        }
        if self.target_file.starts_with("b/") && "/dev/null" == &self.source_file {
            return self.target_file[2..].to_owned();
        }
        self.source_file.clone()
    }

    /// Count of lines added
    pub fn added(&self) -> usize {
        self.hunks.iter().map(|h| h.added).fold(0, |acc, x| acc + x)
    }

    /// Count of lines removed
    pub fn removed(&self) -> usize {
        self.hunks
            .iter()
            .map(|h| h.removed)
            .fold(0, |acc, x| acc + x)
    }

    /// Is this file newly added
    pub fn is_added_file(&self) -> bool {
        self.hunks.len() == 1 && self.hunks[0].source_start == 0 && self.hunks[0].source_length == 0
    }

    /// Is this file removed
    pub fn is_removed_file(&self) -> bool {
        self.hunks.len() == 1 && self.hunks[0].target_start == 0 && self.hunks[0].target_length == 0
    }

    /// Is this file modified
    pub fn is_modified_file(&self) -> bool {
        !self.is_added_file() && !self.is_removed_file()
    }

    fn parse_hunk(&mut self, header: &str, diff: &[(usize, &str)]) -> Result<()> {
        let header_info = RE_HUNK_HEADER.captures(header).unwrap();
        let source_start = header_info
            .name("source_start")
            .map_or("0", |s| s.as_str())
            .parse::<usize>()
            .unwrap();
        let source_length = header_info
            .name("source_length")
            .map_or("1", |s| s.as_str())
            .parse::<usize>()
            .unwrap();
        let target_start = header_info
            .name("target_start")
            .map_or("0", |s| s.as_str())
            .parse::<usize>()
            .unwrap();
        let target_length = header_info
            .name("target_length")
            .map_or("1", |s| s.as_str())
            .parse::<usize>()
            .unwrap();
        let section_header = header_info
            .name("section_header")
            .map_or("", |s| s.as_str());
        let mut hunk = Hunk {
            added: 0usize,
            removed: 0usize,
            source: vec![],
            target: vec![],
            lines: vec![],
            source_start: source_start,
            source_length: source_length,
            target_start: target_start,
            target_length: target_length,
            section_header: section_header.to_owned(),
        };
        let mut source_line_no = source_start;
        let mut target_line_no = target_start;
        let expected_source_end = source_start + source_length;
        let expected_target_end = target_start + target_length;
        for &(diff_line_no, line) in diff {
            if let Some(valid_line) = RE_HUNK_BODY_LINE.captures(line) {
                let mut line_type = valid_line.name("line_type").unwrap().as_str();
                if line_type == LINE_TYPE_EMPTY || line_type == "" {
                    line_type = LINE_TYPE_CONTEXT;
                }
                let value = valid_line.name("value").unwrap().as_str();
                let mut original_line = Line {
                    source_line_no: None,
                    target_line_no: None,
                    diff_line_no: diff_line_no + 1,
                    line_type: line_type.to_owned(),
                    value: value.to_owned(),
                };
                match line_type {
                    LINE_TYPE_ADDED => {
                        original_line.target_line_no = Some(target_line_no);
                        target_line_no = target_line_no + 1;
                    }
                    LINE_TYPE_REMOVED => {
                        original_line.source_line_no = Some(source_line_no);
                        source_line_no = source_line_no + 1;
                    }
                    LINE_TYPE_CONTEXT => {
                        original_line.target_line_no = Some(target_line_no);
                        target_line_no = target_line_no + 1;
                        original_line.source_line_no = Some(source_line_no);
                        source_line_no = source_line_no + 1;
                    }
                    _ => {}
                }
                hunk.append(original_line);
                if source_line_no >= expected_source_end && target_line_no >= expected_target_end {
                    // FIXME: sync with upstream version
                    break;
                }
            } else {
                return Err(Error::ExpectLine(line.to_owned()));
            }
        }
        self.hunks.push(hunk);
        Ok(())
    }

    /// Count of hunks
    pub fn len(&self) -> usize {
        self.hunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }

    /// Hunks in this file
    pub fn hunks(&self) -> &[Hunk] {
        &self.hunks
    }

    pub fn hunks_mut(&mut self) -> &mut [Hunk] {
        &mut self.hunks
    }
}

impl fmt::Display for PatchedFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source = format!("--- {}\n", self.source_file);
        let target = format!("+++ {}\n", self.target_file);
        let hunks = self
            .hunks
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}{}{}", source, target, hunks)
    }
}

impl IntoIterator for PatchedFile {
    type Item = Hunk;
    type IntoIter = ::std::vec::IntoIter<Hunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.hunks.into_iter()
    }
}

impl Index<usize> for PatchedFile {
    type Output = Hunk;

    fn index(&self, idx: usize) -> &Hunk {
        &self.hunks[idx]
    }
}

impl IndexMut<usize> for PatchedFile {
    fn index_mut(&mut self, index: usize) -> &mut Hunk {
        &mut self.hunks[index]
    }
}

/// Unfied patchset
///
/// You can iterate over it to get ``PatchedFile``s.
///
/// ```ignore
/// let mut patch = PatchSet::new();
/// patch.parse("some diff");
/// for patched_file in patch {
///   // do something with patched_file
///   for hunk in patched_file {
///       // do something with hunk
///       for line in hunk {
///           // do something with line
///       }
///   }
/// }
/// ```
#[derive(Clone)]
pub struct PatchSet {
    files: Vec<PatchedFile>,
    #[cfg(feature = "encoding")]
    encoding: &'static encoding_rs::Encoding,
}

impl fmt::Debug for PatchSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("PatchSet")
            .field("files", &self.files)
            .finish()
    }
}

impl Default for PatchSet {
    fn default() -> PatchSet {
        PatchSet::new()
    }
}

impl PatchSet {
    /// Added files vector
    pub fn added_files(&self) -> Vec<PatchedFile> {
        self.files
            .iter()
            .cloned()
            .filter(|f| f.is_added_file())
            .collect()
    }

    /// Removed files vector
    pub fn removed_files(&self) -> Vec<PatchedFile> {
        self.files
            .iter()
            .cloned()
            .filter(|f| f.is_removed_file())
            .collect()
    }

    /// Modified files vector
    pub fn modified_files(&self) -> Vec<PatchedFile> {
        self.files
            .iter()
            .cloned()
            .filter(|f| f.is_modified_file())
            .collect()
    }

    /// Initialize a new PatchSet instance
    pub fn new() -> PatchSet {
        PatchSet {
            files: vec![],
            #[cfg(feature = "encoding")]
            encoding: encoding_rs::UTF_8,
        }
    }

    /// Initialize a new PatchedSet instance with encoding
    #[cfg(feature = "encoding")]
    pub fn with_encoding(coding: &'static encoding_rs::Encoding) -> PatchSet {
        PatchSet {
            files: vec![],
            encoding: coding,
        }
    }

    /// Initialize a new PatchedSet instance with encoding(string form)
    #[cfg(feature = "encoding")]
    pub fn from_encoding<T: AsRef<str>>(coding: T) -> PatchSet {
        let codec = encoding_rs::Encoding::for_label(coding.as_ref().as_bytes());
        PatchSet {
            files: vec![],
            encoding: codec.unwrap_or(encoding_rs::UTF_8),
        }
    }

    /// Parse diff from bytes
    #[cfg(feature = "encoding")]
    pub fn parse_bytes(&mut self, input: &[u8]) -> Result<()> {
        let input = self.encoding.decode(input).0.to_string();
        self.parse(input)
    }

    /// Parse diff from string
    pub fn parse<T: AsRef<str>>(&mut self, input: T) -> Result<()> {
        let input = input.as_ref();
        let mut current_file: Option<PatchedFile> = None;
        let diff: Vec<(usize, &str)> = input.lines().enumerate().collect();
        let mut source_file: Option<String> = None;
        let mut source_timestamp: Option<String> = None;

        for &(line_no, line) in &diff {
            // check for source file header
            if let Some(captures) = RE_SOURCE_FILENAME.captures(line) {
                source_file = match captures.name("filename") {
                    Some(ref filename) => Some(filename.as_str().to_owned()),
                    None => Some("".to_owned()),
                };
                source_timestamp = match captures.name("timestamp") {
                    Some(ref timestamp) => Some(timestamp.as_str().to_owned()),
                    None => Some("".to_owned()),
                };
                if let Some(patched_file) = current_file {
                    self.files.push(patched_file.clone());
                    current_file = None;
                }
                continue;
            }
            // check for target file header
            if let Some(captures) = RE_TARGET_FILENAME.captures(line) {
                if current_file.is_some() {
                    return Err(Error::TargetWithoutSource(line.to_owned()));
                }
                let target_file = match captures.name("filename") {
                    Some(ref filename) => Some(filename.as_str().to_owned()),
                    None => Some("".to_owned()),
                };
                let target_timestamp = match captures.name("timestamp") {
                    Some(ref timestamp) => Some(timestamp.as_str().to_owned()),
                    None => Some("".to_owned()),
                };

                // add current file to PatchSet
                current_file = Some(PatchedFile {
                    source_file: source_file.clone().unwrap(),
                    target_file: target_file.clone().unwrap(),
                    source_timestamp: source_timestamp.clone(),
                    target_timestamp: target_timestamp.clone(),
                    hunks: Vec::new(),
                });
                continue;
            }
            // check for hunk header
            if RE_HUNK_HEADER.is_match(line) {
                if let Some(ref mut patched_file) = current_file {
                    patched_file.parse_hunk(line, &diff[line_no + 1..])?;
                } else {
                    return Err(Error::UnexpectedHunk(line.to_owned()));
                }
            }
        }
        if let Some(patched_file) = current_file {
            self.files.push(patched_file.clone());
        }
        Ok(())
    }

    /// Count of patched files
    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Files in this patch set
    pub fn files(&self) -> &[PatchedFile] {
        &self.files
    }

    pub fn files_mut(&mut self) -> &mut [PatchedFile] {
        &mut self.files
    }
}

impl fmt::Display for PatchSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let diff = self
            .files
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", diff)
    }
}

impl IntoIterator for PatchSet {
    type Item = PatchedFile;
    type IntoIter = ::std::vec::IntoIter<PatchedFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

impl Index<usize> for PatchSet {
    type Output = PatchedFile;

    fn index(&self, idx: usize) -> &PatchedFile {
        &self.files[idx]
    }
}

impl IndexMut<usize> for PatchSet {
    fn index_mut(&mut self, index: usize) -> &mut PatchedFile {
        &mut self.files[index]
    }
}

impl FromStr for PatchSet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut patch = PatchSet::new();
        patch.parse(s)?;
        Ok(patch)
    }
}
