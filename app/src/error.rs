use actix_web::ResponseError;
use actix_web::{HttpResponse, http::StatusCode};
use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Serialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum UserError {
    #[display("An internal server error occured. Please try again later.")]
    InternalError,
    #[display("Validation error on field: {field}")]
    ValidationError {
        field: String,
    },
    ClaimNotFound,
    DecisionConflict,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>, // ValidationError
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        let body = match self {
            UserError::ValidationError { field } => ErrorBody {
                error: self.to_string(),
                field: Some(field.clone()),
            },
            UserError::InternalError => ErrorBody {
                error: self.to_string(),
                field: None,
            },
            UserError::ClaimNotFound => ErrorBody {
                error: self.to_string(),
                field: None,
            },
            UserError::DecisionConflict => ErrorBody {
                error: self.to_string(),
                field: None,
            },
        };
        HttpResponse::build(self.status_code()).json(body)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            UserError::ClaimNotFound => StatusCode::BAD_REQUEST,
            UserError::DecisionConflict => StatusCode::BAD_REQUEST,
        }
    }
}
