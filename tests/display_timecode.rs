use std::process::Command;

#[test]
#[ignore = "runs example 13, which uses the camera and creates an external display configurator that can reconfigure an attached external display"]
fn display_and_timecode_example_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(["run", "--example", "13_display_timecode", "--quiet"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "example failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("desk view application info:")
            || stdout.contains("skipping desk view application:"),
        "unexpected example output:\n{stdout}"
    );
    assert!(
        stdout.contains("external display") || stdout.contains("skipping external display"),
        "unexpected example output:\n{stdout}"
    );
    assert!(
        stdout.contains("timecode") || stdout.contains("skipping timecode generator:"),
        "unexpected example output:\n{stdout}"
    );
    Ok(())
}
