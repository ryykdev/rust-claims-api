use actix_web::ResponseError;
use actix_web::{HttpResponse, http::StatusCode};
use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Serialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum UserError {
    #[display("An internal server error occured. Please try again later.")]
    InternalError,
    ClaimNotFound,
    DecisionConflict,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        let body = match self {
            UserError::InternalError => ErrorBody {
                error: self.to_string(),
            },
            UserError::ClaimNotFound => ErrorBody {
                error: self.to_string(),
            },
            UserError::DecisionConflict => ErrorBody {
                error: self.to_string(),
            },
        };
        HttpResponse::build(self.status_code()).json(body)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::ClaimNotFound => StatusCode::BAD_REQUEST,
            UserError::DecisionConflict => StatusCode::BAD_REQUEST,
        }
    }
}
