use finchers::{Endpoint, Handler};

use error::Error;
use model::User;
use petstore::Petstore;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    AddUser(User),
    AddUsersViaList(Vec<User>),
    DeleteUser(String),
    GetUser(String),
    UpdateUser(User),
}

#[derive(Debug)]
pub enum Response {
    UserCreated(String),
    UsersCreated(Vec<String>),
    TheUser(User),
    UserDeleted,
}

use self::Request::*;
use self::Response::*;

mod imp {
    use super::*;
    use api::common::*;

    impl IntoResponse for Response {
        fn into_response(self) -> HyperResponse {
            match self {
                UserCreated(username) => json_response(&username).with_status(StatusCode::Created),
                UsersCreated(usernames) => json_response(&usernames).with_status(StatusCode::Created),
                TheUser(user) => json_response(&user),
                UserDeleted => no_content(),
            }
        }
    }
}

pub fn endpoint() -> impl Endpoint<Item = Request, Error = Error> + Clone + 'static {
    use finchers::endpoint::prelude::*;
    use finchers::contrib::json::json_body;

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
    type Item = Response;
    type Error = Error;
    type Result = Result<Option<Self::Item>, Self::Error>;

    fn call(&self, request: Request) -> Self::Result {
        match request {
            AddUser(new_user) => self.add_user(new_user)
                .map_err(Error::database)
                .map(|u| Some(UserCreated(u))),
            AddUsersViaList(users) => self.add_users(users)
                .map_err(Error::database)
                .map(|u| Some(UsersCreated(u))),
            DeleteUser(name) => self.delete_user(name)
                .map_err(Error::database)
                .map(|_| Some(UserDeleted)),
            GetUser(name) => self.get_user(name)
                .map_err(Error::database)
                .map(|u| u.map(TheUser)),
            UpdateUser(user) => self.update_user(user)
                .map_err(Error::database)
                .map(|user| Some(TheUser(user))),
        }
    }
}
