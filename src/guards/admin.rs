use mongodb::bson::{doc, Document};
use rocket::{request::{ 
    Outcome, 
    Request, 
    FromRequest
}, 
    http::Status
};
use rocket_db_pools::Connection;

use crate::MongoDB;

pub struct AdminAccount{
    pub admin_id: String,
    pub username: String
}

#[derive(Debug)]
pub enum AdminAccountError {
    DoesNotExist,
    AccessDenied
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAccount {
    type Error = AdminAccountError;
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req.guard::<Connection<MongoDB>>().await.unwrap();
        let jar = req.cookies();

        let collection = db.database("rocket_blog").collection::<Document>("admins");
        
        // Check for admin's private cookie.
        let cookie = match jar.get_private("admin") {
            None => return Outcome::Failure((Status::Forbidden, AdminAccountError::AccessDenied)),
            Some(cookie) => cookie
        };

        // Check if admin's cookie value matches real admin.
        let admin_account = match collection.find_one(doc!{"admin_id": cookie.value()}, None).await.unwrap() {
            None => return Outcome::Failure((Status::NotFound, AdminAccountError::DoesNotExist)),
            Some(account) => account
        };

        // If admin exists and cookie is valid, we can return an admin account.
        Outcome::Success(AdminAccount{
            admin_id: match admin_account.get_str("admin_id") {
                Ok(id) => id.to_owned(),
                Err(_) => return Outcome::Failure((Status::NotFound, AdminAccountError::DoesNotExist))
            },
            username: match admin_account.get_str("username") {
                Ok(username) => username.to_owned(),
                Err(_) => return Outcome::Failure((Status::NotFound, AdminAccountError::DoesNotExist))
            }
        })
    }
}