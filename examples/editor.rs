use inquire::{
    error::InquireResult,
    ui::{Color, RenderConfig, Styled},
    Editor, Text,
};

fn main() -> InquireResult<()> {
    let _title = Text::new("Title:")
        .with_render_config(default_render_config())
        .prompt()?;

    let _description = Editor::new("Description:")
        .with_formatter(&|submission| {
            let char_count = submission.chars().count();
            if char_count == 0 {
                String::from("<skipped>")
            } else if char_count <= 20 {
                submission.into()
            } else {
                let mut substr: String = submission.chars().take(17).collect();
                substr.push_str("...");
                substr
            }
        })
        .with_render_config(description_render_config())
        .prompt()?;

    Ok(())
}

fn default_render_config() -> RenderConfig<'static> {
    RenderConfig::default().with_global_prefix(Styled::new("║ "))
}

fn description_render_config() -> RenderConfig<'static> {
    default_render_config()
        .with_canceled_prompt_indicator(Styled::new("<skipped>").with_fg(Color::DarkYellow))
}
