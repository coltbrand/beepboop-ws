use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct UserDTO {
//     #[serde(rename = "_id")]
//     pub first_name: String,
//     pub last_name: String,
//     pub email: String,
//     pub password: String,
//     pub role: String,
// }

// // to dto
// impl From<UserDAO> for UserDTO {
//     fn from(user: UserDAO) -> Self {
//         Self {
//             first_name: user.first_name,
//             last_name: user.last_name,
//             email: user.email,
//             password: user.password,
//             role: user.role,
//         }
//     }
// }

// // to dao
// impl From<UserDTO> for UserDAO {
//     fn from(user: UserDTO) -> Self {
//         Self {
//             id: None,
//             first_name: user.first_name,
//             last_name: user.last_name,
//             email: user.email,
//             password: user.password,
//             role: user.role,
//         }
//     }
// }
