use mongodb::bson::{doc, Document};
use rocket::{request::{ 
    Outcome, 
    Request, 
    FromRequest
}
};
use rocket_db_pools::Connection;

use crate::MongoDB;

#[derive(Clone, Debug)]
pub struct AdminAccount{
    pub admin_id: String,
    pub username: String
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAccount {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cached_result = req.local_cache_async(
            async {
                let connection = req.guard::<Connection<MongoDB>>().await.unwrap();
                let collection = connection.database("rocket_blog").collection::<Document>("admins");

                let cookie = match req.cookies().get_private("admin") {
                    Some(cookie) => cookie,
                    None => return None
                };

                match collection.find_one(doc!{"admin_id": cookie.value()}, None).await {
                    Ok(doc) => match doc {
                        Some(admin) => {
                            return Some(AdminAccount{
                                admin_id: admin.get_str("admin_id").unwrap().to_owned(),
                                username: admin.get_str("username").unwrap().to_owned()
                            })
                        },
                        None => return None
                    },
                    Err(_) => return None
                };
            }
        ).await;
        match cached_result {
            Some(admin) => Outcome::Success(admin.to_owned()),
            None => Outcome::Forward(())
        }
    }
}