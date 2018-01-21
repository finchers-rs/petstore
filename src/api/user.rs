use finchers::{Endpoint, Handler};

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

#[derive(Debug)]
pub enum Response {
    UserCreated(String),
    UsersCreated(Vec<String>),
    TheUser(User),
    UserDeleted,
}

mod imp {
    use api::common::*;
    impl IntoResponse for super::Response {
        fn into_response(self) -> HyperResponse {
            use super::Response::*;
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
    type Item = Response;
    type Error = super::Error;
    type Result = Result<Option<Self::Item>, Self::Error>;

    fn call(&self, request: Request) -> Self::Result {
        use self::Request::*;
        use self::Response::*;
        match request {
            AddUser(new_user) => self.db
                .add_user(new_user)
                .map_err(Error::database)
                .map(|u| Some(UserCreated(u))),
            AddUsersViaList(users) => self.db
                .add_users(users)
                .map_err(Error::database)
                .map(|u| Some(UsersCreated(u))),
            DeleteUser(name) => self.db
                .delete_user(name)
                .map_err(Error::database)
                .map(|_| Some(UserDeleted)),
            GetUser(name) => self.db
                .get_user(name)
                .map_err(Error::database)
                .map(|u| u.map(TheUser)),
            UpdateUser(user) => self.db
                .update_user(user)
                .map_err(Error::database)
                .map(|user| Some(TheUser(user))),
        }
    }
}
