use std::{env, ffi::OsString, fs, io::Write, path::Path, process};

use tempfile::NamedTempFile;

use crate::{
    api::EditorPromptAction,
    error::InquireResult,
    new_prompts::{action::ParseKey, action_result::ActionResult, base::PromptImpl},
    ui::{EditorBackend, Key},
    EditorConfig,
};

impl ParseKey for EditorPromptAction {
    fn from_key(key: Key) -> Option<Self> {
        match key {
            Key::Char('e', _) => Some(Self::OpenEditor),
            _ => None,
        }
    }
}

pub struct EditorPrompt {
    config: EditorConfig,
    tmp_file: NamedTempFile,
    file_content_cache: String,
}

impl EditorPrompt {
    pub fn new(config: EditorConfig, default: Option<String>) -> InquireResult<Self> {
        let tmp_file =
            Self::create_file(&config.file_extension.to_string_lossy(), default.as_ref())?;

        Ok(Self {
            config,
            tmp_file,
            file_content_cache: default.unwrap_or_default(),
        })
    }

    fn create_file(
        file_extension: &str,
        predefined_text: Option<&String>,
    ) -> std::io::Result<NamedTempFile> {
        let mut tmp_file = tempfile::Builder::new()
            .prefix("tmp-")
            .suffix(file_extension)
            .rand_bytes(10)
            .tempfile()?;

        if let Some(predefined_text) = predefined_text {
            tmp_file.write_all(predefined_text.as_bytes())?;
            tmp_file.flush()?;
        }

        Ok(tmp_file)
    }

    fn run_editor(&mut self) -> InquireResult<()> {
        process::Command::new(&self.config.editor_command)
            .args(&self.config.editor_args)
            .arg(self.tmp_file.path())
            .spawn()?
            .wait()?;

        let mut submission = fs::read_to_string(self.tmp_file.path())?;
        let len = submission.trim_end_matches(&['\n', '\r'][..]).len();
        submission.truncate(len);
        self.file_content_cache = submission;

        Ok(())
    }
}

impl<'a, B> PromptImpl<'a, B> for EditorPrompt
where
    B: EditorBackend,
{
    type Action = EditorPromptAction;
    type Output = String;
    type OutputAsArgument = &'a str;

    fn handle(&mut self, action: EditorPromptAction) -> InquireResult<ActionResult> {
        let result = match action {
            EditorPromptAction::OpenEditor => {
                self.run_editor()?;

                ActionResult::NeedsRedraw
            }
        };

        Ok(result)
    }

    fn render(&self, message: &str, backend: &mut B) -> InquireResult<()> {
        let path = Path::new(&self.config.editor_command);
        let editor_name = path
            .file_stem()
            .and_then(|f| f.to_str())
            .unwrap_or("editor");

        backend.render_prompt(message, editor_name)?;

        Ok(())
    }

    fn current_submission(&self) -> Self::OutputAsArgument {
        &self.file_content_cache
    }

    fn into_output(self) -> Self::Output {
        self.file_content_cache
    }
}

pub fn get_default_editor_command() -> OsString {
    let mut default_editor = if cfg!(windows) {
        String::from("notepad")
    } else {
        String::from("nano")
    };

    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            default_editor = editor;
        }
    }

    if let Ok(editor) = env::var("VISUAL") {
        if !editor.is_empty() {
            default_editor = editor;
        }
    }

    default_editor.into()
}
