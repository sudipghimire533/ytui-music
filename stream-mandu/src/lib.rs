#[cfg(feature = "invidious")]
pub mod invidious;
pub mod region;
pub mod web_client;

#[cfg(feature = "common-interface")]
pub mod common_interface;
#[cfg(feature = "common-interface")]
pub use common_interface::StreamMandu;

#[macro_export]
macro_rules! ensure {
    ($statement: expr, $error: expr) => {
        if !$statement {
            return ::std::result::Result::Err($error.into());
        }
    };
}
