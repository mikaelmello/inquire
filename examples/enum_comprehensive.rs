use std::fmt::{Display, Formatter};

use inquire::error::InquireResult;
use inquire_derive::Selectable;

fn main() -> InquireResult<()> {
    println!("=== Programming Language Preferences ===\n");

    // Single selection example
    println!("1. Select your primary programming language:");
    let primary_language = ProgrammingLanguage::select("Primary language:")
        .with_help_message("Use ↑↓ to navigate, Enter to select")
        .prompt()?;
    println!("You chose: {primary_language}\n");

    // Multi-selection example with customization
    println!("2. Select all languages you're familiar with:");
    let familiar_languages = ProgrammingLanguage::multi_select("Familiar languages:")
        .with_help_message("Use ↑↓ to navigate, Space to select, Enter to confirm")
        .prompt()?;

    if familiar_languages.is_empty() {
        println!("No languages selected.\n");
    } else {
        println!(
            "You're familiar with {} language(s):",
            familiar_languages.len()
        );
        for lang in &familiar_languages {
            println!("  - {lang}");
        }
        println!();
    }

    // Demonstration with Task Priority and custom page size
    println!("3. Select a task priority level:");
    let priority = TaskPriority::select("Priority level:")
        .with_page_size(3)
        .prompt()?;
    println!("Task priority: {} ({})\n", priority, priority.description());

    // Multi-select with task priorities and default selection
    println!("4. Select multiple priority levels to filter tasks:");
    let filter_priorities = TaskPriority::multi_select("Filter by priorities:")
        .with_default(&[0, 1]) // Pre-select Low and Medium
        .prompt()?;
    if !filter_priorities.is_empty() {
        println!("Filtering tasks with priorities: {filter_priorities:?}");
    }

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Selectable)]
enum ProgrammingLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    CPlusPlus,
    C,
    Swift,
    Kotlin,
    Other,
}

impl Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgrammingLanguage::Rust => write!(f, "Rust 🦀"),
            ProgrammingLanguage::Python => write!(f, "Python 🐍"),
            ProgrammingLanguage::JavaScript => write!(f, "JavaScript 🟨"),
            ProgrammingLanguage::TypeScript => write!(f, "TypeScript 🔷"),
            ProgrammingLanguage::Go => write!(f, "Go 🐹"),
            ProgrammingLanguage::Java => write!(f, "Java ☕"),
            ProgrammingLanguage::CSharp => write!(f, "C# 💜"),
            ProgrammingLanguage::CPlusPlus => write!(f, "C++ ⚡"),
            ProgrammingLanguage::C => write!(f, "C 🔧"),
            ProgrammingLanguage::Swift => write!(f, "Swift 🦉"),
            ProgrammingLanguage::Kotlin => write!(f, "Kotlin 🎯"),
            ProgrammingLanguage::Other => write!(f, "Other 🤷"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Selectable)]
enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    fn description(&self) -> &'static str {
        match self {
            TaskPriority::Low => "Can be done later",
            TaskPriority::Medium => "Should be done soon",
            TaskPriority::High => "Important, do this week",
            TaskPriority::Critical => "Urgent, drop everything!",
        }
    }
}

impl Display for TaskPriority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "🟢 Low"),
            TaskPriority::Medium => write!(f, "🟡 Medium"),
            TaskPriority::High => write!(f, "🟠 High"),
            TaskPriority::Critical => write!(f, "🔴 Critical"),
        }
    }
}
