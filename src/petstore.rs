use std::cell::{self, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use model::*;
use self::PetstoreErrorKind::*;

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

        MissingPet(msg: String) {
            display("missing pet: {}", msg)
        }

        MissingUser(msg: String) {
            display("missing user: {}", msg)
        }

        RedundantUserName(msg: String) {
            display("redundant username: {}", msg)
        }
    }

    foreign_links {
        Borrow(cell::BorrowError);
        BorrowMutError(cell::BorrowMutError);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Petstore {
    pets: Rc<RefCell<HashMap<u64, Pet>>>,
    tags: Rc<RefCell<HashMap<u64, Tag>>>,
    categories: Rc<RefCell<HashMap<u64, Category>>>,
    orders: Rc<RefCell<HashMap<u64, Order>>>,
    photos: Rc<RefCell<HashMap<u64, Vec<u8>>>>,
    users: Rc<RefCell<HashMap<u64, User>>>,
}

impl Petstore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_pet(&self, id: u64) -> PetstoreResult<Option<Pet>> {
        self.pets
            .try_borrow()
            .map_err(Into::into)
            .map(|pets| pets.get(&id).cloned())
    }

    pub fn add_pet(&self, mut pet: Pet) -> PetstoreResult<u64> {
        if pet.id.is_some() {
            bail!(InvalidInput("New pet should not contain an ID".to_string()));
        }

        let mut pets = self.pets.try_borrow_mut()?;

        let new_id = if pets.is_empty() {
            0
        } else {
            pets.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        pet.id = Some(new_id);
        pets.insert(new_id, pet.clone());

        if let Some(tags) = pet.tags {
            for tag in tags {
                self.add_tag(tag)?;
            }
        }

        if let Some(category) = pet.category {
            self.add_category(category)?;
        }

        Ok(new_id)
    }

    fn add_tag(&self, mut tag: Tag) -> PetstoreResult<Tag> {
        if tag.id.is_some() {
            bail!(InvalidInput("New tag should not contain an ID".to_string()));
        }

        let mut tags = self.tags.try_borrow_mut()?;
        let new_id = if tags.is_empty() {
            0
        } else {
            tags.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        tag.id = Some(new_id);
        tags.insert(new_id, tag.clone());

        Ok(tag)
    }

    fn add_category(&self, mut category: Category) -> PetstoreResult<Category> {
        if category.id.is_some() {
            bail!(InvalidInput(
                "New category should not contain an ID".to_string(),
            ));
        }

        let mut categories = self.categories.try_borrow_mut()?;
        let new_id = if categories.is_empty() {
            0
        } else {
            categories.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        category.id = Some(new_id);
        categories.insert(new_id, category.clone());

        Ok(category)
    }

    pub fn update_pet(&self, pet: Pet) -> PetstoreResult<Pet> {
        let id = pet.id
            .ok_or_else(|| MissingIdentifier(format!("Missing id for pet: {:?}", pet)))?;

        let mut pets = self.pets.try_borrow_mut()?;
        if !pets.contains_key(&id) {
            bail!(MissingPet("Invalid id: doesn't exist".to_string()));
        }
        pets.insert(id, pet.clone());

        Ok(pet)
    }

    pub fn get_pets_by_status(&self, statuses: Vec<Status>) -> PetstoreResult<Vec<Pet>> {
        self.find_pets(|p| p.status.map_or(true, |s| statuses.contains(&s)))
    }

    pub fn find_pets_by_tag(&self, tags: Vec<String>) -> PetstoreResult<Vec<Pet>> {
        self.find_pets(|p| {
            tags.iter().all(|ftag| {
                p.tags
                    .as_ref()
                    .map_or(false, |tags| tags.iter().any(|tag| tag.name == *ftag))
            })
        })
    }

    fn find_pets<F>(&self, mut f: F) -> PetstoreResult<Vec<Pet>>
    where
        F: FnMut(&Pet) -> bool,
    {
        let pets = self.pets.try_borrow()?;

        let mut pets: Vec<_> = pets.values().filter(|&p| f(p)).cloned().collect();
        pets.sort_by(|l, r| match (l.id, r.id) {
            (Some(l), Some(r)) => l.partial_cmp(&r).unwrap(),
            _ => panic!(),
        });

        Ok(pets)
    }

    pub fn delete_pet(&self, id: u64) -> PetstoreResult<()> {
        let mut pets = self.pets.try_borrow_mut()?;
        if !pets.contains_key(&id) {
            bail!(MissingPet(format!(
                "Pet with id {} does not exist and cannot be deleted",
                id
            )));
        }
        pets.remove(&id);
        Ok(())
    }

    pub fn update_pet_name_status(
        &self,
        pet_id: u64,
        name: Option<String>,
        status: Option<Status>,
    ) -> PetstoreResult<Pet> {
        let mut pets = self.pets.try_borrow_mut()?;
        if !pets.contains_key(&pet_id) {
            bail!(MissingPet(format!("Invalid id: doesn't exist")));
        }
        let pet = pets.get_mut(&pet_id).unwrap();
        if let Some(s) = status {
            pet.status = Some(s);
        }
        if let Some(n) = name {
            pet.name = n;
        }
        Ok(pet.clone())
    }

    // TODO: add_image
}

impl Petstore {
    pub fn get_inventory(&self) -> PetstoreResult<Inventory> {
        let pets = self.pets.try_borrow()?;
        let mut inventory = Inventory {
            available: 0,
            pending: 0,
            adopted: 0,
        };
        for (_, pet) in &*pets {
            match pet.status {
                Some(Available) => inventory.available += 1,
                Some(Pending) => inventory.pending += 1,
                Some(Adopted) => inventory.adopted += 1,
                None => {}
            }
        }
        Ok(inventory)
    }

    pub fn add_order(&self, mut order: Order) -> PetstoreResult<u64> {
        if order.status.is_some() {
            bail!(InvalidInput("New order should not contain an ID".into()));
        }
        let mut orders = self.orders.try_borrow_mut()?;
        let new_id = if orders.is_empty() {
            0
        } else {
            orders.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        order.id = Some(new_id);
        orders.insert(new_id, order.clone());

        Ok(new_id)
    }

    pub fn delete_order(&self, id: u64) -> PetstoreResult<bool> {
        let mut orders = self.orders.try_borrow_mut()?;
        if orders.contains_key(&id) {
            orders.remove(&id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn find_order(&self, id: u64) -> PetstoreResult<Option<Order>> {
        self.orders
            .try_borrow()
            .map_err(Into::into)
            .map(|orders| orders.get(&id).cloned())
    }
}

// user APIs
impl Petstore {
    pub fn add_user(&self, mut new_user: User) -> PetstoreResult<String> {
        if new_user.id.is_some() {
            bail!(InvalidInput("New user should not contain an ID".into()));
        }
        let new_username = new_user.username.clone();

        let mut users = self.users.try_borrow_mut()?;
        if users.values().any(|user| user.username == new_username) {
            bail!(RedundantUserName(format!(
                "Username {} is already taken",
                new_user.username
            )));
        }
        let new_id = if users.is_empty() {
            0
        } else {
            users.keys().map(|id| *id).max().unwrap_or(0) + 1
        };
        new_user.id = Some(new_id);
        users.insert(new_id, new_user);

        Ok(new_username)
    }

    pub fn add_users(&self, users: Vec<User>) -> PetstoreResult<Vec<String>> {
        users
            .into_iter()
            .map(move |new_user| self.add_user(new_user))
            .collect()
    }

    pub fn get_user(&self, name: String) -> PetstoreResult<Option<User>> {
        let users = self.users.try_borrow()?;
        Ok(users.values().find(|user| user.username == name).cloned())
    }

    pub fn delete_user(&self, name: String) -> PetstoreResult<()> {
        let mut users = self.users.try_borrow_mut()?;
        if let Some(id) = users
            .values()
            .find(|user| user.username == name)
            .and_then(|user| user.id)
        {
            users.remove(&id);
        }
        Ok(())
    }

    pub fn update_user(&self, mut updated_user: User) -> PetstoreResult<User> {
        let mut users = self.users.try_borrow_mut()?;
        if let Some(user) = users
            .values_mut()
            .find(|user| user.username == updated_user.username)
        {
            updated_user.id = user.id;
            *user = updated_user.clone();
            Ok(updated_user)
        } else {
            bail!(MissingUser("This user doesn't exist".into()))
        }
    }
}
