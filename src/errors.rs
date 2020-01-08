use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            ServiceError::InternalServerError => HttpResponse::InternalServerError()
                .json("Internal Server Error, Please try later"),
            ServiceError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(message)
            }
            ServiceError::Unauthorized => {
                HttpResponse::Unauthorized().json("Unauthorized")
            }
        }
    }
}

// #[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
// pub struct Error {
//     pub status: String,
//     #[serde(rename = "scimType", default)]
//     pub scim_type: Option<ErrorType>,
//     pub detail: String,
//     #[serde(default = "Error::default_schema")]
//     pub schemas: Vec<String>,
// }

// impl Error {
//     fn new(status: &str, scim_type: Option<ErrorType>, detail: &str) -> Error {
//         Error {
//             status: String::from(status),
//             scim_type: scim_type,
//             detail: String::from(detail),
//             schemas: Error::default_schema(),
//         }
//     }
//     pub fn default_schema() -> Vec<String> {
//         vec![String::from("urn:ietf:params:scim:api:messages:2.0:Error")]
//     }
// }
