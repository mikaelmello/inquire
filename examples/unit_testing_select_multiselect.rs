//! Demonstrates how to unit test `Select` and `MultiSelect` while keeping production
//! code **unchanged**.
//!
//! **Key idea**
//! - In tests, enable the `inquire` `testing` feature and wrap that code in
//!   [`inquire::testing::with_input`].
//! - While that scope is active, `.prompt()` reads scripted keys and captures a
//!   frame-by-frame trace.
//!
//! This example asserts the returned values and (optionally) prints the captured
//! trace as a token dump and ANSI replay for debugging.

use inquire::{
    testing::{with_input, Key, KeyModifiers},
    MultiSelect, Select,
};

fn select_flow() -> Result<&'static str, inquire::InquireError> {
    Select::new("Pick one", vec!["A", "B", "C"]).prompt()
}

fn multiselect_flow() -> Result<Vec<&'static str>, inquire::InquireError> {
    MultiSelect::new("Pick many", vec!["A", "B", "C"]).prompt()
}

fn main() -> Result<(), inquire::InquireError> {
    // --- Unit-testing a Select-driven flow ---
    // Script the user's interaction:
    // - press Down once (moves cursor from option 0 -> option 1)
    // - press Enter (submits current option)
    let (select_result, select_report) =
        with_input(vec![Key::Down(KeyModifiers::NONE), Key::Enter], || {
            select_flow()
        });

    let selected = select_result?;
    assert_eq!("B", selected);
    select_report.assert_all_input_consumed();

    // Assert that something meaningful was rendered.
    assert!(
        select_report.trace().contains_text("Pick one"),
        "Expected prompt text to appear in trace. Trace:\n{}",
        select_report.trace().to_token_dump_string()
    );

    let select_ansi = select_report.trace().to_ansi_string();
    assert!(
        select_ansi.contains("Pick one"),
        "Expected prompt text to appear in ANSI replay. ANSI:\n{select_ansi}"
    );

    // If you want to debug what was rendered at each step, print the trace.
    // In real unit tests you'd typically only print this on failure.
    println!(
        "Select token dump (debug view):\n{}",
        select_report.trace().to_token_dump_string()
    );
    println!("Select ANSI replay (visual view):\n{select_ansi}");

    // --- Unit-testing a MultiSelect-driven flow ---
    // Scripted interaction:
    // - Down (cursor 0 -> 1)
    // - Space (toggle option 1)
    // - Down (cursor 1 -> 2)
    // - Space (toggle option 2)
    // - Enter (submit)
    let (multi_result, multi_report) = with_input(
        vec![
            Key::Down(KeyModifiers::NONE),
            Key::Char(' ', KeyModifiers::NONE),
            Key::Down(KeyModifiers::NONE),
            Key::Char(' ', KeyModifiers::NONE),
            Key::Enter,
        ],
        || multiselect_flow(),
    );

    let selected = multi_result?;
    assert_eq!(vec!["B", "C"], selected);
    multi_report.assert_all_input_consumed();

    assert!(
        multi_report.trace().contains_text("Pick many"),
        "Expected prompt text to appear in trace. Trace:\n{}",
        multi_report.trace().to_token_dump_string()
    );

    let multi_ansi = multi_report.trace().to_ansi_string();
    assert!(
        multi_ansi.contains("Pick many"),
        "Expected prompt text to appear in ANSI replay. ANSI:\n{multi_ansi}"
    );

    println!(
        "MultiSelect token dump (debug view):\n{}",
        multi_report.trace().to_token_dump_string()
    );
    println!("MultiSelect ANSI replay (visual view):\n{multi_ansi}");

    Ok(())
}
