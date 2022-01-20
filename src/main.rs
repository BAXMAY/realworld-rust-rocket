use realworld;
use rocket::launch;

#[launch]
async fn rocket() -> _ {
    realworld::rocket()
}
