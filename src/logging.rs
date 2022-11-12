use anyhow::Context;

pub fn setup_logging() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for(
            "aws_smithy_http_tower::parse_response",
            log::LevelFilter::Warn,
        )
        .level_for(
            "aws_config::default_provider::credentials",
            log::LevelFilter::Warn,
        )
        .chain(std::io::stdout())
        .apply()
        .context("failed to set up logger")
        .unwrap();
}

pub fn dump_build_details() {
    log::info!("maccas-api v{}", env!("VERGEN_BUILD_SEMVER"));
    log::info!("build: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    log::info!("hash: {}", env!("VERGEN_GIT_SHA"));
    log::info!("rustc: {}", env!("VERGEN_RUSTC_SEMVER"));
}
