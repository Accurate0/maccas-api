pub trait StringExtensions {
    fn lowercase_trim(&self) -> String;
}

impl StringExtensions for String {
    fn lowercase_trim(&self) -> String {
        self.to_ascii_uppercase().trim().to_owned()
    }
}

impl StringExtensions for &String {
    fn lowercase_trim(&self) -> String {
        self.to_ascii_uppercase().trim().to_owned()
    }
}

impl StringExtensions for &str {
    fn lowercase_trim(&self) -> String {
        self.to_ascii_uppercase().trim().to_owned()
    }
}
