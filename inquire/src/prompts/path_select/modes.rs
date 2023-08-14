//! Prompt modes 
//! 
//! Everything here should be exposed to the (library) user
use std::path::Path;

/// [PathSelectionMode] filters 
/// 
/// For examples, see [PathFilter::check].
#[derive(Clone, Eq, PartialEq, PartialOrd)]
pub enum PathFilter<'a> {
    /// Accept all path entries
    All,
    /// Accept path entries with precisely the given extension
    AcceptExtension(&'a str),
    /// Accept path entries with precisely the given stem (non-extension portion)
    AcceptStem(&'a str),
    /// Accept path entries for which the filter function returns true 
    AcceptMatching(fn(p: &Path) -> bool),
    /// Deny path entries with precisely the given extension
    DenyExtension(&'a str),
    /// Deny path entries with precisely the given stem (non-extension portion)
    DenyStem(&'a str),
    /// Accept path entries for which the filter function returns true 
    DenyMatching(fn(p: &Path) -> bool),
    /// Accept path entries matching ANY (first match) of the nested filters 
    AcceptAny(Vec<Self>),
    /// Accept path entries matching ALL of the nested filters 
    AcceptAll(Vec<Self>),
}

impl<'a> Default for PathFilter<'a> {
    fn default() -> Self {
        Self::All
    }
}

impl<'a> PathFilter<'a> {
    /// Check whether the given path passes this filter 
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::path::Path;
    /// use inquire::PathFilter;
    /// 
    /// assert_eq!(true, PathFilter::All.check(Path::new("trachea.stl")));
    /// assert_eq!(true, PathFilter::All.check(Path::new("heartbeat.mp3")));
    /// assert_eq!(true, PathFilter::AcceptExtension("stl").check(Path::new("trachea.stl")));
    /// assert_eq!(false, PathFilter::AcceptExtension("stl").check(Path::new("trachea.rs")));
    /// assert_eq!(false, PathFilter::DenyExtension("stl").check(Path::new("trachea.stl")));
    /// assert_eq!(true, PathFilter::DenyExtension("stl").check(Path::new("trachea.rs")));
    /// assert_eq!(true, PathFilter::AcceptStem("trachea").check(Path::new("trachea.rs")));
    /// assert_eq!(false, PathFilter::DenyStem("trachea").check(Path::new("trachea.rs")));
    /// assert_eq!(
    ///     true, 
    ///     PathFilter::AcceptAny(vec![
    ///         PathFilter::AcceptExtension("stl"),
    ///         PathFilter::AcceptExtension("rs"),
    ///     ]).check(Path::new("trachea.rs"))
    /// );
    /// assert_eq!(
    ///     false, 
    ///     PathFilter::AcceptAll(vec![
    ///         PathFilter::AcceptExtension("stl"),
    ///         PathFilter::AcceptStem("artery"),
    ///     ]).check(Path::new("artery.rs"))
    /// );
    pub fn check<P:AsRef<Path>>(&self, path: P) -> bool {
        let extension_matches = |criterion: &str| {
            path.as_ref()
                .extension()
                .map(|os_str| os_str.to_string_lossy().eq_ignore_ascii_case(criterion))
                .unwrap_or_default()
        };
        let stem_matches = |criterion: &str| {
            path.as_ref()
                .file_stem()
                .map(|os_str| os_str.to_string_lossy().eq_ignore_ascii_case(criterion))
                .unwrap_or_default()
        };
        match self {
            Self::All => true,
            Self::AcceptExtension(criterion) => extension_matches(criterion),
            Self::AcceptStem(criterion) => stem_matches(criterion),
            Self::AcceptMatching(criterion_fn) => criterion_fn(path.as_ref()),
            Self::AcceptAny(criteria) => criteria.iter()
                .any(|criterion| criterion.check(path.as_ref())),
            Self::AcceptAll(criteria) => criteria.iter()
                .all(|criterion| criterion.check(path.as_ref())),  
            Self::DenyExtension(criterion) => !extension_matches(criterion),
            Self::DenyStem(criterion) => !stem_matches(criterion),
            Self::DenyMatching(criterion_fn) => !criterion_fn(path.as_ref()),
        }
    }
}

/// Different path selection modes specify what the user can choose
/// 
/// 
#[derive(Clone, Eq, PartialEq)]
pub enum PathSelectionMode<'a> {
    /// The user may pick a directory path matching the filter.
    Directory(PathFilter<'a>),
    /// The user may pick a file path matching the filter.
    File(PathFilter<'a>),
    /// The user may pick a file or directory path
    FileOrDirectory(PathFilter<'a>),
}

impl<'a> Default for PathSelectionMode<'a> {
    fn default() -> Self {
        Self::File(PathFilter::default())
    }
}

macro_rules! define_path_sorting_modes {
    (
        $(#[$e_attr:meta])*
        pub enum $E:ident {
            $(
                $(#[$v_attr:meta])* 
                $V:ident,
            )+
        }
    ) => {
        $(#[$e_attr])*
        pub enum $E {
            $(
                $(#[$v_attr])* 
                $V, 
            )+
        }


        impl std::fmt::Display for $E {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$V => write!(f, stringify!($V)),
                    )+
                }
            }
        }

        impl $E {
            /// Get the next sorting mode
            pub (crate) fn next(self) -> Self {
                let variants = [ $(Self::$V),+];
                let index = variants.iter().position(|v| v == &self).expect("must find own position");
                let index = (index + 1) % variants.len();
                variants[index]
            }        
        } 


    };
}


define_path_sorting_modes!{
    #[doc = "Item sort options when displaying the list of files and directories."]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub enum PathSortingMode {
        #[doc = "Sort by path according to the standard library implementation"]
        Path,
        #[doc = "Sort by file size (directories listed first)"]
        Size,
        #[doc = "Sort by extension"]
        Extension,
    }     
}

impl Default for PathSortingMode {
    fn default() -> Self {
        Self::Path
    }
}
