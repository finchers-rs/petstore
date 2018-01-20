use finchers::{Endpoint, Handler};
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
    use finchers::endpoint::prelude::*;
    use finchers::contrib::json::json_body;
    use self::Request::*;

    endpoint("user").with(choice![
        get(path()).map(GetUser),
        delete(path()).map(DeleteUser),
        post(json_body()).map(AddUser),
        put(json_body()).map(UpdateUser),
        post("createWithList")
            .with(json_body())
            .map(AddUsersViaList),
        post("createWithArray")
            .with(json_body())
            .map(AddUsersViaList),
    ])
}

impl Handler<Request> for Petstore {
    type Item = super::PetstoreResponse;
    type Error = super::Error;
    type Result = super::PetstoreHandlerFuture;

    fn call(&self, request: Request) -> Self::Result {
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
