#[cfg(test)]
#[cfg(not(any(target_os = "windows")))]
mod tests {
    use std::process::Command;

    use assert_cmd::assert::OutputAssertExt;
    use rexpect::{session::PtySession, spawn};

    #[test]
    fn test() -> anyhow::Result<()> {
        Command::new("cargo")
            .args(["build"])
            .assert()
            .success()
            .code(0);

        run_example("password_simple", |mut process| {
            process.exp_string("RSA Encryption Key")?;
            process.send_line("secret")?;
            process.exp_string("Confirmation:")?;
            process.send_line("secret")?;
            process.exp_string("This doesn't look like a key.")?;
            process.exp_eof()?;
            Ok(())
        })?;

        Ok(())
    }

    fn run_example<T>(example_name: &str, test_case: T) -> anyhow::Result<()>
    where
        T: FnOnce(PtySession) -> anyhow::Result<()>,
    {
        let process = spawn(
            format!("../target/debug/examples/{example_name}").as_str(),
            Some(30_000),
        )?;
        test_case(process)?;

        Ok(())
    }
}
