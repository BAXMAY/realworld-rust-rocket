use crate::auth::Auth;
use crate::db;
use crate::models::user::Profile;
use rocket::serde::json::{serde_json::json, Value};

fn to_profile_json(profile: Profile) -> Value {
    json!({ "profile": profile })
}

#[get("/profiles/<username>")]
pub async fn get_profile(username: String, auth: Option<Auth>, conn: db::Conn) -> Option<Value> {
    let user_id = auth.map(|auth| auth.id);
    conn.run(move |c| db::profiles::find(c, &username, user_id).map(to_profile_json))
        .await
}

#[post("/profiles/<username>/follow")]
pub async fn follow(username: String, auth: Auth, conn: db::Conn) -> Option<Value> {
    conn.run(move |c| db::profiles::follow(c, &username, auth.id).map(to_profile_json))
        .await
}

#[delete("/profiles/<username>/follow")]
pub async fn unfollow(username: String, auth: Auth, conn: db::Conn) -> Option<Value> {
    conn.run(move |c| db::profiles::unfollow(c, &username, auth.id).map(to_profile_json))
        .await
}
