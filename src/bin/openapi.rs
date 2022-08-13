use libapi::routes::docs::openapi::get_openapi;
use std::env;
use std::{fs::File, io::Write};

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().skip(1).collect();
    if !args.is_empty() {
        let output = &args[0];
        let openapi = get_openapi();
        let mut file = File::create(output)?;
        file.write_all(openapi.unwrap().to_pretty_json().unwrap().as_bytes())?;
    } else {
        println!("help: openapi [OUTPUT]")
    }
    Ok(())
}
