mod api;
mod models;
mod repository;
mod webserverconfig;
use dotenv::dotenv;

#[macro_use]
extern crate rocket;
use api::{
    auth::{login, who_am_i},
    user_api::{create_user, delete_user, get_all_users, get_user, update_user},
};
use repository::mongodb_repo::MongoRepo;
use rocket::{get, http::Status, serde::json::Json};
use webserverconfig::cors::CORS;

#[get("/")]
fn hello() -> Result<Json<String>, Status> {
    Ok(Json(String::from("Hello world")))
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let db = MongoRepo::init();
    rocket::build().manage(db).attach(CORS).mount(
        "/",
        routes![
            hello,
            create_user,
            get_user,
            update_user,
            delete_user,
            get_all_users,
            login,
            who_am_i
        ],
    )
}
