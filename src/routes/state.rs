use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anzen_lib::anzen;
use anzen_lib::ClientRef;

use crate::ResultT;

pub struct Validation {
    pub key: Arc<String>,
    pub allowed_names: Arc<HashSet<String>>,
}

impl Validation {
    pub fn init(key: String, allowed: HashSet<String>) -> Validation {
        Validation {
            key: Arc::new(key),
            allowed_names: Arc::new(allowed),
        }
    }

    pub async fn name_allowed(&self, name: &String) -> bool {
        self.allowed_names.get(name).is_some()
    }
}

pub struct CoreAPI {
    token: Arc<String>,
    name: Arc<String>,
    client: ClientRef,
}

impl CoreAPI {
    pub fn init(token: String, client: ClientRef, name: String) -> CoreAPI {
        CoreAPI {
            token: Arc::new(token),
            name: Arc::new(name),
            client,
        }
    }

    pub async fn get_stats(&self) -> ResultT<anzen::InfoResponse> {
        let mut req = tonic::Request::new(anzen::InfoRequest {});

        anzen_lib::insert_authorization(&mut req, self.token.to_string(), self.name.to_string());

        let mut client = self.client.lock().await;

        let data = client.info(req).await?;

        Ok(data.into_inner())
    }

    pub async fn toggle_armed(&self) -> ResultT<()> {
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

        let mut req = tonic::Request::new(anzen::PostSingleCommandRequest {
            command: Some(command),
        });

        anzen_lib::insert_authorization(&mut req, self.token.to_string(), self.name.to_string());

        let mut client = self.client.lock().await;

        let _resp = client.post_single_command(req).await?;

        Ok(())
    }
}
