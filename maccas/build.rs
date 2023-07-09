use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    EmitBuilder::builder()
        .git_sha(false)
        .build_timestamp()
        .rustc_semver()
        .emit()?;
    Ok(())
}
