use futures::stream::TryStreamExt;
use mongodb::{bson::doc, options::FindOptions};
use rocket_db_pools::{
    Connection, 
    mongodb::bson::Document
};
use rocket::{ 
    response::{
        Flash, 
        Redirect
    }
};
use rocket_dyn_templates::{
    Template, 
    context
};
use pulldown_cmark::{
    Parser, 
    Options, 
    html
};

use crate::MongoDB;

#[get("/")]
pub async fn blog_index(db: Connection<MongoDB>) -> Template {
    let collection = db.database("rocket_blog").collection::<Document>("posts");
    let find_options = FindOptions::builder().sort(doc! { "title": 1 }).build();
    let mut cursor = collection.find(None, find_options).await.unwrap();

    let mut posts = Vec::new();

    while let Some(post) = cursor.try_next().await.unwrap() {
        posts.push(post);
    }

    Template::render("public/blog_index", context! {
        document_title: "Blog posts",
        blog_posts: posts
    })
}

#[get("/post?<uuid>")]
pub async fn view_post(db: Connection<MongoDB>, uuid: Option<&str>) -> Result<Template, Flash<Redirect>> {
    let query = match uuid {
        Some(search) => search,
        None => return Err(Flash::error(Redirect::to(uri!("/")), "Can't view null post."))
    };

    let collection = db.database("rocket_blog").collection::<Document>("posts");
    let filter = doc!{"uuid": query};
    let post = collection.find_one(filter, None).await.unwrap();

    match post {
        Some(view) => {
            let title = view.get_str("title").expect("Missing title!");
            let body = view.get_str("body").expect("Missing body!");

            let mut options = Options::empty();
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TABLES);

            let parser = Parser::new_ext(body, options);
            let mut html_buffer = String::new();
            html::push_html(&mut html_buffer, parser);
            
            Ok(
                Template::render("public/blog", context! {
                    document_title: title,
                    content: html_buffer
                })
            )
        },
        None => Err(Flash::error(Redirect::to(uri!("/")), "That post doesn't exists."))
    }
}