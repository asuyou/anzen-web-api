use super::errors::{self, ErrorJson};
use super::returns::*;
use crate::{model::AnzenDB, routes::state};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::outcome::Outcome::Success;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{json::Json, Serialize};
use rocket::State;
use serde::Deserialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type TextError = errors::APIError<&'static str>;

const MONTH: u64 = 60 * 60 * 24 * 30;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserCred
{
    email: String,
    password: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserRegister
{
    email: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims
{
    pub exp: usize,
    pub sub: String,
}

#[post("/login", data = "<form>")]
pub async fn login(
    form: Json<UserCred>,
    state: &State<state::Validation>,
    db: &State<AnzenDB>,
) -> Result<Json<LoginResponse>, TextError>
{
    let valid = state.inner();
    let db = db.inner();

    if !valid.email_allowed(&form.email).await {
        return Err(errors::APIError::Unauthorized(ErrorJson::new(
            errors::MSG_NO_LOGON_ALLOWED,
        )));
    };

    let valid_user = match db.valid_user(&form.email, &form.password).await {
        Ok(value) => value,
        Err(_) => {
            return Err(errors::APIError::Unauthorized(ErrorJson::new(
                errors::MSG_INVALID_TOKEN,
            )))
        }
    };
    if !valid_user {
        return Err(errors::APIError::Unauthorized(ErrorJson::new(
            errors::MSG_INVALID_PWD,
        )));
    }

    let exp = SystemTime::now()
        .checked_add(Duration::from_secs(MONTH))
        .unwrap();
    let exp = exp.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let claims = Claims {
        exp: exp.try_into().unwrap(),
        sub: form.email.clone(),
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.key.as_ref().as_bytes()),
    ) {
        Ok(v) => v,
        Err(_) => {
            return Err(errors::APIError::Unauthorized(ErrorJson::new(
                errors::MSG_GEN_TOKEN,
            )))
        }
    };

    let response = LoginResponse {
        username: form.email.clone(),
        token,
    };

    Ok(Json(response))
}

#[post("/register", data = "<form>")]
pub async fn register(
    form: Json<UserRegister>,
    state: &State<state::Validation>,
    db: &State<AnzenDB>,
) -> Result<Json<RegisterResponse>, TextError>
{
    let error_user_exists = errors::APIError::Conflict(ErrorJson::new(errors::MSG_USER_EXISTS));
    let valid = state.inner();
    let db = db.inner();

    let mut level: u8 = 2;

    if !valid.email_allowed(&form.email).await {
        return Err(errors::APIError::Unauthorized(ErrorJson::new(
            errors::MSG_NO_LOGON_ALLOWED,
        )));
    };

    if form.username == "admin" {
        level = 0;
    }

    let ok = match db
        .new_user(&form.email, &form.username, level, form.password.clone().as_bytes())
        .await
    {
        Ok(v) => v,
        Err(_) => return Err(error_user_exists),
    };

    if ok {
        return Ok(Json(RegisterResponse { ok: true }));
    }

    Err(error_user_exists)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims
{
    type Error = TextError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Claims, Self::Error>
    {
        let failure = Outcome::Failure((
            Status::Unauthorized,
            errors::APIError::Forbidden(ErrorJson::new(errors::MSG_INVALID_TOKEN)),
        ));

        let state = request.guard::<&State<state::Validation>>().await;
        let state = match state {
            Success(state) => state,
            _ => return failure,
        };

        let auth = match request.headers().get_one("Authorization") {
            Some(v) => v,
            None => return failure,
        };

        let token = match auth.strip_prefix("Bearer ") {
            Some(token) => token.to_string(),
            None => return failure,
        };

        let claims = match jsonwebtoken::decode::<Claims>(
            &token,
            &DecodingKey::from_secret(state.key.as_ref().as_bytes()),
            &Validation::default(),
        ) {
            Ok(claims) => claims,
            Err(_) => return failure,
        };

        if state.email_allowed(&claims.claims.sub).await {
            return Outcome::Success(claims.claims);
        }

        failure
    }
}
