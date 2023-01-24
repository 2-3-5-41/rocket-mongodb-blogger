use mongodb::bson::doc;
use rocket_db_pools::{
    Connection, 
    mongodb::bson::Document
};
use rocket::{ 
    response::{
        Flash, 
        Redirect
    }, form::Form
};
use rocket_dyn_templates::{
    Template, 
    context
};
use uuid::Uuid;

use crate::MongoDB;
use crate::guards::admin::AdminAccount;

// Create new post.
#[get("/post/new")]
pub async fn new_post(admin: AdminAccount) -> Template {
    Template::render("private/blog/new", context! {
        document_title: "Create blog post.",
        author: admin.username
    })
}

// Upload new post to database.
#[derive(FromForm)]
pub struct NewPost<'r> {
    title: &'r str,
    description: &'r str,
    body: &'r str,
}

#[post("/post/new", data = "<body>")]
pub async fn upload_post(db: Connection<MongoDB>, admin: AdminAccount, body: Form<NewPost<'_>>) -> Flash<Redirect> {
    let collection = db.database("rocket_blog").collection::<Document>("posts");
    let id = Uuid::new_v4();

    let post = collection.insert_one(doc! {
        "author": admin.username,
        "title": body.title,
        "description": body.description,
        "body": body.body,
        "uuid": id.to_string()
    }, None).await;

    match post {
        Ok(_) => Flash::success(Redirect::to(uri!("/admin/dashboard")), "Created new post."),
        Err(e) => Flash::error(Redirect::to(uri!("/admin", new_post)), e.to_string())
    }
}

// Get contents of post to edit.
#[get("/post/edit?<uuid>")]
pub async fn edit_post(db: Connection<MongoDB>, admin: AdminAccount, uuid: Option<&str>) -> Result<Template, Flash<Redirect>> {
    let collection = db.database("rocket_blog").collection::<Document>("posts");
    let document = collection.find_one(doc!{"uuid": uuid}, None).await.unwrap();

    match document {
        Some(doc) => {
            Ok(
                Template::render("private/blog/edit", context!{
                    document_title: doc.get_str("title").unwrap(),
                    admin: admin.username,
                    uuid: doc.get_str("uuid").unwrap(),
                    title: doc.get_str("title").unwrap(),
                    description: doc.get_str("description").unwrap(),
                    body: doc.get_str("body").unwrap()
                })
            )
        },
        None => Err(Flash::error(Redirect::to(uri!("/admin/dashboard")), "No document to edit."))
    }
}

// Publish updates to post.
#[post("/post/update?<uuid>", data = "<post>")]
pub async fn update_post(db: Connection<MongoDB>, admin: AdminAccount, uuid: Option<&str>, post: Form<NewPost<'_>>) -> Flash<Redirect> {
    let collection = db.database("rocket_blog").collection::<Document>("posts");
    let document = collection.find_one(doc!{"uuid": uuid}, None).await.unwrap();

    match document {
        Some(doc) => {
            collection.replace_one(
                doc, 
                doc! {
                    "author": admin.username,
                    "title": post.title,
                    "description": post.description,
                    "body": post.body,
                    "uuid": Uuid::new_v4().to_string()
                }, 
                None).await.unwrap();
            
            
            Flash::success(Redirect::to(uri!("/admin/dashboard")), format!("Updated {} successfully.", uuid.unwrap()))
        },
        None => Flash::error(Redirect::to(uri!("/admin/dashboard")), "Couldn't find document to udpate.")
    }
}

// Delete post.
#[delete("/post/delete?<uuid>")]
pub async fn delete_post(db: Connection<MongoDB>, admin: AdminAccount, uuid: Option<&str>) -> Flash<Redirect> {
    let collection = db.database("rocket_blog").collection::<Document>("posts");

    match collection.delete_one(doc!{ "uuid": uuid.unwrap() }, None).await {
        Ok(_) => {
            println!("Admin: {} deleted post: {}", admin.username, uuid.unwrap());
            Flash::success(Redirect::to(uri!("/admin/dashboard")), "Successfully deleted post.")
        },
        Err(e) => Flash::error(Redirect::to(uri!("/admin/dashboard")), e.to_string()) 
    }
}