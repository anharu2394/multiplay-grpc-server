use crate::schema::users;
use crate::schema::rooms;
use crate::schema::members;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String
}

#[derive(Queryable)]
pub struct Room {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable)]
pub struct Member {
    pub id: i32,
    pub user_id: i32,
    pub room_id: i32,
}

