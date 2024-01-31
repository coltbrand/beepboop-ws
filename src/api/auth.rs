use crate::models::jwt::Claims;
use crate::models::login_request::LoginRequest;
use crate::repository::mongodb_repo::MongoRepo;
use chrono::Utc;
use jsonwebtoken::errors::{Error, ErrorKind};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use std::env;

#[post("/login", format = "application/json", data = "<usercreds>")]
pub fn login(db: &State<MongoRepo>, usercreds: Json<LoginRequest>) -> Result<String, Status> {
    let user = db.get_user_by_login(usercreds.into_inner());
    match user {
        Ok(user) => {
            let id = user.id;
            match create_jwt(id.unwrap()) {
                Ok(token) => Ok(token),
                Err(_) => Err(Status::InternalServerError),
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

pub fn create_jwt(id: String) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let ttl = env::var("TOKEN_TTL").expect("TOKEN_TTL must be set.");
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(ttl.parse::<i64>().unwrap()))
        .expect("Invalid timestamp")
        .timestamp();

    let claims = Claims {
        subject_id: id,
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);

    return encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    );
}

pub fn decode_jwt(token: String) -> Result<Claims, ErrorKind> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let token = token.trim_start_matches("Bearer").trim();
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(token) => Ok(token.claims),
        Err(err) => Err(err.kind().to_owned()),
    }
}
