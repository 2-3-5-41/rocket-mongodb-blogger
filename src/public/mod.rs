use rocket_dyn_templates::{Template, context};

pub mod blog;

#[get("/")]
pub async fn index() -> Template {
    Template::render("public/index", context!{
        document_title: "Hello, world!"
    })
}