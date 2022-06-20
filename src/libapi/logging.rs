use simplelog::*;

pub fn setup_logging() {
    let term_config = ConfigBuilder::new().set_level_padding(LevelPadding::Right).build();
    TermLogger::init(LevelFilter::Info, term_config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();
}
