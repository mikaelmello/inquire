//! Path entries
use super::PathSelectionMode;
use crate::{InquireError, SortingMode};
use std::{
    cmp, 
    convert::TryFrom,
    ffi::OsStr,
    fmt, fs,
    ops::Deref,
    path::{Path, PathBuf}
};

/// A path with cached information
#[derive(Clone, Debug, Hash)]
pub struct PathEntry {
    /// The (owned) [path](PathBuf).
    ///
    /// Corresponds to the target if this is a symlink.  
    pub path: PathBuf,
    /// The [file type](fs::FileType)
    pub file_type: fs::FileType,
    /// The original symlink path.
    pub symlink_path_opt: Option<PathBuf>,
    /// The file size in bytes.
    /// 
    /// Corresponds to the target file size if this is a symlink.
    pub size: u64,
}

impl Eq for PathEntry {}

impl PartialEq for PathEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl AsRef<Path> for PathEntry {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl Deref for PathEntry {
    type Target = fs::FileType;
    fn deref(&self) -> &Self::Target {
        &self.file_type
    }
}

impl fmt::Display for PathEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.to_string_lossy();
        use humansize::FormatSize;
        let size_formatting = if cfg!(windows) {
            humansize::WINDOWS
        } else {
            humansize::DECIMAL
        };
        let size_string = self.size.format_size(size_formatting); 
        if let Some(symlink_path) = self.symlink_path_opt.as_ref() {
            write!(f, "{} -> {path} ({size_string})", symlink_path.to_string_lossy())
        } else if self.is_dir() {
            write!(f, "(dir) {path} ({size_string})")
        } else {
            write!(f, "{path} ({size_string})")
        }
    }
}

impl TryFrom<&Path> for PathEntry {
    type Error = InquireError;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let is_symlink = value.is_symlink();
        value
            .metadata()
            .map_err(Self::Error::from)
            .and_then(|target_metadata| {
                let (path, symlink_path_opt) = if is_symlink {
                    (fs_err::canonicalize(value)?, Some(value.to_path_buf()))
                } else {
                    (value.to_path_buf(), None)
                };
                let size = if target_metadata.is_dir() {
                    0
                } else {
                    target_metadata.len()
                };
                Ok(Self {
                    path,
                    file_type: target_metadata.file_type(),
                    symlink_path_opt,
                    size,
                })
            })
    }
}

impl TryFrom<fs::DirEntry> for PathEntry {
    type Error = InquireError;
    fn try_from(value: fs::DirEntry) -> Result<Self, Self::Error> {
        Self::try_from(value.path().as_path())
    }
}

impl TryFrom<fs_err::DirEntry> for PathEntry {
    type Error = InquireError;
    fn try_from(value: fs_err::DirEntry) -> Result<Self, Self::Error> {
        Self::try_from(value.path().as_path())
    }
}

impl PathEntry {
    /// Is this path entry selectable based on the given selection mode?
    pub fn is_selectable<'a>(&self, selection_mode: &PathSelectionMode<'a>) -> bool {
        let is_dir = self.is_dir();
        let is_file = self.is_file();
        let file_ext_opt = self.path.extension().map(OsStr::to_os_string);
        match (selection_mode, is_dir, is_file) {
            (PathSelectionMode::Directory, true, _) => true,
            (PathSelectionMode::File(None), _, true) => true,
            (PathSelectionMode::File(Some(extension)), _, true) => file_ext_opt
                .as_ref()
                .map(|osstr| osstr.to_string_lossy().eq_ignore_ascii_case(*extension))
                .unwrap_or_default(),
            (PathSelectionMode::Multiple(ref path_selection_modes), _, _) => path_selection_modes
                .iter()
                .any(|submode| self.is_selectable(submode)),
            _ => false,
        }
    }

    /// Is this path entry for a symlink?
    pub fn is_symlink(&self) -> bool {
        self.symlink_path_opt.is_some()
    }

    /// Sort by the given sorting mode 
    pub fn sort_by_mode(a: &Self, b: &Self, sorting_mode: SortingMode) -> cmp::Ordering {
        match sorting_mode {
            SortingMode::Path => a.path.partial_cmp(&b.path).unwrap(),
            SortingMode::Size => a.size.cmp(&b.size),
            SortingMode::Extension => {
                match (a.is_dir(), b.is_dir()) {
                    (true, true) => Self::sort_by_mode(a, b, SortingMode::Path),
                    (true, false) => cmp::Ordering::Less,
                    (false, true) => cmp::Ordering::Greater,
                    (false, false) => {
                        a.path
                        .extension()
                        .unwrap_or_default()
                        .cmp(&b.path
                            .extension()
                            .unwrap_or_default()
                        )
                    }
                }
            }  
        }
    }
}
