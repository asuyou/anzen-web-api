use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anzen_lib::anzen;
use anzen_lib::client::ClientRef;

use crate::ResultT;

use serde_json::json;

pub struct Validation
{
    pub key: Arc<String>,
    pub allowed_emails: Arc<HashSet<String>>,
}

impl Validation
{
    pub fn init(key: String, allowed: HashSet<String>) -> Validation
    {
        Validation {
            key: Arc::new(key),
            allowed_emails: Arc::new(allowed),
        }
    }

    pub async fn email_allowed(&self, name: &String) -> bool
    {
        self.allowed_emails.get(name).is_some()
    }
}

pub struct CoreAPI
{
    token: Arc<String>,
    name: Arc<String>,
    client: ClientRef,
}

impl CoreAPI
{
    pub fn init(token: String, client: ClientRef, name: String) -> CoreAPI
    {
        CoreAPI {
            token: Arc::new(token),
            name: Arc::new(name),
            client,
        }
    }

    pub async fn get_stats(&self) -> ResultT<anzen::InfoResponse>
    {
        let mut req = tonic::Request::new(anzen::InfoRequest {});

        anzen_lib::client::insert_authorization(
            &mut req,
            self.token.to_string(),
            self.name.to_string(),
        );

        let mut client = self.client.lock().await;

        let data = client.info(req).await?;

        Ok(data.into_inner())
    }

    pub async fn add_email(&self, email: String, priority: u128) -> ResultT<()> {
        let data = json!({
            "request": "add-email",
            "email": email,
            "priority": priority
        });

        let command = anzen::Command {
            command_type: 0,
            origin: self.name.to_string(),
            data: data.to_string(),
            arm_status: Some(anzen::ArmStatus::Unspecified as i32),
            set_info: HashMap::new()
        };

        self.post_command(command).await
    }

    pub async fn toggle_armed(&self) -> ResultT<()>
    {
        let new_status = match self.get_stats().await?.armed {
            true => anzen::ArmStatus::Disarmed as i32,
            false => anzen::ArmStatus::Armed as i32,
        };

        let command = anzen::Command {
            command_type: 0,
            origin: self.name.to_string(),
            data: "".into(),
            arm_status: Some(new_status),
            set_info: HashMap::new(),
        };

        self.post_command(command).await
    }

    async fn post_command(&self, command: anzen::Command) -> ResultT<()> {

        let mut req = tonic::Request::new(anzen::PostSingleCommandRequest {
            command: Some(command),
        });

        anzen_lib::client::insert_authorization(
            &mut req,
            self.token.to_string(),
            self.name.to_string(),
        );

        let mut client = self.client.lock().await;

        let _resp = client.post_single_command(req).await?;

        Ok(())
    }
}
