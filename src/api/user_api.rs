use crate::{
    models::{jwt::JWT, network_response::NetworkResponse, user_model::User},
    repository::mongodb_repo::MongoRepo,
};
use mongodb::{bson::oid::ObjectId, results::InsertOneResult};
use rocket::{http::Status, serde::json::Json, State};

#[post("/user", data = "<new_user>")]
pub fn create_user(
    key: Result<JWT, NetworkResponse>,
    db: &State<MongoRepo>,
    new_user: Json<User>,
) -> Result<Json<InsertOneResult>, NetworkResponse> {
    let key = key?;
    let data = User {
        id: None,
        first_name: new_user.first_name.to_owned(),
        last_name: new_user.last_name.to_owned(),
        email: new_user.email.to_owned(),
        password: new_user.password.to_owned(),
        role: new_user.role.to_owned(),
    };
    let user_detail = db.create_user(data);
    match user_detail {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(NetworkResponse::BadRequest(
            "Error retrieving user details.".to_owned(),
        )),
    }
}

#[get("/user/<oid>")]
pub fn get_user(db: &State<MongoRepo>, oid: String) -> Result<Json<User>, Status> {
    let id = oid;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let user_detail = db.get_user(&id);
    match user_detail {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[put("/user/<oid>", data = "<new_user>")]
pub fn update_user(
    db: &State<MongoRepo>,
    oid: String,
    new_user: Json<User>,
) -> Result<Json<User>, Status> {
    let id = oid;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let data = User {
        id: Some(ObjectId::parse_str(&id).unwrap()),
        first_name: new_user.first_name.to_owned(),
        last_name: new_user.last_name.to_owned(),
        email: new_user.email.to_owned(),
        password: new_user.password.to_owned(),
        role: new_user.role.to_owned(),
    };
    let update_result = db.update_user(&id, data);
    match update_result {
        Ok(update) => {
            if update.matched_count == 1 {
                let updated_user_info = db.get_user(&id);
                return match updated_user_info {
                    Ok(user) => Ok(Json(user)),
                    Err(_) => Err(Status::InternalServerError),
                };
            } else {
                return Err(Status::NotFound);
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[delete("/user/<oid>")]
pub fn delete_user(db: &State<MongoRepo>, oid: String) -> Result<Json<&str>, Status> {
    let id = oid;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let result = db.delete_user(&id);
    match result {
        Ok(res) => {
            if res.deleted_count == 1 {
                return Ok(Json("User successfully deleted!"));
            } else {
                return Err(Status::NotFound);
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/user")]
pub fn get_all_users(db: &State<MongoRepo>) -> Result<Json<Vec<User>>, Status> {
    let users = db.get_all_users();
    match users {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err(Status::InternalServerError),
    }
}
