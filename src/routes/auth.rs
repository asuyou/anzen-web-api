use crate::{routes::state, model::AnzenDB};
use rocket::request::{ Outcome, Request, FromRequest};
use rocket::http::Status;
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, Validation};
use rocket::outcome::Outcome::Success;
use rocket::serde::{json::Json, Serialize};
use rocket::State;
use serde::Deserialize;
use super::errors::{self, ErrorJson};
use super::returns::*;

pub type TextError = errors::Error<&'static str>;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserCred {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub sub: String
} 

#[post("/login", data = "<form>")]
pub async fn login(
    form: Json<UserCred>,
    state: &State<state::Validation>,
    db: &State<AnzenDB>
) -> Result<Json<LoginResponse>, TextError> {
    let valid = state.inner();
    let db = db.inner();

    if !valid.name_allowed(&form.username).await
    {
        return Err(errors::Error::Unauthorized(ErrorJson::new(errors::MSG_NO_LOGON_ALLOWED)));
    };

    let valid_user = match db.valid_user(&form.username, &form.password).await {
        Ok(value) => value,
        Err(_) => return Err(errors::Error::Unauthorized(ErrorJson::new(errors::MSG_INVALID_TOKEN)))
    };
    if !valid_user
    {
        return Err(errors::Error::Unauthorized(ErrorJson::new(errors::MSG_INVALID_PWD)))
    }

    let claims = Claims {
        exp: 1_000_000_000_000,
        sub: form.username.clone()
    };

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(state.key.as_ref().as_bytes())) {
        Ok(v) => v,
        Err(_) => return Err(errors::Error::Unauthorized(ErrorJson::new(errors::MSG_GEN_TOKEN)))
    };

    let response = LoginResponse {
        username: form.username.clone(),
        token
    };

    Ok(Json(response))
}

#[post("/register", data = "<form>")]
pub async fn register(
    form: Json<UserCred>,
    state: &State<state::Validation>,
    db: &State<AnzenDB>
) -> Result<Json<RegisterResponse>, TextError> {
    let error_user_exists = errors::Error::Conflict(ErrorJson::new(errors::MSG_USER_EXISTS));
    let valid = state.inner();
    let db = db.inner();

    if !valid.name_allowed(&form.username).await
    {
        return Err(errors::Error::Unauthorized(ErrorJson::new(errors::MSG_NO_LOGON_ALLOWED)));
    };

    let ok = match db.new_user(&form.username, form.password.clone().as_bytes()).await {
        Ok(v) => v,
        Err(_) => return Err(error_user_exists)
    };

    if ok {
        return Ok(Json(RegisterResponse { ok: true }));
    } 

    Err(error_user_exists)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = TextError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Claims, Self::Error> {
        println!("Here");
        let failure = Outcome::Failure((Status::Unauthorized, errors::Error::Forbidden(ErrorJson::new(errors::MSG_INVALID_TOKEN))));

        let state = request.guard::<&State<state::Validation>>().await;
        let state = match state {
            Success(state) => state,
            _ => return failure
        };


        let auth = match request.headers().get_one("Authorization") {
            Some(v) => v,
            None => return failure
        };

        let token = match auth.strip_prefix("Bearer ") {
            Some(token) => token.to_string(),
            None => return failure
        };

        let claims = match jsonwebtoken::decode::<Claims>(&token, &DecodingKey::from_secret(state.key.as_ref().as_bytes()), &Validation::default()) {
            Ok(claims) => claims,
            Err(_) => return failure
        };

        if state.name_allowed(&claims.claims.sub).await {
            return Outcome::Success(claims.claims);
        }

        failure
    }
}

