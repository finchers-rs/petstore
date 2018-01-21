pub mod repository;

use model::*;
use self::PetstoreErrorKind::*;
use self::repository::{RepositoryError, RepositoryErrorKind};

error_chain! {
    types {
        PetstoreError, PetstoreErrorKind, ResultExt, PetstoreResult;
    }

    errors {
        InvalidInput(msg: String) {
            display("invalid input: {}", msg)
        }

        MissingIdentifier(msg: String) {
            display("missing identifier: {}", msg)
        }
    }

    links {
        Repository(RepositoryError, RepositoryErrorKind);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Petstore {
    pets: self::repository::PetRepository,
    tags: self::repository::TagRepository,
    categories: self::repository::CategoryRepository,
    orders: self::repository::OrderRepository,
    users: self::repository::UserRepository,
}

impl Petstore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_pet(&self, id: u64) -> PetstoreResult<Option<Pet>> {
        self.pets.get(id).map_err(Into::into)
    }

    pub fn add_pet(&self, pet: Pet) -> PetstoreResult<u64> {
        if pet.id.is_some() {
            bail!(InvalidInput("New pet should not contain an ID".to_string()));
        }
        let pet = self.pets.add(pet)?;
        if let Some(tags) = pet.tags {
            for tag in tags {
                if tag.id.is_some() {
                    bail!(InvalidInput("New tag should not contain an ID".to_string()));
                }
                self.tags.add(tag)?;
            }
        }
        if let Some(category) = pet.category {
            if category.id.is_some() {
                bail!(InvalidInput(
                    "New category should not contain an ID".to_string(),
                ));
            }
            self.categories.add(category)?;
        }
        Ok(pet.id.unwrap())
    }

    pub fn update_pet(&self, pet: Pet) -> PetstoreResult<Pet> {
        if pet.id.is_none() {
            bail!(MissingIdentifier(format!("Missing id for pet: {:?}", pet)));
        }
        self.pets.update(pet).map_err(Into::into)
    }

    pub fn get_pets_by_status(&self, statuses: Vec<Status>) -> PetstoreResult<Vec<Pet>> {
        self.pets
            .find(|p| p.status.map_or(true, |s| statuses.contains(&s)))
            .map_err(Into::into)
    }

    pub fn find_pets_by_tag(&self, tags: Vec<String>) -> PetstoreResult<Vec<Pet>> {
        self.pets
            .find(|p| {
                tags.iter().all(|ftag| {
                    p.tags
                        .as_ref()
                        .map_or(false, |tags| tags.iter().any(|tag| tag.name == *ftag))
                })
            })
            .map_err(Into::into)
    }

    pub fn delete_pet(&self, id: u64) -> PetstoreResult<()> {
        self.pets.delete(id).map(|_| ()).map_err(Into::into)
    }
}

impl Petstore {
    pub fn get_inventory(&self) -> PetstoreResult<Inventory> {
        let pets = self.pets.find(|_| true)?;
        let mut inventory = Inventory {
            available: 0,
            pending: 0,
            adopted: 0,
        };
        for pet in pets {
            match pet.status {
                Some(Available) => inventory.available += 1,
                Some(Pending) => inventory.pending += 1,
                Some(Adopted) => inventory.adopted += 1,
                None => {}
            }
        }
        Ok(inventory)
    }

    pub fn add_order(&self, order: Order) -> PetstoreResult<u64> {
        if order.status.is_some() {
            bail!(InvalidInput("New order should not contain an ID".into()));
        }
        self.orders
            .add(order)
            .map(|o| o.id.unwrap())
            .map_err(Into::into)
    }

    pub fn delete_order(&self, id: u64) -> PetstoreResult<bool> {
        self.orders
            .delete(id)
            .map(|o| o.is_some())
            .map_err(Into::into)
    }

    pub fn find_order(&self, id: u64) -> PetstoreResult<Option<Order>> {
        self.orders.get(id).map_err(Into::into)
    }
}

// user APIs
impl Petstore {
    pub fn add_user(&self, new_user: User) -> PetstoreResult<String> {
        if new_user.id.is_some() {
            bail!(InvalidInput("New user should not contain an ID".into()));
        }
        self.users
            .add(new_user)
            .map(|user| user.username)
            .map_err(Into::into)
    }

    pub fn add_users(&self, users: Vec<User>) -> PetstoreResult<Vec<String>> {
        users
            .into_iter()
            .map(move |new_user| self.add_user(new_user))
            .collect()
    }

    pub fn get_user(&self, name: String) -> PetstoreResult<Option<User>> {
        self.users
            .find_one(|user| user.username == name)
            .map_err(Into::into)
    }

    pub fn delete_user(&self, name: String) -> PetstoreResult<()> {
        self.users.delete(name).map(|_| ()).map_err(Into::into)
    }

    pub fn update_user(&self, updated_user: User) -> PetstoreResult<User> {
        self.users.update(updated_user).map_err(Into::into)
    }
}
