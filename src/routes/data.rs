use super::auth::{Claims, TextError};
use crate::model::AnzenDB;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::{Json, Value};
use rocket::State;
use super::returns::*;
use super::errors::{Error, ErrorJson, self};

#[get("/test")]
pub async fn test(
    claims: Result<Claims, TextError>
) -> Result<String, TextError> {
    Ok(claims?.sub)
}

#[get("/stats")]
pub async fn stats(
    claims: Result<Claims, TextError>,
    db: &State<AnzenDB>
    ) -> Result<Value, TextError>
{
    claims?;

    let db_fail = Err(Error::Internal(ErrorJson::new(errors::MSG_INTERNAL_DB_ERR)));

    let db = db.inner();

    let event_stats = match db.event_statistics().await {
        Ok(v) => v,
        Err(_) => return db_fail
    };

    let hourly_totals = match db.count_status_time().await {
        Ok(v) => v,
        Err(_) => return db_fail
    };

    Ok(json!({
        "data": {
            "hourlyTotals": hourly_totals,
            "eventStats": event_stats
        }
    }))
}

