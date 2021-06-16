pub struct PromptConfig {
    pub page_size: usize,
    pub help_input: String,
    pub filter: fn(filter: &str, value: &str, index: usize) -> bool,
    pub keep_filter: bool,
    pub show_cursor: bool,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            page_size: 7,
            help_input: String::from("?"),
            filter: |filter: &str, value: &str, _| -> bool {
                let filter = filter.to_lowercase();

                value.to_lowercase().contains(&filter)
            },
            keep_filter: true,
            show_cursor: false,
        }
    }
}
