pub mod invidious;

#[macro_export]
macro_rules! ensure {
    ($statement: expr, $error: expr) => {
        if !$statement {
            return ::std::result::Result::Err($error.into());
        }
    };
}
