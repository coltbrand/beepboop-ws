use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub value: String,
}
