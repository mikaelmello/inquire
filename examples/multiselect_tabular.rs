use inquire::{
    tabular::{ColumnAlignment, ColumnConfig},
    MultiSelect,
};

fn main() {
    println!("Example 1: Project Selection with Tabular Formatting\n");
    example_project_selection();

    println!("\n\nExample 2: Package Manager Style Display\n");
    example_package_manager();

    println!("\n\nExample 3: File Browser with Metadata\n");
    example_file_browser();
}

/// Example demonstrating project selection with aligned columns
/// Shows project name, size, date, and path in a readable tabular format
fn example_project_selection() {
    let projects = vec![
        "copy_current_location: 898.95 KB (2025-10-12 15:41), /Users/tom/Prog/Rust/copy_current_location",
        "summary-gen: 211.29 MB (2025-10-13 20:04), /Users/tom/Prog/Rust/summary-gen",
        "rona: 1.26 GB (2025-10-14 18:29), /Users/tom/Prog/Rust/rona/rona",
        "src-tauri: 2.59 GB (2025-10-13 10:24), /Users/tom/Prog/Rust/wordle-helper/src-tauri",
        "clean-dev-dirs: 2.75 GB (2025-10-15 11:50), /Users/tom/Prog/Rust/clean-dev-dirs",
        "qobuz-client: 3.21 GB (2025-10-12 16:52), /Users/tom/Prog/Rust/qobuz-client",
    ];

    // Define columns: name (left-aligned), size (right-aligned), date (left), path (left)
    let columns = vec![
        ColumnConfig::new_with_separator(": ", ColumnAlignment::Left), // Project name
        ColumnConfig::new(ColumnAlignment::Right), // Size (right-aligned for comparison)
        ColumnConfig::new_with_separator(", ", ColumnAlignment::Left), // Date
    ];

    let ans = MultiSelect::new("Select projects to clean:", projects)
        .with_tabular_columns(columns)
        .with_help_message("Use space to select, arrow keys to navigate, enter to confirm")
        .prompt();

    match ans {
        Ok(selected) => {
            println!("\nYou selected {} project(s) to clean:", selected.len());
            for project in selected {
                let name = project.split(':').next().unwrap_or("unknown");
                println!("  - {}", name);
            }
        }
        Err(_) => println!("Selection cancelled"),
    }
}

/// Example showing package manager style display with version, size, and description
fn example_package_manager() {
    let packages = vec![
        "serde: 1.0.197 (50.2 KB), Serialization framework",
        "tokio: 1.36.0 (634.8 KB), Async runtime",
        "clap: 4.5.1 (55.3 KB), Command line argument parser",
        "reqwest: 0.11.24 (186.4 KB), HTTP client",
        "sqlx: 0.7.3 (321.7 KB), SQL toolkit and ORM",
    ];

    let columns = vec![
        ColumnConfig::new_with_separator(": ", ColumnAlignment::Left), // Package name
        ColumnConfig::new(ColumnAlignment::Right), // Version (default separator)
        ColumnConfig::new_with_separator(", ", ColumnAlignment::Left), // Size
    ];

    let ans = MultiSelect::new("Select packages to install:", packages)
        .with_tabular_columns(columns)
        .prompt();

    match ans {
        Ok(selected) => {
            println!("\nInstalling {} package(s)...", selected.len());
            for package in selected {
                let name = package.split(':').next().unwrap_or("unknown");
                println!("  ✓ {}", name);
            }
        }
        Err(_) => println!("Installation cancelled"),
    }
}

/// Example showing file browser with permissions, size, and modification date
fn example_file_browser() {
    let files = vec![
        "config.toml: -rw-r--r-- (2.4 KB), 2025-10-15 10:23",
        "main.rs: -rw-r--r-- (15.7 KB), 2025-10-15 14:30",
        "lib.rs: -rw-r--r-- (8.1 KB), 2025-10-14 16:45",
        "tests.rs: -rw-r--r-- (22.3 KB), 2025-10-15 09:12",
        "README.md: -rw-r--r-- (4.8 KB), 2025-10-13 11:20",
    ];

    let columns = vec![
        ColumnConfig::new_with_separator(": ", ColumnAlignment::Left), // Filename
        ColumnConfig::new(ColumnAlignment::Left), // Permissions (default separator)
        ColumnConfig::new_with_separator(", ", ColumnAlignment::Right), // Size
    ];

    let ans = MultiSelect::new("Select files to include:", files)
        .with_tabular_columns(columns)
        .with_page_size(10)
        .prompt();

    match ans {
        Ok(selected) => {
            println!("\nSelected {} file(s):", selected.len());
            for file in selected {
                let name = file.split(':').next().unwrap_or("unknown");
                println!("  - {}", name);
            }
        }
        Err(_) => println!("Selection cancelled"),
    }
}
