use rocket::serde::json::Value;
use rocket::State;
use serde_json::json;
use super::auth::{Claims, TextError};
use super::state::CoreAPI;
use super::errors::{APIError, ErrorJson};

use serde::Deserialize;
use rocket::serde::json::Json;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EmailForm
{
    email: String,
    priority: Option<u128>
}

#[post("/addmail", data = "<form>")]
pub async fn addmail(
    claims: Result<Claims, TextError>,
    core_api: &State<CoreAPI>,
    form: Json<EmailForm>
) -> Result<Value, TextError>
{
    claims?;

    let core_api = core_api.inner();

    let priority = form.priority.unwrap_or(0);

    match core_api.add_email(form.email.clone(), priority).await {
        Ok(_) => {
            Ok(json!({
                "ok": true
            }))
        },
        Err(_) => {
            Err(APIError::Internal(ErrorJson::new(
                "Could not send request to update email"
            )))
        }
    }
}

