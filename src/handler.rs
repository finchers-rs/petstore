use finchers::Handler;
use finchers::http::StatusCode;
use futures::Future;

use model::*;
use db::{DbError, PetstoreDb};
use endpoint::Request;
use endpoint::Request::*;

#[derive(Debug)]
pub struct PetstoreResponse {
    pub status: StatusCode,
    pub content: Option<PetstoreResponseContent>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum PetstoreResponseContent {
    ThePet(Option<Pet>),
    PetId(u64),
    Pets(Vec<Pet>),

    TheInventory(Inventory),
    OrderId(u64),
    TheOrder(Option<Order>),
    OrderDeleted(bool),

    Username(String),
    Usernames(Vec<String>),
    TheUser(Option<User>),
}

pub use self::PetstoreResponseContent::*;

impl From<PetstoreResponseContent> for PetstoreResponse {
    fn from(content: PetstoreResponseContent) -> Self {
        PetstoreResponse {
            status: StatusCode::Ok,
            content: Some(content),
        }
    }
}

impl PetstoreResponse {
    pub fn created(content: PetstoreResponseContent) -> Self {
        PetstoreResponse {
            status: StatusCode::Created,
            content: Some(content),
        }
    }

    pub fn no_content() -> Self {
        PetstoreResponse {
            status: StatusCode::NoContent,
            content: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PetstoreHandler {
    db: PetstoreDb,
}

impl PetstoreHandler {
    pub fn new(db: PetstoreDb) -> Self {
        PetstoreHandler { db }
    }
}

impl Handler<Request> for PetstoreHandler {
    type Item = PetstoreResponse;
    type Error = DbError;
    type Future = Box<Future<Item = Self::Item, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        let future = match request {
            GetPet(id) => Box::new(self.db.get_pet(id).map(|pet| ThePet(pet).into())) as Self::Future,
            AddPet(pet) => Box::new(
                self.db
                    .add_pet(pet)
                    .map(|id| PetstoreResponse::created(PetId(id))),
            ) as Self::Future,
            UpdatePet(pet) => Box::new(self.db.update_pet(pet).map(|pet| ThePet(Some(pet)).into())) as Self::Future,
            DeletePet(id) => Box::new(
                self.db
                    .delete_pet(id)
                    .map(|_| PetstoreResponse::no_content()),
            ) as Self::Future,
            FindPetsByStatuses(param) => Box::new(
                self.db
                    .get_pets_by_status(param.status)
                    .map(|pets| Pets(pets).into()),
            ) as Self::Future,
            FindPetsByTags(param) => Box::new(
                self.db
                    .find_pets_by_tag(param.tags)
                    .map(|pets| Pets(pets).into()),
            ) as Self::Future,
            UpdatePetViaForm(id, param) => Box::new(
                self.db
                    .update_pet_name_status(id, param.name, param.status)
                    .map(|pet| ThePet(Some(pet)).into()),
            ) as Self::Future,

            GetInventory => Box::new(
                self.db
                    .get_inventory()
                    .map(|inventory| TheInventory(inventory).into()),
            ) as Self::Future,
            AddOrder(order) => Box::new(self.db.add_order(order).map(|id| OrderId(id).into())) as Self::Future,
            DeleteOrder(id) => Box::new(
                self.db
                    .delete_order(id)
                    .map(|deleted| OrderDeleted(deleted).into()),
            ) as Self::Future,
            FindOrder(id) => Box::new(self.db.find_order(id).map(|order| TheOrder(order).into())) as Self::Future,

            AddUser(new_user) => Box::new(
                self.db
                    .add_user(new_user)
                    .map(|username| Username(username).into()),
            ) as Self::Future,
            AddUsersViaList(users) => Box::new(
                ::futures::future::join_all(users.into_iter().map({
                    let db = self.db.clone();
                    move |new_user| db.add_user(new_user)
                })).map(|usernames| PetstoreResponse::created(Usernames(usernames))),
            ) as Self::Future,
            DeleteUser(name) => Box::new(
                self.db
                    .delete_user(name)
                    .map(|_| PetstoreResponse::no_content()),
            ) as Self::Future,
            GetUser(name) => Box::new(self.db.get_user(name).map(|user| TheUser(user).into())) as Self::Future,
            UpdateUser(user) => Box::new(
                self.db
                    .update_user(user)
                    .map(|user| TheUser(Some(user)).into()),
            ) as Self::Future,
        };
        future
    }
}
