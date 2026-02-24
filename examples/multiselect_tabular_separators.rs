use inquire::{
    tabular::{ColumnAlignment, ColumnConfig},
    MultiSelect,
};

fn main() {
    println!("Example: Understanding Column Separators in Tabular Formatting\n");

    example_default_separator();
    println!("\n{}\n", "=".repeat(70));

    example_custom_separators();
    println!("\n{}\n", "=".repeat(70));

    example_replacing_separator();
    println!("\n{}\n", "=".repeat(70));

    example_raw_vs_formatted();
}

/// Example showing the default separator (single space " ")
fn example_default_separator() {
    println!("1. Using Default Separators");
    println!("   ColumnConfig::new(alignment) uses a single space ' ' as separator\n");

    let data = vec![
        "Project1 100MB 2025-10-15 Active",
        "LongerProjectName 2GB 2025-10-14 Inactive",
        "Proj 50KB 2025-10-16 Active",
    ];

    // All columns use default separator (single space)
    let columns = vec![
        ColumnConfig::new(ColumnAlignment::Left), // Project name + " "
        ColumnConfig::new(ColumnAlignment::Right), // Size + " "
        ColumnConfig::new(ColumnAlignment::Left), // Date + " "
        ColumnConfig::new(ColumnAlignment::Left), // Status (last column)
    ];

    println!("Column configuration:");
    println!("  - Column 1: Left-aligned, separator: ' ' (default)");
    println!("  - Column 2: Right-aligned, separator: ' ' (default)");
    println!("  - Column 3: Left-aligned, separator: ' ' (default)");
    println!("  - Column 4: Left-aligned (last column)\n");

    let ans = MultiSelect::new("Select items (default separators):", data)
        .with_tabular_columns(columns)
        .with_page_size(5)
        .prompt();

    match ans {
        Ok(selected) => println!("\nYou selected {} item(s)", selected.len()),
        Err(_) => println!("\nSelection cancelled"),
    }
}

/// Example showing custom separators
fn example_custom_separators() {
    println!("2. Using Custom Separators");
    println!("   ColumnConfig::new_with_separator(sep, alignment) for custom separators\n");

    let servers = vec![
        "web-server-01: 192.168.1.10, 8080, Running",
        "api-gateway: 192.168.1.20, 3000, Running",
        "db-primary: 192.168.1.30, 5432, Stopped",
    ];

    // Mix of custom and default separators
    let columns = vec![
        ColumnConfig::new_with_separator(": ", ColumnAlignment::Left), // Server name + ": "
        ColumnConfig::new_with_separator(", ", ColumnAlignment::Right), // IP + ", "
        ColumnConfig::new_with_separator(", ", ColumnAlignment::Right), // Port + ", "
        ColumnConfig::new(ColumnAlignment::Left),                      // Status (last column)
    ];

    println!("Column configuration:");
    println!("  - Column 1: Left-aligned, separator: ': '");
    println!("  - Column 2: Right-aligned, separator: ', '");
    println!("  - Column 3: Right-aligned, separator: ', '");
    println!("  - Column 4: Left-aligned (last column)\n");

    let ans = MultiSelect::new("Select servers (custom separators):", servers)
        .with_tabular_columns(columns)
        .with_page_size(5)
        .prompt();

    match ans {
        Ok(selected) => println!("\nYou selected {} server(s)", selected.len()),
        Err(_) => println!("\nSelection cancelled"),
    }
}

/// Example showing how to set/replace separators on existing configurations
fn example_replacing_separator() {
    println!("3. Setting Separators with .separator()");
    println!("   Set or change the separator on a ColumnConfig\n");

    let tasks = vec![
        "task-1: Pending, High, 2025-10-15",
        "task-2: Complete, Low, 2025-10-14",
        "task-3: In Progress, Medium, 2025-10-16",
    ];

    // Start with default separator, then set it to something else
    let columns = vec![
        ColumnConfig::new(ColumnAlignment::Left).separator(": "), // Set separator to ": "
        ColumnConfig::new(ColumnAlignment::Left).separator(", "), // Set separator to ", "
        ColumnConfig::new_with_separator(" | ", ColumnAlignment::Left).separator(", "), // Replace " | " with ", "
        ColumnConfig::new(ColumnAlignment::Left), // Last column
    ];

    println!("Column configuration:");
    println!("  - Column 1: new() then .separator(': ')");
    println!("  - Column 2: new() then .separator(', ')");
    println!("  - Column 3: new_with_separator(' | ') then .separator(', ')");
    println!("  - Column 4: Left-aligned (last column)\n");

    println!("This is useful when you want to:");
    println!("  • Build column configs with a fluent API");
    println!("  • Modify configurations programmatically");
    println!("  • Chain configuration methods\n");

    let ans = MultiSelect::new("Select tasks (replaced separators):", tasks)
        .with_tabular_columns(columns)
        .with_page_size(5)
        .prompt();

    match ans {
        Ok(selected) => println!("\nYou selected {} task(s)", selected.len()),
        Err(_) => println!("\nSelection cancelled"),
    }
}

/// Example showing raw rows vs formatted output
fn example_raw_vs_formatted() {
    use inquire::tabular::format_as_table;

    println!("4. Raw Rows vs Formatted Output");
    println!("   See the difference tabular formatting makes\n");

    // Raw data - this is what you'd typically have
    let raw_rows = vec![
        "file.txt: 1.2 KB, 2025-10-15, /home/user/docs".to_string(),
        "document.pdf: 3.4 MB, 2025-10-14, /home/user/downloads".to_string(),
        "image.png: 856 KB, 2025-10-16, /home/user/pictures".to_string(),
    ];

    println!("RAW ROWS (unformatted):");
    println!("─────────────────────────────────────────────────────────────");
    for row in &raw_rows {
        println!("{}", row);
    }
    println!();

    // Configure columns for formatting
    let columns = vec![
        ColumnConfig::new_with_separator(": ", ColumnAlignment::Left), // Filename
        ColumnConfig::new_with_separator(", ", ColumnAlignment::Right), // Size
        ColumnConfig::new_with_separator(", ", ColumnAlignment::Left), // Date
        ColumnConfig::new(ColumnAlignment::Left),                      // Path
    ];

    // Format the rows
    let formatted_rows = format_as_table(&raw_rows, &columns);

    println!("FORMATTED ROWS (with tabular alignment):");
    println!("─────────────────────────────────────────────────────────────");
    for row in &formatted_rows {
        println!("{}", row);
    }
    println!();

    println!("Notice how:");
    println!("  ✓ Filenames align on the left");
    println!("  ✓ File sizes align on the right (easier to compare)");
    println!("  ✓ Dates start at the same column");
    println!("  ✓ Paths start at the same column");
    println!("  ✓ Much easier to scan and read!\n");

    // Now use it in a MultiSelect
    let ans = MultiSelect::new("Select files:", formatted_rows)
        .with_page_size(5)
        .prompt();

    match ans {
        Ok(selected) => println!("\nYou selected {} file(s)", selected.len()),
        Err(_) => println!("\nSelection cancelled"),
    }
}
