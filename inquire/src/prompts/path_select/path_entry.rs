//! Path entries
use super::PathSelectionMode;
use crate::InquireError;
use std::{
  convert::TryFrom,
  ffi::OsStr,
  fmt, fs,
  ops::Deref,
  path::{Path, PathBuf}
};

/// A path with cached information
#[derive(Clone, Debug, Hash)]
pub struct PathEntry {
    /// The (owned) [path](PathBuf)
    ///
    /// Corresponds to the target if this is a symlink  
    pub path: PathBuf,
    /// The [file type](fs::FileType)
    pub file_type: fs::FileType,
    /// The original symlink path
    pub symlink_path_opt: Option<PathBuf>,
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
        if let Some(symlink_path) = self.symlink_path_opt.as_ref() {
            write!(f, "{} -> {path}", symlink_path.to_string_lossy())
        } else if self.is_dir() {
            write!(f, "(dir) {path}")
        } else {
            write!(f, "{path}")
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
                Ok(Self {
                    path,
                    file_type: target_metadata.file_type(),
                    symlink_path_opt,
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
}
