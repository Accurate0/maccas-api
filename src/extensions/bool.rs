pub trait BoolExtensions {
    fn unwrap_or_false(self) -> bool;
}

impl BoolExtensions for Option<bool> {
    fn unwrap_or_false(self) -> bool {
        self.unwrap_or(false)
    }
}
