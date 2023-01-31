#[macro_use] extern crate rocket;
use rocket::fs::{
    FileServer,
    relative
};
use rocket_db_pools::{
    mongodb,
    Database
};
use rocket_dyn_templates::Template;

// APIs
mod public;
mod private;
mod guards;

// MongoDB Database
#[derive(Database)]
#[database("mongodb")]
pub struct MongoDB (mongodb::Client);

#[launch]
fn rocket() -> _ {
    rocket::build()
        // Database connection
        .attach(MongoDB::init())

        // Server side rendering
        .attach(Template::fairing())
        .mount("/public", FileServer::from(relative!("public")))

        // Routes
        .mount("/", routes![public::index])
        .mount("/admin", routes![
            private::admin::admin_checkpoint,
            private::admin::admin_auth,
            private::admin::admin_login,
            private::admin::admin_logout,
            private::admin::admin_dash,
            private::blog::new_post,
            private::blog::upload_post,
            private::blog::edit_post,
            private::blog::update_post,
            private::blog::delete_post
        ])
        .mount("/blog", routes![
            public::blog::view_post,
            public::blog::blog_index
        ])
}