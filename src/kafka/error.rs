use crate::kafka::{Response, Status};

pub mod error_codes {
    pub const INTERNAL_SERVER_ERROR: &str = "INTERNAL_SERVER_ERROR";
    pub const URI_NOT_FOUND: &str = "URI_NOT_FOUND";
    pub const INVALID_PARAMETER: &str = "INVALID_PARAMETER";
    pub const FIELD_REQUIRED: &str = "FIELD_REQUIRED";
    pub const VALUE_INVALID: &str = "VALUE_INVALID";
    pub const TIMEOUT_ERROR: &str = "TIMEOUT_ERROR";
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const OBJECT_NOT_FOUND: &str = "OBJECT_NOT_FOUND";
    pub const SECOND_FACTOR_REQUIRED: &str = "SECOND_FACTOR_REQUIRED";
}

#[derive(thiserror::Error, Debug)]

pub enum Error {
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Uri not found: {0}")]
    UriNotFound(String),
}

impl Error {
    pub fn to_response(&self) -> Response {
        match self {
            Error::InternalServerError(_) => Response {
                status: Some(Status {
                    code: error_codes::INTERNAL_SERVER_ERROR.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
            Error::UriNotFound(_) => Response {
                status: Some(Status {
                    code: error_codes::URI_NOT_FOUND.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
        }
    }
}
