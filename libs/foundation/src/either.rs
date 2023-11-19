use rocket::request::FromParam;
use std::{fmt::Debug, str::FromStr};

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: FromStr, R: FromStr> FromParam<'_> for Either<L, R>
where
    L::Err: Sync + Send + Debug,
    R::Err: Sync + Send + Debug,
{
    type Error = R::Err;

    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        Either::from_str(param)
    }
}

impl<L, R> FromStr for Either<L, R>
where
    L: FromStr,
    R: FromStr,
    L::Err: Sync + Send + Debug,
    R::Err: Sync + Send + Debug,
{
    type Err = R::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.parse::<L>() {
            Ok(p) => Either::Left(p),
            Err(_) => Either::Right(s.parse::<R>()?),
        })
    }
}
