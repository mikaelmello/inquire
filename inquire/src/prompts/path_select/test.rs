//! Tests for path selector
use std::{iter::FromIterator, path::{Path, PathBuf}};

use crate::{
  PathSelect, 
  prompts::path_select::{PathSelectionMode, PathSortingMode}, terminal::crossterm::CrosstermTerminal, ui::{Backend, RenderConfig}, list_option::ListOption, 
};
use lazy_static::lazy_static;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};

#[derive(Clone, Debug)]
enum Spec {
    Directory{ name: &'static str, tree: Vec<Spec> },
    File{ name: &'static str, size: u64 },
}

impl From<Spec> for PathBuf {
    fn from(value: Spec) -> Self {
        match value {
            Spec::Directory{ name, .. } => PathBuf::from(name),
            Spec::File{ name, .. } => PathBuf::from(name),
        }
    }
}

lazy_static!{
  static ref TREE: Vec<Spec> = vec![
      Spec::Directory{ 
          name:"dir_0", 
          tree: vec![
              Spec::File{ name: "xmlfile_0_0.xml", size: 66 },
              Spec::File{ name: "xmlfile_0_1.xml", size: 14 },
              Spec::File{ name: "rustfile_0_0.rs", size: 336 },
              Spec::File{ name: "rustfile_0_1.rs", size: 3336 },
        ] 
      },
      Spec::Directory{ 
          name:"dir_1", 
          tree: vec![] 
      },
      Spec::Directory{ 
          name:"dir_2", 
          tree: vec![
              Spec::File{ name: "htmlfile_2_0.html", size: 66 },
              Spec::File{ name: "htmlfile_2_1.html", size: 14 },
          Spec::Directory{ 
              name:"dir_2_0", 
              tree: vec![
                  Spec::File{ name: "a.html", size: 66 },
                  Spec::File{ name: "b.html", size: 14 },
                  Spec::File{ name: "c.rs", size: 4181 },
                  Spec::File{ name: "d.rs", size: 987 },
                  Spec::File{ name: "e.rs", size: 2584 },
              ] 
          },
        ] 
      },
      Spec::Directory{ 
          name:"dir_3", 
          tree: vec![
              Spec::File{ name: "jpegfile_3_0.jpeg", size: 332}
          ] 
      },
      Spec::File{ 
          name: "tomlfile_0.toml", size: 10 
      },
      Spec::File{ 
          name: "tomlfile_1.toml", size: 300 
      },
      Spec::File{ 
          name: "rustfile_0.rs", size: 300 
      },
      Spec::File{ 
          name: "rustfile_1.rs", size: 4000 
      },
      Spec::File{ 
          name: "mp3file_0.mp3", size: 300 
      },
      Spec::File{ 
          name: "mp3file_1.mp3", size: 5000 
      },
  ];
  static ref TEMPDIR: PathBuf = std::env::temp_dir(); 
  static ref SUBDIR_NAME: PathBuf = PathBuf::from("tests");
  static ref SUBDIR: PathBuf = TEMPDIR.join(&*SUBDIR_NAME);
}

fn tk<P: AsRef<Path>>(spec: &Spec, base: P) -> Result<(), std::io::Error> {
  let base = base.as_ref();
  match spec {
      Spec::File{ name, size, .. } => {
          let contents = str::repeat("x", *size as usize);
          fs_err::write(base.join(name), contents)
      },
      Spec::Directory { name, tree, ..} => {
          let subpath = &base.join(name);
          fs_err::create_dir_all(subpath)
              .and_then(|_| tree.iter().try_for_each(|subspec| {
                  tk(subspec, subpath)
              }))
      }
  }
}

/// Set up some dummy file system
fn setup_spec() -> Result <(), std::io::Error> {
  if SUBDIR.exists() {
      fs_err::remove_dir_all(&*SUBDIR)?;
  }
  fs_err::create_dir_all(&*SUBDIR)?;
  TREE.iter()
      .try_for_each(|spec| tk(spec, &*SUBDIR))
}

/// Create a check path string from file components 
fn make_check_path(components: &[&'static str]) -> String {
  let mut base_path = SUBDIR.to_path_buf();
  components.iter().for_each(|c| {
      base_path = base_path.join(c);
  });
  base_path.to_string_lossy().to_string()
}

#[test]
/// Tests that the only entries shown/selectable are those allowed in configuration
fn filtering_entries() -> Result <(), std::io::Error> {
  setup_spec()?;

  let key_events = Vec::<KeyEvent>::from_iter([
      KeyCode::Down.into(),
      KeyCode::Down.into(),
      KeyCode::Right.into(),
      KeyCode::Right.into(), 
      KeyEvent::new_with_kind(
          KeyCode::Right, 
          KeyModifiers::SHIFT, 
          KeyEventKind::Press
      ),
      KeyCode::Enter.into(),
  ].iter().copied());
  let read = &mut key_events.iter(); 

  // let write = &mut std::io::stdout();
  let write = &mut Vec::<u8>::new();
  let terminal = CrosstermTerminal::new_with_io(write, read);
  let backend = &mut Backend::new(terminal, RenderConfig::default())?;

  let answer = PathSelect::new(
    "select path", 
    Some(SUBDIR.to_path_buf())
  )
      .with_select_multiple(true)
      .with_selection_mode(PathSelectionMode::File(Some("rs")))
      .with_sorting_mode(PathSortingMode::Size)
      // .raw_prompt()
      .prompt_with_backend(backend)
      .map(|mut selected| {
          Vec::from_iter(selected.drain(0..).map(|ListOption { index, value }|{
              (
                  index, 
                  value.path.to_str().expect("must get listoption path str").to_string()
              )
          }))
      })
      .expect("must get answer");

  assert_eq!(
      vec![
          (0usize, make_check_path(&["dir_2", "dir_2_0", "d.rs"])),
          (1, make_check_path(&["dir_2", "dir_2_0", "e.rs"])),
          (2, make_check_path(&["dir_2", "dir_2_0", "c.rs"])),
      ],
      answer,
      "must select items based on extension filter",
  );  
  
  Ok(())
}