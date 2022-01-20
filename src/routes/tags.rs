use crate::db;
use rocket::serde::json::{serde_json::json, Value};

#[get("/tags")]
pub async fn get_tags(conn: db::Conn) -> Value {
    conn.run(|c| json!({ "tags": db::articles::tags(c) })).await
}
