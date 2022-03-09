#[macro_export]
macro_rules! status {
    ($name:ident,$($arg:tt)*) => ({
        $crate::Status::$name(format!($($arg)*))
    })
}

#[macro_export]
macro_rules! invalid_argument {
    ($($arg:tt)*) => ($crate::status!(invalid_argument, $($arg)*))
}

#[macro_export]
macro_rules! internal {
    ($($arg:tt)*) => ($crate::status!(internal, $($arg)*))
}

#[cfg(test)]
mod tests {
    use crate::Code;

    #[test]
    pub fn macros_use_correct_code() {
        assert_eq!(invalid_argument!("message").code(), Code::InvalidArgument);
        assert_eq!(
            invalid_argument!("bad input: {0}", "some issue").message(),
            "bad input: some issue"
        );
    }
}
