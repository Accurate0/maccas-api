#[macro_export]
macro_rules! return_jwt_unauthorized {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                log::warn!("return unauthorized: {}", e);
                return Ok(LambdaAuthorizerResponse {
                    is_authorized: false,
                });
            }
        }
    };
}
