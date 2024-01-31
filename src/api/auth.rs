use crate::models::jwt::{Claims, JWT};
use crate::models::login_request::LoginRequest;
use crate::models::login_response::{Response, ResponseBody};
use crate::models::network_response::NetworkResponse;
use crate::repository::mongodb_repo::MongoRepo;
use chrono::Utc;
use jsonwebtoken::errors::Error;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::State;
use std::env;

#[post("/login", format = "application/json", data = "<usercreds>")]
pub fn login(db: &State<MongoRepo>, usercreds: Json<LoginRequest>) -> Result<String, Status> {
    let user = db.get_user_by_login(usercreds.into_inner());
    match user {
        Ok(user) => {
            let id = user.id;
            match create_jwt(id.unwrap(), user.permissions) {
                Ok(token) => Ok(token),
                Err(_) => Err(Status::InternalServerError),
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

pub fn create_jwt(id: String, permissions: Vec<String>) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let ttl = env::var("TOKEN_TTL").expect("TOKEN_TTL must be set.");
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(ttl.parse::<i64>().unwrap()))
        .expect("Invalid timestamp")
        .timestamp();

    let claims = Claims {
        user_id: id,
        permissions: permissions,
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
