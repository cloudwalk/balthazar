use std::collections::HashMap;

use crate::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Message {
    pub topic: String,
    pub key: String,
    pub payload: String,
    pub headers: HashMap<String, String>,
}
