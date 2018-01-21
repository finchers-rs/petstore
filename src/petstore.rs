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

#[derive(Debug, Clone)]
struct InMemoryDatabase<T> {
    inner: Rc<RefCell<HashMap<u64, T>>>,
}

impl<T> Default for InMemoryDatabase<T> {
    fn default() -> Self {
        InMemoryDatabase {
            inner: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl<T> InMemoryDatabase<T> {
    pub fn read<F, R>(&self, f: F) -> PetstoreResult<R>
    where
        F: FnOnce(&HashMap<u64, T>) -> PetstoreResult<R>,
    {
        let inner = self.inner.try_borrow()?;
        f(&*inner)
    }

    pub fn write<F, R>(&self, f: F) -> PetstoreResult<R>
    where
        F: FnOnce(&mut HashMap<u64, T>) -> PetstoreResult<R>,
    {
        let mut inner = self.inner.try_borrow_mut()?;
        f(&mut *inner)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PetRepository {
    pets: InMemoryDatabase<Pet>,
}

impl PetRepository {
    pub fn get(&self, id: u64) -> PetstoreResult<Option<Pet>> {
        self.pets.read(|pets| Ok(pets.get(&id).cloned()))
    }

    pub fn find<F>(&self, mut f: F) -> PetstoreResult<Vec<Pet>>
    where
        F: FnMut(&Pet) -> bool,
    {
        self.pets.read(|pets| {
            let mut pets: Vec<_> = pets.values().filter(|&p| f(p)).cloned().collect();
            pets.sort_by(|l, r| match (l.id, r.id) {
                (Some(l), Some(r)) => l.partial_cmp(&r).unwrap(),
                _ => panic!(),
            });

            Ok(pets)
        })
    }

    pub fn add(&self, mut new_pet: Pet) -> PetstoreResult<Pet> {
        self.pets.write(|pets| {
            if new_pet.id.is_some() {
                bail!(InvalidInput("New pet should not contain an ID".to_string()));
            }
            let new_id = if pets.is_empty() {
                0
            } else {
                pets.keys().map(|id| *id).max().unwrap_or(0) + 1
            };
            new_pet.id = Some(new_id);
            pets.insert(new_id, new_pet.clone());

            Ok(new_pet)
        })
    }

    pub fn update(&self, pet: Pet) -> PetstoreResult<Pet> {
        self.pets.write(|pets| {
            let id = pet.id
                .ok_or_else(|| MissingIdentifier(format!("Missing id for pet: {:?}", pet)))?;
            if !pets.contains_key(&id) {
                bail!(MissingPet("Invalid id: doesn't exist".to_string()));
            }
            pets.insert(id, pet.clone());

            Ok(pet)
        })
    }

    pub fn delete(&self, id: u64) -> PetstoreResult<Option<Pet>> {
        self.pets.write(|pets| Ok(pets.remove(&id)))
    }

    pub fn update_name_status(&self, id: u64, name: Option<String>, status: Option<Status>) -> PetstoreResult<Pet> {
        self.pets.write(|pets| {
            if !pets.contains_key(&id) {
                bail!(MissingPet(format!("Invalid id: doesn't exist")));
            }
            let pet = pets.get_mut(&id).unwrap();
            if let Some(s) = status {
                pet.status = Some(s);
            }
            if let Some(n) = name {
                pet.name = n;
            }
            Ok(pet.clone())
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct TagRepository {
    tags: InMemoryDatabase<Tag>,
}

impl TagRepository {
    pub fn add(&self, mut tag: Tag) -> PetstoreResult<Tag> {
        self.tags.write(|tags| {
            if tag.id.is_some() {
                bail!(InvalidInput("New tag should not contain an ID".to_string()));
            }
            let new_id = if tags.is_empty() {
                0
            } else {
                tags.keys().map(|id| *id).max().unwrap_or(0) + 1
            };
            tag.id = Some(new_id);
            tags.insert(new_id, tag.clone());

            Ok(tag)
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct CategoryRepository {
    categories: InMemoryDatabase<Category>,
}

impl CategoryRepository {
    pub fn add(&self, mut category: Category) -> PetstoreResult<Category> {
        self.categories.write(|categories| {
            if category.id.is_some() {
                bail!(InvalidInput(
                    "New category should not contain an ID".to_string(),
                ));
            }
            let new_id = if categories.is_empty() {
                0
            } else {
                categories.keys().map(|id| *id).max().unwrap_or(0) + 1
            };
            category.id = Some(new_id);
            categories.insert(new_id, category.clone());

            Ok(category)
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrderRepository {
    orders: InMemoryDatabase<Order>,
}

impl OrderRepository {
    pub fn get(&self, id: u64) -> PetstoreResult<Option<Order>> {
        self.orders.read(|orders| Ok(orders.get(&id).cloned()))
    }

    pub fn add(&self, mut order: Order) -> PetstoreResult<Order> {
        self.orders.write(|orders| {
            if order.status.is_some() {
                bail!(InvalidInput("New order should not contain an ID".into()));
            }
            let new_id = if orders.is_empty() {
                0
            } else {
                orders.keys().map(|id| *id).max().unwrap_or(0) + 1
            };
            order.id = Some(new_id);
            orders.insert(new_id, order.clone());
            Ok(order)
        })
    }

    pub fn delete(&self, id: u64) -> PetstoreResult<Option<Order>> {
        self.orders.write(|orders| Ok(orders.remove(&id)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserRepository {
    users: InMemoryDatabase<User>,
}

impl UserRepository {
    pub fn add(&self, mut new_user: User) -> PetstoreResult<User> {
        self.users.write(|users| {
            if new_user.id.is_some() {
                bail!(InvalidInput("New user should not contain an ID".into()));
            }
            let new_username = new_user.username.clone();
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
            users.insert(new_id, new_user.clone());

            Ok(new_user)
        })
    }

    pub fn find_one<F>(&self, mut f: F) -> PetstoreResult<Option<User>>
    where
        F: FnMut(&User) -> bool,
    {
        self.users
            .read(|users| Ok(users.values().find(|u| f(u)).cloned()))
    }


    pub fn delete(&self, name: String) -> PetstoreResult<Option<User>> {
        self.users.write(|users| {
            if let Some(id) = users
                .values()
                .find(|user| user.username == name)
                .and_then(|user| user.id)
            {
                Ok(users.remove(&id))
            } else {
                Ok(None)
            }
        })
    }

    pub fn update(&self, mut updated_user: User) -> PetstoreResult<User> {
        self.users.write(|users| {
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
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Petstore {
    pets: PetRepository,
    tags: TagRepository,
    categories: CategoryRepository,
    orders: OrderRepository,
    users: UserRepository,
}

impl Petstore {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn get_pet(&self, id: u64) -> PetstoreResult<Option<Pet>> {
        self.pets.get(id)
    }

    pub fn add_pet(&self, pet: Pet) -> PetstoreResult<u64> {
        let pet = self.pets.add(pet)?;
        if let Some(tags) = pet.tags {
            for tag in tags {
                self.tags.add(tag)?;
            }
        }
        if let Some(category) = pet.category {
            self.categories.add(category)?;
        }
        Ok(pet.id.unwrap())
    }

    #[inline]
    pub fn update_pet(&self, pet: Pet) -> PetstoreResult<Pet> {
        self.pets.update(pet)
    }

    #[inline]
    pub fn get_pets_by_status(&self, statuses: Vec<Status>) -> PetstoreResult<Vec<Pet>> {
        self.pets
            .find(|p| p.status.map_or(true, |s| statuses.contains(&s)))
    }

    pub fn find_pets_by_tag(&self, tags: Vec<String>) -> PetstoreResult<Vec<Pet>> {
        self.pets.find(|p| {
            tags.iter().all(|ftag| {
                p.tags
                    .as_ref()
                    .map_or(false, |tags| tags.iter().any(|tag| tag.name == *ftag))
            })
        })
    }

    pub fn delete_pet(&self, id: u64) -> PetstoreResult<()> {
        self.pets.delete(id).and_then(|pet| {
            if pet.is_none() {
                bail!(MissingPet(format!(
                    "Pet with id {} does not exist and cannot be deleted",
                    id
                )));
            }
            Ok(())
        })
    }

    pub fn update_pet_name_status(
        &self,
        pet_id: u64,
        name: Option<String>,
        status: Option<Status>,
    ) -> PetstoreResult<Pet> {
        self.pets.update_name_status(pet_id, name, status)
    }

    // TODO: add_image
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

    #[inline]
    pub fn add_order(&self, order: Order) -> PetstoreResult<u64> {
        self.orders.add(order).map(|o| o.id.unwrap())
    }

    #[inline]
    pub fn delete_order(&self, id: u64) -> PetstoreResult<bool> {
        self.orders.delete(id).map(|o| o.is_some())
    }

    #[inline]
    pub fn find_order(&self, id: u64) -> PetstoreResult<Option<Order>> {
        self.orders.get(id)
    }
}

// user APIs
impl Petstore {
    pub fn add_user(&self, new_user: User) -> PetstoreResult<String> {
        self.users.add(new_user).map(|user| user.username)
    }

    pub fn add_users(&self, users: Vec<User>) -> PetstoreResult<Vec<String>> {
        users
            .into_iter()
            .map(move |new_user| self.add_user(new_user))
            .collect()
    }

    pub fn get_user(&self, name: String) -> PetstoreResult<Option<User>> {
        self.users.find_one(|user| user.username == name)
    }

    pub fn delete_user(&self, name: String) -> PetstoreResult<()> {
        self.users.delete(name).map(|_| ())
    }

    pub fn update_user(&self, updated_user: User) -> PetstoreResult<User> {
        self.users.update(updated_user)
    }
}
