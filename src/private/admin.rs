extern crate bcrypt;
use bcrypt::verify;
use rocket_db_pools::{
    Connection, 
    mongodb::bson::Document
};
use mongodb::{bson::doc, options::FindOptions};
use rocket_dyn_templates::{
    Template,
    context
};
use rocket::{
    form::Form, 
    response::{
        Flash, 
        Redirect
    }, 
    http::{
        CookieJar, 
        Cookie
    }, 
    request::FlashMessage, futures::TryStreamExt
};
use crate::MongoDB;
use crate::guards::admin::AdminAccount;

// Rocket form parsing
#[derive(FromForm)]
pub struct AdminAuth<'r> {
    username: &'r str,
    password: &'r str,
}

// API calls
#[get("/login")]
pub async fn admin_form(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("private/admin/login", context!{
        document_title: "Admin Login",
        login_error: flash.map(|flash| format!("{}", flash.message()))
                          .unwrap_or_else(|| "".to_string())
    })
}

#[post("/login", data = "<auth>")]
pub async fn admin_login(auth: Form<AdminAuth<'_>>, db: Connection<MongoDB>, jar: &CookieJar<'_>) -> Result<Redirect, Flash<Redirect>> {    
    let collection = db.database("rocket_blog").collection::<Document>("admins");
    let filter = doc!{"username": &auth.username};
    let pointer = collection.find_one(filter, None).await.unwrap();

    match pointer {
        Some(admin) => {
            if verify(auth.password, admin.get_str("password").expect("Missing password variable.")).unwrap() {
                let admin_access = admin.get_str("admin_id").expect("Missing admin id!");
                jar.add_private(Cookie::new("admin", admin_access.to_owned()));
                
                Ok(Redirect::to(uri!("/admin", admin_dash)))
            } else {
                Err(Flash::error(Redirect::to(uri!("/admin", admin_form)), "Invalid password."))
            }
        },
        None => Err(Flash::error(Redirect::to(uri!("/admin", admin_form)), "No such admin exists."))
    }
}

#[post("/logout")]
pub async fn admin_logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    let admin = jar.get_private("admin");
    
    match admin {
        None => Flash::error(Redirect::to(uri!("/admin", admin_form)), "Can't log you out if you aren't logged in."),
        Some(cookie) => {
            jar.remove_private(cookie);
            return Flash::success(Redirect::to(uri!("/admin", admin_login)), "You've been successfully logged out.");
        }
    }
}

#[get("/dashboard")]
pub async fn admin_dash(db: Connection<MongoDB>, admin: AdminAccount) -> Template {
    // Get collection of all blog posts.
    let blogs_collection = db.database("rocket_blog").collection::<Document>("posts");
    let find_options = FindOptions::builder().sort(doc! { "title": 1 }).build();
    let mut blogs_cursor = blogs_collection.find(None, find_options).await.unwrap();
    let mut blogs = Vec::new();

    while let Some(blog) = blogs_cursor.try_next().await.unwrap() {
        blogs.push(blog);
    }

    // Render the page.
    Template::render("private/admin/dashboard", context!{
        document_title: "Admin dashboard",
        admin: admin.username,
        blog_posts: blogs
    })
}