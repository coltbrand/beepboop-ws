use std::env;

use crate::models::{login_request::LoginRequest, user_model::User};
use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    sync::{Client, Collection},
};

pub struct MongoRepo {
    user_col: Collection<User>,
}

impl MongoRepo {
    pub fn init() -> Self {
        let uri = env::var("MONGO_URI").expect("MONGO_URI must be set.");
        let client = Client::with_uri_str(uri).unwrap();
        let auth_db = client.database("auth");
        let user_col: Collection<User> = auth_db.collection("users");
        MongoRepo { user_col }
    }

    pub fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: None,
            first_name: new_user.first_name,
            last_name: new_user.last_name,
            email: new_user.email,
            password: new_user.password,
            role: new_user.role,
        };
        let user = self
            .user_col
            .insert_one(new_doc, None)
            .ok()
            .expect("Error creating user");
        Ok(user)
    }

    pub fn get_user(&self, id: &String) -> Result<User, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let user_detail = self
            .user_col
            .find_one(filter, None)
            .ok()
            .expect("Error getting user's detail");
        Ok(user_detail.unwrap())
    }

    pub fn update_user(&self, id: &String, new_user: User) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let new_doc = doc! {
            "$set":
                {
                    "id": new_user.id,
                    "first_name": new_user.first_name,
                    "last_name": new_user.last_name,
                    "email": new_user.email,
                    "password": new_user.password,
                    "role": new_user.role,
                },
        };
        let updated_doc = self
            .user_col
            .update_one(filter, new_doc, None)
            .ok()
            .expect("Error updating user");
        Ok(updated_doc)
    }

    pub fn delete_user(&self, id: &String) -> Result<DeleteResult, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let user_detail = self
            .user_col
            .delete_one(filter, None)
            .ok()
            .expect("Error deleting user");
        Ok(user_detail)
    }

    pub fn get_all_users(&self) -> Result<Vec<User>, Error> {
        let cursors = self
            .user_col
            .find(None, None)
            .ok()
            .expect("Error getting list of users");
        let users = cursors.map(|doc| doc.unwrap()).collect();
        Ok(users)
    }

    pub fn get_user_by_login(&self, login: LoginRequest) -> Result<User, Error> {
        let filter = doc! {"$and": [{"email": login.username},{"password": login.password}]};
        let user_detail = self
            .user_col
            .find_one(filter, None)
            .ok()
            .expect("Error getting user's detail");
        Ok(user_detail.unwrap())
    }
}
