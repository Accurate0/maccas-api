pub fn dump_build_details() {
    log::info!("build: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    log::info!("hash: {}", env!("VERGEN_GIT_SHA"));
    log::info!("rustc: {}", env!("VERGEN_RUSTC_SEMVER"));
}
