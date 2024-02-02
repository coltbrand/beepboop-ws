use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WhoAmIResponse {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub permissions: Vec<String>,
}
