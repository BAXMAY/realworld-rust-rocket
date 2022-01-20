use crate::auth::Auth;
use crate::config::AppState;
use crate::db::{self, users::UserCreationError};
use crate::errors::{Errors, FieldValidator};

use rocket::serde::json::{serde_json::json, Json, Value};
use rocket::State;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize)]
pub struct NewUser {
    user: NewUserData,
}

#[derive(Deserialize, Validate)]
struct NewUserData {
    #[validate(length(min = 1))]
    username: Option<String>,
    #[validate(email)]
    email: Option<String>,
    #[validate(length(min = 8))]
    password: Option<String>,
}

#[post("/users", format = "json", data = "<new_user>")]
pub async fn post_users(
    new_user: Json<NewUser>,
    conn: db::Conn,
    state: &State<AppState>,
) -> Result<Value, Errors> {
    let new_user = new_user.into_inner().user;

    let mut extractor = FieldValidator::validate(&new_user);
    let username = extractor.extract("username", new_user.username);
    let email = extractor.extract("email", new_user.email);
    let password = extractor.extract("password", new_user.password);

    extractor.check()?;

    let secret = state.secret.to_owned();

    conn.run(move |c| {
        db::users::create(c, &username, &email, &password)
            .map(|user| json!({ "user": user.to_user_auth(&secret) }))
            .map_err(|error| {
                let field = match error {
                    UserCreationError::DuplicatedEmail => "email",
                    UserCreationError::DuplicatedUsername => "username",
                };
                Errors::new(&[(field, "has already been taken")])
            })
    })
    .await
}

#[derive(Deserialize)]
pub struct LoginUser {
    user: LoginUserData,
}

#[derive(Deserialize)]
struct LoginUserData {
    email: Option<String>,
    password: Option<String>,
}

#[post("/users/login", format = "json", data = "<user>")]
pub async fn post_users_login(
    user: Json<LoginUser>,
    conn: db::Conn,
    state: &State<AppState>,
) -> Result<Value, Errors> {
    let user = user.into_inner().user;

    let mut extractor = FieldValidator::default();
    let email = extractor.extract("email", user.email);
    let password = extractor.extract("password", user.password);
    extractor.check()?;

    let secret = state.secret.to_owned();

    conn.run(move |c| {
        db::users::login(c, &email, &password)
            .map(|user| json!({ "user": user.to_user_auth(&secret) }))
            .ok_or_else(|| Errors::new(&[("email or password", "is invalid")]))
    })
    .await
}

#[get("/user")]
pub async fn get_user(auth: Auth, conn: db::Conn, state: &State<AppState>) -> Option<Value> {
    let secret = state.secret.to_owned();

    conn.run(move |c| {
        db::users::find(c, auth.id).map(|user| json!({ "user": user.to_user_auth(&secret) }))
    })
    .await
}

#[derive(Deserialize)]
pub struct UpdateUser {
    user: db::users::UpdateUserData,
}

#[put("/user", format = "json", data = "<user>")]
pub async fn put_user(
    user: Json<UpdateUser>,
    auth: Auth,
    conn: db::Conn,
    state: &State<AppState>,
) -> Option<Value> {
    let secret = state.secret.to_owned();

    conn.run(move |c| {
        db::users::update(c, auth.id, &user.user)
            .map(|user| json!({ "user": user.to_user_auth(&secret) }))
    })
    .await
}
