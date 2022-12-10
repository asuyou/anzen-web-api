use rocket::serde::json::Json;
use rocket::serde::Serialize;

pub const MSG_NO_LOGON_ALLOWED: &str = "User logon is not currently allowed";
pub const MSG_INVALID_PWD: &str = "Could not validate password";
pub const MSG_GEN_TOKEN: &str = "Could not generate token";
pub const MSG_INVALID_TOKEN: &str = "Invalid bearer token";
pub const MSG_USER_EXISTS: &str = "User already exists";
pub const MSG_INTERNAL_DB_ERR: &str = "There was an internal DB error";
pub const MSG_INTERNAL_CORE_ERR: &str = "Core endpoints are offline";

#[derive(Debug, Responder, Serialize)]
#[serde(crate = "rocket::serde")]
#[response(content_type = "json")]
pub struct ErrorJson<T> {
    error: T,
}

impl ErrorJson<&'static str> {
    pub fn new(error: &'static str) -> Json<Self> {
        Json(Self { error })
    }
}

#[derive(Debug, Responder)]
pub enum Error<R> {
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorJson<R>>),
    #[response(status = 403, content_type = "json")]
    Forbidden(Json<ErrorJson<R>>),
    #[response(status = 409, content_type = "json")]
    Conflict(Json<ErrorJson<R>>),
    #[response(status = 500, content_type = "json")]
    Internal(Json<ErrorJson<R>>),
}
