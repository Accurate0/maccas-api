use crate::constants::AWS_LAMBDA_FUNCTION_NAME;
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_logger() {
    let is_aws = std::env::var(AWS_LAMBDA_FUNCTION_NAME).is_ok();

    let subscriber = tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(!is_aws);

    if is_aws {
        subscriber.json().init()
    } else {
        subscriber.init()
    };
}
