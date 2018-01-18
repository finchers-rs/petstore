use finchers::contrib::json::Json;
use finchers::endpoint::{body, path, Endpoint};
use finchers::endpoint::method::{delete, get, post, put};
use finchers::handler::Handler;
use futures::Future;

use model::User;
use super::{Error, Petstore};

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    // users APIs
    AddUser(User),
    AddUsersViaList(Vec<User>),
    DeleteUser(String),
    GetUser(String),
    UpdateUser(User),
}

pub fn endpoint() -> impl Endpoint<Item = Request, Error = Error> + Clone + 'static {
    use self::Request::*;

    endpoint![
        post("user")
            .with(body().map_err(Error::endpoint))
            .map(|Json(u)| AddUser(u)),
        post("user/createWithList")
            .with(body().map_err(Error::endpoint))
            .map(|Json(body)| AddUsersViaList(body)),
        post("user/createWithArray")
            .with(body().map_err(Error::endpoint))
            .map(|Json(body)| AddUsersViaList(body)),
        delete("user")
            .with(path().map_err(Error::endpoint))
            .map(|n| DeleteUser(n)),
        get("user")
            .with(path().map_err(Error::endpoint))
            .map(|n| GetUser(n)),
        put("user")
            .with(body().map_err(Error::endpoint))
            .map(|Json(u)| UpdateUser(u)),
    ]
}

impl Handler<Request> for Petstore {
    type Item = super::PetstoreResponse;
    type Error = super::Error;
    type Future = super::PetstoreHandlerFuture;

    fn call(&self, request: Request) -> Self::Future {
        use self::Request::*;
        use super::PetstoreResponse::*;
        match request {
            AddUser(new_user) => self.db.add_user(new_user).map(UserCreated).into(),
            AddUsersViaList(users) => self.add_users(users).map(UsersCreated).into(),
            DeleteUser(name) => self.db.delete_user(name).map(|_| UserDeleted).into(),
            GetUser(name) => self.db.get_user(name).map(TheUser).into(),
            UpdateUser(user) => self.db
                .update_user(user)
                .map(|user| TheUser(Some(user)))
                .into(),
        }
    }
}
