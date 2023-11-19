use rocket::request::FromParam;
use std::{fmt::Debug, str::FromStr};
use uuid::Uuid;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: FromStr> FromParam<'_> for Either<L, Uuid>
where
    L::Err: Sync + Send + Debug,
{
    type Error = L::Err;

    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        match uuid::Uuid::parse_str(param) {
            Ok(uuid) => Ok(Either::Right(uuid)),
            Err(_) => Ok(Either::Left(param.parse()?)),
        }
    }
}
