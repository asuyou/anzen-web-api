use std::collections::HashMap;

use mongodb::bson::Document;
use rocket::serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginResponse
{
    pub username: String,
    pub token: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RegisterResponse
{
    pub ok: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StatsResponse
{
    pub ok: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CoreStatus
{
    pub armed: bool,
    pub store: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct EventCommandN
{
    pub events: Vec<Document>,
    pub commands: Vec<Document>,
}
