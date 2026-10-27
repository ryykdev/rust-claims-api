use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Serialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum CustomError {
    IngestFail,
    ServerError,
    DbSaveFail,
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    error_type: String,
}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, error_msg) = match self {
            CustomError::IngestFail => {
                log::error!("Ingest failure: {}", self);
                (500, self.to_string())
            }
            _ => {
                log::error!("Fallback error: {}", self);
                (500, self.to_string())
            }
        };

        let error_response = ErrorResponse {
            code: status_code,
            message: error_msg,
            error_type: format!("{:?}", self),
        };

        HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
            .json(error_response)
    }
}
