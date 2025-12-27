use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &["src/caching/proto/offer_details.proto"],
        &["src/caching/proto"],
    )?;
    Ok(())
}
