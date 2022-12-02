use rocket::serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginResponse {
    pub username: String,
    pub token: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RegisterResponse {
    pub ok: bool
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StatsResponse {
    pub ok: bool
}

