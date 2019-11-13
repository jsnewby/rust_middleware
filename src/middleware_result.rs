use serde_json;
use std::error::Error;
use std::fmt;
use std::option::NoneError;
use ws::Error as WsError;

#[derive(Debug)]
pub struct MiddlewareError {
    details: String,
}

impl MiddlewareError {
    pub fn new(msg: &str) -> MiddlewareError {
        let bt = backtrace::Backtrace::new();

        MiddlewareError {
            details: format!("{}\n{:?}", msg.to_string(), bt),
        }
    }
}

impl PartialEq for MiddlewareError {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for MiddlewareError {
    fn description(&self) -> &str {
        &self.details
    }
    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl From<NoneError> for MiddlewareError {
    fn from(_none: std::option::NoneError) -> Self {
        MiddlewareError::new("None")
    }
}

#[macro_export]
macro_rules! middleware_error_from {
    {$fromtype:ty} => {
        impl From<$fromtype> for MiddlewareError {
            fn from(err: $fromtype) -> Self {
                MiddlewareError::new(&err.to_string())
            }
        }
    }
}

middleware_error_from!(r2d2::Error);
middleware_error_from!(Box<dyn std::error::Error>);
middleware_error_from!(serde_json::Error);
middleware_error_from!(diesel::result::Error);
middleware_error_from!(postgres::Error);
middleware_error_from!(curl::Error);
middleware_error_from!(std::str::Utf8Error);
middleware_error_from!(std::sync::mpsc::TryRecvError);
middleware_error_from!(std::sync::mpsc::SendError<i64>);
middleware_error_from!(WsError);
middleware_error_from!(std::string::FromUtf8Error);
middleware_error_from!(curl::FormError);
middleware_error_from!(reqwest::Error);
middleware_error_from!(
    std::sync::PoisonError<
        std::sync::MutexGuard<'_, std::cell::RefCell<std::vec::Vec<crate::websocket::Client>>>,
    >
);

middleware_error_from!(bigdecimal::ParseBigDecimalError);
middleware_error_from!(std::env::VarError);
middleware_error_from!(reqwest::header::InvalidHeaderValue);
middleware_error_from!(base64::DecodeError);
middleware_error_from!(
    std::sync::PoisonError<std::sync::MutexGuard<'_, std::collections::HashMap<i64, bool>>>
);

pub type MiddlewareResult<T> = Result<T, MiddlewareError>;
