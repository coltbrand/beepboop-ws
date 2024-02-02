use crate::models::jwt::{Claims, JWT};
use crate::models::login_request::LoginRequest;
use crate::models::login_response::{Response, ResponseBody, TokenResponse};
use crate::models::network_response::NetworkResponse;
use crate::models::user_model::User;
use crate::models::who_am_i_response::{self, WhoAmIResponse};
use crate::repository::mongodb_repo::MongoRepo;
use chrono::Utc;
use jsonwebtoken::errors::Error;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::State;
use std::borrow::Borrow;
use std::env;

use super::user_api::get_user;

#[post("/login", format = "application/json", data = "<usercreds>")]
pub fn login(
    db: &State<MongoRepo>,
    usercreds: Json<LoginRequest>,
) -> Result<Json<TokenResponse>, Status> {
    let user = db.get_user_by_login(usercreds.into_inner());
    match user {
        Ok(user) => {
            let id = user.id;
            match create_jwt(id.unwrap(), user.permissions) {
                Ok(token) => Ok(Json(token)),
                Err(_) => Err(Status::InternalServerError),
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/whoAmI", format = "application/json")]
pub fn who_am_i(
    db: &State<MongoRepo>,
    key: Result<JWT, NetworkResponse>,
) -> Result<Json<WhoAmIResponse>, NetworkResponse> {
    let key = key?;
    let user = db.get_user(key.claims.user_id);
    match user {
        Ok(user) => Ok(Json(WhoAmIResponse {
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            permissions: user.permissions,
        })),
        Err(_) => Err(NetworkResponse::Unauthorized(
            "User is unauthorized.".to_owned(),
        )),
    }
}

pub fn create_jwt(id: String, permissions: Vec<String>) -> Result<TokenResponse, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let token_ttl = env::var("TOKEN_TTL").expect("TOKEN_TTL must be set.");
    let refresh_ttl = env::var("REFRESH_TTL").expect("REFRESH_TTL must be set.");
    let token_expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(token_ttl.parse::<i64>().unwrap()))
        .expect("Invalid timestamp")
        .timestamp();
    let refresh_expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(
            refresh_ttl.parse::<i64>().unwrap(),
        ))
        .expect("Invalid timestamp")
        .timestamp();

    let claims = Claims {
        user_id: id.clone(),
        permissions: permissions,
        exp: token_expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    );
    let refresh_claims = Claims {
        user_id: id,
        permissions: vec!["N/A".to_owned()],
        exp: refresh_expiration as usize,
    };
    let refresh = encode(
        &header,
        &refresh_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    );
    match token {
        Ok(s) => Ok(TokenResponse {
            access_token: s,
            refresh_token: refresh.unwrap(),
            expiration: token_expiration,
        }),
        Err(err) => Err(err),
    }
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = NetworkResponse;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, NetworkResponse> {
        fn is_valid(key: &str) -> Result<Claims, Error> {
            Ok(decode_jwt(String::from(key))?)
        }

        match req.headers().get_one("authorization") {
            None => {
                let response = Response {
                    body: ResponseBody::Message(String::from(
                        "Error validating JWT token - No token provided",
                    )),
                };
                rocket::request::Outcome::Error((
                    Status::BadRequest,
                    NetworkResponse::BadRequest(serde_json::to_string(&response).unwrap()),
                ))
            }
            Some(key) => match is_valid(key) {
                Ok(claims) => Outcome::Success(JWT { claims }),
                Err(err) => match &err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        let response = Response {
                            body: ResponseBody::Message(format!(
                                "Error validating JWT token - Expired Token"
                            )),
                        };
                        Outcome::Error((
                            Status::BadRequest,
                            NetworkResponse::BadRequest(serde_json::to_string(&response).unwrap()),
                        ))
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        let response = Response {
                            body: ResponseBody::Message(format!(
                                "Error validating JWT token - Invalid Token"
                            )),
                        };
                        Outcome::Error((
                            Status::BadRequest,
                            NetworkResponse::BadRequest(serde_json::to_string(&response).unwrap()),
                        ))
                    }
                    _ => {
                        let response = Response {
                            body: ResponseBody::Message(format!(
                                "Error validating JWT token - {}",
                                err
                            )),
                        };
                        Outcome::Error((
                            Status::BadRequest,
                            NetworkResponse::BadRequest(serde_json::to_string(&response).unwrap()),
                        ))
                    }
                },
            },
        }
    }
}

pub fn check_permission(token: JWT, permission_to_check: String) -> bool {
    let claims = token.claims;
    let permissions = claims.permissions;

    return permissions.contains(&permission_to_check);
}
