use std::{borrow::Borrow, env};

use crate::models::{login_request::LoginRequest, permission::Permission, user_model::User};
use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    sync::{Client, Collection},
};
use uuid::Uuid;

pub struct MongoRepo {
    user_col: Collection<User>,
    permission_col: Collection<Permission>,
}

impl MongoRepo {
    pub fn init() -> Self {
        let uri = env::var("MONGO_URI").expect("MONGO_URI must be set.");
        let client = Client::with_uri_str(uri).unwrap();
        let auth_db = client.database("auth");
        let user_col: Collection<User> = auth_db.collection("users");
        let permission_col: Collection<Permission> = auth_db.collection("permissions");
        MongoRepo {
            user_col,
            permission_col,
        }
    }

    pub fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let filtered_permissions = self
            .validate_permissions(new_user.permissions)
            .expect("Could not validate permissions list.");
        let new_doc = User {
            id: Some(Uuid::new_v4().to_string()),
            first_name: new_user.first_name,
            last_name: new_user.last_name,
            email: new_user.email,
            password: new_user.password,
            permissions: filtered_permissions,
        };
        let user = self
            .user_col
            .insert_one(new_doc, None)
            .ok()
            .expect("Error creating user");
        Ok(user)
    }

    pub fn get_user(&self, id: String) -> Result<User, Error> {
        let obj_id = id;
        let filter = doc! {"_id": obj_id};
        let user_detail = self
            .user_col
            .find_one(filter, None)
            .ok()
            .expect("Error getting user's detail");
        Ok(user_detail.unwrap())
    }

    pub fn update_user(&self, new_user: User) -> Result<UpdateResult, Error> {
        let id = new_user.id;
        let filter = doc! {"_id": &id};
        let filtered_permissions = self
            .validate_permissions(new_user.permissions)
            .expect("Could not validate permissions list.");
        let new_doc = doc! {
            "$set":
                {
                    "id": id,
                    "first_name": new_user.first_name,
                    "last_name": new_user.last_name,
                    "email": new_user.email,
                    "password": new_user.password,
                    "permissions": filtered_permissions,
                },
        };
        let updated_doc = self
            .user_col
            .update_one(filter, new_doc, None)
            .ok()
            .expect("Error updating user");
        Ok(updated_doc)
    }

    pub fn delete_user(&self, id: String) -> Result<DeleteResult, Error> {
        let filter = doc! {"_id": id};
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

    pub fn find_permission(&self, value: String) -> Result<Option<Permission>, Error> {
        let filter = doc! {"value": value};
        let permission_detail = self
            .permission_col
            .find_one(filter, None)
            .ok()
            .expect("Error getting permission's detail");
        Ok(permission_detail)
    }

    pub fn validate_permissions(&self, permissions: Vec<String>) -> Result<Vec<String>, Error> {
        let filterd_permissions = permissions
            .into_iter()
            .filter(|x| self.filter_permissions(x.to_string()))
            .collect();
        Ok(filterd_permissions)
    }

    fn filter_permissions(&self, permission: String) -> bool {
        match self.find_permission(permission) {
            Ok(detail) => {
                if let Some(_) = detail {
                    return true;
                } else {
                    return false;
                }
            }
            Err(_) => false,
        }
    }
}
