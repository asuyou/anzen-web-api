use super::auth::{Claims, TextError};
use super::errors::{self, Error, ErrorJson};
use super::returns::CoreStatus;
use super::state::CoreAPI;
use crate::model::AnzenDB;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;
use rocket::State;

#[get("/test")]
pub async fn test(claims: Result<Claims, TextError>) -> Result<String, TextError> {
    Ok(claims?.sub)
}

#[get("/stats")]
pub async fn stats(
    claims: Result<Claims, TextError>,
    db: &State<AnzenDB>,
    core_api: &State<CoreAPI>,
) -> Result<Value, TextError> {
    claims?;

    let db_fail = Error::Internal(ErrorJson::new(errors::MSG_INTERNAL_DB_ERR));
    let core_fail = Error::Internal(ErrorJson::new(errors::MSG_INTERNAL_CORE_ERR));

    let db = db.inner();
    let core_api = core_api.inner();

    let event_stats = match db.event_statistics().await {
        Ok(v) => v,
        Err(_) => return Err(db_fail),
    };

    let hourly_totals = match db.count_status_time().await {
        Ok(v) => v,
        Err(_) => return Err(db_fail),
    };

    let core_status = match core_api.get_stats().await {
        Ok(v) => v,
        Err(_) => return Err(core_fail),
    };

    let last_n = match db.last_n(10).await {
        Ok(v) => v,
        Err(_) => return Err(core_fail),
    };

    // Must change to be able to serialize

    let core_status = CoreStatus {
        armed: core_status.armed,
        store: core_status.values,
    };

    Ok(json!({
        "data": {
            "hourlyTotals": hourly_totals,
            "eventStats": event_stats,
            "coreStatus": core_status,
            "lastCE": last_n
        }
    }))
}

#[post("/toggle")]
pub async fn toggle(
    claims: Result<Claims, TextError>,
    core_api: &State<CoreAPI>,
) -> Result<Value, TextError> {
    claims?;

    let core_api = core_api.inner();

    match core_api.toggle_armed().await {
        Ok(_) => Ok(json!({
            "ok": true
        })),
        Err(_) => Err(Error::Internal(ErrorJson::new(
            "Could not toggle arm status",
        ))),
    }
}
