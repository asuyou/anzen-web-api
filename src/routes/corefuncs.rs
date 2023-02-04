use rocket::serde::json::Value;
use rocket::State;
use serde_json::json;
use super::auth::{Claims, TextError};
use super::state::CoreAPI;
use super::errors::{APIError, ErrorJson};

#[get("/addmail?<email>&<priority>")]
pub async fn addmail(
    claims: Result<Claims, TextError>,
    core_api: &State<CoreAPI>,
    email: String,
    priority: Option<u128>
) -> Result<Value, TextError>
{
    claims?;

    let core_api = core_api.inner();

    let priority = priority.unwrap_or(0);

    match core_api.add_email(email, priority).await {
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

