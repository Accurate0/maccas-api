pub trait ResultExtension<T, E> {
    fn flatten_std(self) -> Result<T, E>;
}

impl<T, E> ResultExtension<T, E> for Result<Result<T, E>, E> {
    #[inline]
    fn flatten_std(self) -> Result<T, E> {
        self.and_then(std::convert::identity)
    }
}
