pub enum SupportedFileTypes {
    PNG,
}

impl SupportedFileTypes {
    pub fn _get_file_extension(&self) -> &'static str {
        match self {
            Self::PNG => "png",
        }
    }

    // get all enums
    pub fn _all() -> &'static [Self] {
        &[Self::PNG]
    }

    // Checks if the input matches this type (ignoring leading dot)
    pub fn _matches(&self, input: &str) -> bool {
        let input = input.strip_prefix('.').unwrap_or(input);
        input.eq_ignore_ascii_case(self._get_file_extension())
            || input.ends_with(&format!(".{}", self._get_file_extension()))
    }

    // Checks if input is supported by any enum variant
    pub fn is_supported(input: &str) -> bool {
        Self::_all().iter().any(|v| v._matches(input))
    }
}
