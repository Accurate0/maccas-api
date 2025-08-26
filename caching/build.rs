use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/offer_details.proto"], &["src/proto"])?;
    Ok(())
}
