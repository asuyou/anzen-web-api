use rocket::serde::json::Value;
use rocket::State;
use serde_json::json;
use super::{auth::{Claims, TextError}, errors::{APIError, ErrorJson, self}};
use crate::model::AnzenDB;


use serde::Deserialize;
use rocket::serde::json::Json;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PasswordForm
{
    password: String,
}

#[get("/user")]
pub async fn user(
    claims: Result<Claims, TextError>,
    db: &State<AnzenDB>,
) -> Result<Value, TextError>
{
    let email = claims?.sub;

    let db = db.inner();

    let user = db.get_user(&email).await.unwrap();

    Ok(json!({
        "data": {
            "username": user.username,
            "email": user.email,
            "created": user.created,
            "level": user.level
        }
    }))
}

#[post("/updatepassword", data = "<form>")]
pub async fn updatepassword(
    claims: Result<Claims, TextError>,
    db: &State<AnzenDB>,
    form: Json<PasswordForm>,
) -> Result<Value, TextError>
{
    let email = claims?.sub;

    let db = db.inner();

    match db.change_password(&email, form.password.as_bytes()).await {
        Ok(outcome) => Ok(json!({ "ok": outcome })),
        Err(_) => Err(errors::APIError::Internal(ErrorJson::new(
            "Internal error updating password"
        )))
    }
}

#[get("/users")]
pub async fn users(
    claims: Result<Claims, TextError>,
    db: &State<AnzenDB>,
) -> Result<Value, TextError>
{
    let email = claims?.sub;

    let db = db.inner();

    let user = db.get_user(&email).await.unwrap();

    if user.level != 0 {
        return Err(APIError::Forbidden(ErrorJson::new(
            "Must have admin credentials to access all users"
        )))
    }

    Ok(json!({
        "data": {
            "username": user.username,
            "email": user.email,
            "created": user.created,
            "level": user.level
        }
    }))
}

