use rocket::form::{self, Error};

pub fn validate_username<'v>(s: &str) -> form::Result<'v, ()> {
    if !s.is_ascii() {
        Err(Error::validation("must be ascii only"))?;
    }

    Ok(())
}

pub fn validate_password<'v>(s: &str) -> form::Result<'v, ()> {
    if !s.is_ascii() {
        Err(Error::validation("must be ascii only"))?;
    }

    if s.len() <= 5 {
        Err(Error::validation("must be 6 or more characters"))?;
    }

    Ok(())
}
