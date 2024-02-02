use rocket::serde::Serialize;

#[derive(Serialize)]
pub enum ResponseBody {
    Message(String),
    AuthToken(String),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub body: ResponseBody,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expiration: i64,
}
