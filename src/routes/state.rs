use std::sync::Arc;
use std::collections::HashSet;

pub struct Validation {
    pub key: Arc<String>,
    pub allowed_names: Arc<HashSet<String>>,
}

impl Validation {
    pub fn init(key: String, allowed: HashSet<String>) -> Validation 
    {
        Validation {
            key: Arc::new(key),
            allowed_names: Arc::new(allowed)
        }
    }

    pub async fn name_allowed(&self, name: &String) -> bool {
        self.allowed_names.get(name).is_some()
    }
}

