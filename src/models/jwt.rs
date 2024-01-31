use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: String,
    pub permissions: Vec<String>,
    pub exp: usize,
}

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims,
}
