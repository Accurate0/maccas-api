use anyhow::Context;
use log::LevelFilter;

pub fn init_logger(level: LevelFilter, warn_modules: &[&'static str]) {
    let cfg = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level);

    let cfg = {
        let cfg = cfg
            .level_for(
                "aws_smithy_http_tower::parse_response",
                log::LevelFilter::Warn,
            )
            .level_for(
                "aws_config::default_provider::credentials",
                log::LevelFilter::Warn,
            );

        let mut cfg = cfg;
        for module in warn_modules {
            cfg = cfg.level_for(*module, log::LevelFilter::Warn);
        }

        cfg
    };

    cfg.chain(std::io::stdout())
        .apply()
        .context("failed to set up logger")
        .unwrap();
}
