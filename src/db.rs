#![allow(dead_code)]

use model::*;

use std::cell::{self, RefCell};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum DbError {
    BorrowError(cell::BorrowError),
    BorrowMutError(cell::BorrowMutError),
    InvalidInput(String),
    MissingIdentifier(String),
    MissingPet(String),
    MissingUser(String),
    RedundantUserName(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BorrowError(ref e) => e.fmt(f),
            BorrowMutError(ref e) => e.fmt(f),
            InvalidInput(ref msg) => write!(f, "invalid input: {}", msg),
            MissingIdentifier(ref msg) => write!(f, "missing identifier: {}", msg),
            MissingPet(ref msg) => write!(f, "missing pet: {}", msg),
            MissingUser(ref msg) => write!(f, "missing user: {}", msg),
            RedundantUserName(ref msg) => write!(f, "redundant username: {}", msg),
        }
    }
}

impl error::Error for DbError {
    fn description(&self) -> &str {
        "database error"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            BorrowError(ref e) => Some(e),
            BorrowMutError(ref e) => Some(e),
            _ => None,
        }
    }
}

pub use self::DbError::*;

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Default)]
pub struct PetstoreDbContext {
    pets: HashMap<u64, Pet>,
    tags: HashMap<u64, Tag>,
    categories: HashMap<u64, Category>,
    orders: HashMap<u64, Order>,
    photos: HashMap<u64, Vec<u8>>,
    users: HashMap<u64, User>,
}

