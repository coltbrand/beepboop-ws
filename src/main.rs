mod api;
mod models;
mod repository;

#[macro_use]
extern crate rocket;
use api::user_api::{create_user, delete_user, get_all_users, get_user, update_user};
use repository::mongodb_repo::MongoRepo;
use rocket::{get, http::Status, serde::json::Json};

#[get("/")]
fn hello() -> Result<Json<String>, Status> {
    Ok(Json(String::from("Hello from rust and mongoDB")))
}

#[launch]
fn rocket() -> _ {
    let db = MongoRepo::init();
    rocket::build().manage(db).mount(
        "/",
        routes![
            hello,
            create_user,
            get_user,
            update_user,
            delete_user,
            get_all_users,
        ],
    )
}
