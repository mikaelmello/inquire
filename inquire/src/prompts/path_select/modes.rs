//! Prompt modes 
//! 
//! Everything here should be exposed to the (library) user
use strum::{Display, EnumCount, FromRepr};


/// Different path selection modes specify what the user can choose
#[derive(Clone, Eq, PartialEq)]
pub enum PathSelectionMode<'a> {
    /// The user may pick a directory.
    Directory,
    /// The user may pick a file with the given (optional) extension.
    File(Option<&'a str>),
    /// The user can set gitignore rules from a file 
    /// The user may pick multiple paths
    Multiple(Vec<PathSelectionMode<'a>>),
}

impl<'a> Default for PathSelectionMode<'a> {
    fn default() -> Self {
        Self::Directory
    }
}


/// Item sort options when displaying the list of files and directories.
#[derive(Copy, Clone, Eq, PartialEq, Display, EnumCount, FromRepr)]
#[strum(serialize_all = "lowercase")]
pub enum PathSortingMode {
    /// Sort by path according to the standard library implementation  
    Path,
    /// Sort by file size (directories listed first)
    Size,
    /// Sort by extension  
    Extension
}

impl Default for PathSortingMode {
    fn default() -> Self {
        Self::Path
    }
}

impl PathSortingMode {
    /// Get the next sorting mode
    pub (crate) fn next(self) -> Self {
        Self::from_repr((self as usize + 1) % Self::COUNT)
            .expect("must get next mode from enum representation")
    }
}