impl PetstoreDbContext {
    fn add_tag(&mut self, mut tag: Tag) -> DbResult<Tag> {
        if tag.id.is_some() {
            return Err(InvalidInput("New tag should not contain an ID".to_string()));
        }
        let new_id = if self.tags.is_empty() {
            0
        } else {
            self.tags.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        tag.id = Some(new_id);
        self.tags.insert(new_id, tag.clone());
        Ok(tag)
    }

    fn add_category(&mut self, mut category: Category) -> DbResult<Category> {
        if category.id.is_some() {
            return Err(InvalidInput(
                "New category should not contain an ID".to_string(),
            ));
        }
        let new_id = if self.categories.is_empty() {
            0
        } else {
            self.categories.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        category.id = Some(new_id);
        self.categories.insert(new_id, category.clone());
        Ok(category)
    }

    fn find_pets<F>(&self, mut f: F) -> Vec<Pet>
    where
        F: FnMut(&Pet) -> bool,
    {
        let mut pets: Vec<_> = self.pets.values().filter(|&p| f(p)).cloned().collect();
        pets.sort_by(|l, r| match (l.id, r.id) {
            (Some(l), Some(r)) => l.partial_cmp(&r).unwrap(),
            _ => panic!(),
        });
        pets
    }

    fn add_user(&mut self, mut new_user: User) -> DbResult<String> {
        if new_user.id.is_some() {
            return Err(InvalidInput("New user should not contain an ID".into()));
        }
        let new_username = new_user.username.clone();
        if self.users
            .values()
            .any(|user| user.username == new_username)
        {
            return Err(RedundantUserName(format!(
                "Username {} is already taken",
                new_user.username
            )));
        }
        let new_id = if self.users.is_empty() {
            0
        } else {
            self.users.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        new_user.id = Some(new_id);
        self.users.insert(new_id, new_user);

        Ok(new_username)
    }
}

#[derive(Debug, Clone)]
pub struct PetstoreDb {
    context: Rc<RefCell<PetstoreDbContext>>,
}

impl PetstoreDb {
    pub fn new() -> Self {
        Self {
            context: Rc::new(RefCell::new(Default::default())),
        }
    }

    fn read_async<F, T: 'static>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&PetstoreDbContext) -> Result<T, DbError>,
    {
        self.context
            .try_borrow()
            .map_err(BorrowError)
            .and_then(|context| f(&*context))
    }

    fn write_async<F, T: 'static>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut PetstoreDbContext) -> Result<T, DbError>,
    {
        self.context
            .try_borrow_mut()
            .map_err(BorrowMutError)
            .and_then(|mut context| f(&mut *context))
    }

    pub fn get_pet(&self, id: u64) -> DbResult<Option<Pet>> {
        self.read_async(move |context| Ok(context.pets.get(&id).cloned()))
    }

    pub fn add_pet(&self, mut pet: Pet) -> DbResult<u64> {
        if pet.id.is_some() {
            return Err(InvalidInput("New pet should not contain an ID".to_string()));
        }
        self.write_async(move |context| -> DbResult<_> {
            let new_id = if context.pets.is_empty() {
                0
            } else {
                context.pets.keys().map(|id| *id).max().unwrap_or(0) + 1
            };
            pet.id = Some(new_id);
            context.pets.insert(new_id, pet.clone());

            if let Some(tags) = pet.tags {
                for tag in tags {
                    context.add_tag(tag)?;
                }
            }

            if let Some(category) = pet.category {
                context.add_category(category)?;
            }

            Ok(new_id)
        })
    }

    pub fn update_pet(&self, pet: Pet) -> DbResult<Pet> {
        if pet.id.is_none() {
            return Err(MissingIdentifier(format!("Missing id for pet: {:?}", pet)));
        }
        self.write_async(move |context| {
            let id = pet.id.unwrap();
            if context.pets.contains_key(&id) {
                context.pets.insert(id, pet.clone());
                Ok(pet)
            } else {
                Err(MissingPet("Invalid id: doesn't exist".to_string()))
            }
        })
    }

    pub fn get_pets_by_status(&self, statuses: Vec<Status>) -> DbResult<Vec<Pet>> {
        self.read_async(move |context| Ok(context.find_pets(|p| p.status.map_or(true, |s| statuses.contains(&s)))))
    }

    pub fn find_pets_by_tag(&self, tags: Vec<String>) -> DbResult<Vec<Pet>> {
        self.read_async(move |context| {
            Ok(context.find_pets(|p| {
                tags.iter().all(|ftag| {
                    p.tags
                        .as_ref()
                        .map_or(false, |tags| tags.iter().any(|tag| tag.name == *ftag))
                })
            }))
        })
    }

    pub fn delete_pet(&self, id: u64) -> DbResult<()> {
        self.write_async(move |context| {
            if context.pets.contains_key(&id) {
                context.pets.remove(&id);
                Ok(())
            } else {
                Err(MissingPet(format!(
                    "Pet with id {} does not exist and cannot be deleted",
                    id
                )))
            }
        })
    }

    pub fn update_pet_name_status(&self, pet_id: u64, name: Option<String>, status: Option<Status>) -> DbResult<Pet> {
        self.write_async(move |context| {
            if context.pets.contains_key(&pet_id) {
                let pet = context.pets.get_mut(&pet_id).unwrap();
                if let Some(s) = status {
                    pet.status = Some(s);
                }
                if let Some(n) = name {
                    pet.name = n;
                }
                Ok(pet.clone())
            } else {
                Err(MissingPet(format!("Invalid id: doesn't exist")))
            }
        })
    }

    // TODO: add_image

    pub fn get_inventory(&self) -> DbResult<Inventory> {
        self.read_async(move |context| {
            let mut inventory = Inventory {
                available: 0,
                pending: 0,
                adopted: 0,
            };
            for (_, pet) in &context.pets {
                match pet.status {
                    Some(Available) => inventory.available += 1,
                    Some(Pending) => inventory.pending += 1,
                    Some(Adopted) => inventory.adopted += 1,
                    None => {}
                }
            }
            Ok(inventory)
        })
    }

    pub fn add_order(&self, mut order: Order) -> DbResult<u64> {
        if order.status.is_some() {
            return Err(InvalidInput("New order should not contain an ID".into()));
        }
        self.write_async(move |context| {
            let new_id = if context.orders.is_empty() {
                0
            } else {
                context.orders.keys().map(|id| *id).max().unwrap_or(0) + 1
            };
            order.id = Some(new_id);
            context.orders.insert(new_id, order.clone());
            Ok(new_id)
        })
    }

    pub fn delete_order(&self, id: u64) -> DbResult<bool> {
        self.write_async(move |context| {
            if context.orders.contains_key(&id) {
                context.orders.remove(&id);
                Ok(true)
            } else {
                Ok(false)
            }
        })
    }

    pub fn find_order(&self, id: u64) -> DbResult<Option<Order>> {
        self.read_async(move |context| Ok(context.orders.get(&id).cloned()))
    }

    pub fn add_user(&self, new_user: User) -> DbResult<String> {
        self.write_async(move |context| context.add_user(new_user))
    }

    pub fn add_users(&self, users: Vec<User>) -> DbResult<Vec<String>> {
        self.write_async(move |context| {
            users
                .into_iter()
                .map(move |new_user| context.add_user(new_user))
                .collect()
        })
    }

    pub fn get_user(&self, name: String) -> DbResult<Option<User>> {
        self.read_async(move |context| {
            Ok(context
                .users
                .values()
                .find(|user| user.username == name)
                .cloned())
        })
    }

    pub fn delete_user(&self, name: String) -> DbResult<()> {
        self.write_async(move |context| {
            if let Some(id) = context
                .users
                .values()
                .find(|user| user.username == name)
                .and_then(|user| user.id)
            {
                context.users.remove(&id);
            }
            Ok(())
        })
    }

    pub fn update_user(&self, mut updated_user: User) -> DbResult<User> {
        self.write_async(move |context| {
            if let Some(user) = context
                .users
                .values_mut()
                .find(|user| user.username == updated_user.username)
            {
                updated_user.id = user.id;
                *user = updated_user.clone();
                Ok(updated_user)
            } else {
                Err(MissingUser("This user doesn't exist".into()))
            }
        })
    }
}
