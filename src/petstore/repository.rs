use std::cell::{self, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use model::*;
use self::RepositoryErrorKind::*;

error_chain! {
    types {
        RepositoryError, RepositoryErrorKind, ResultExt, RepositoryResult;
    }

    errors {
        NotFound {}
    }

    foreign_links {
        Borrow(cell::BorrowError);
        BorrowMut(cell::BorrowMutError);
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryDatabase<T> {
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
    pub fn read<F, R>(&self, f: F) -> RepositoryResult<R>
    where
        F: FnOnce(&HashMap<u64, T>) -> RepositoryResult<R>,
    {
        let inner = self.inner.try_borrow()?;
        f(&*inner)
    }

    pub fn write<F, R>(&self, f: F) -> RepositoryResult<R>
    where
        F: FnOnce(&mut HashMap<u64, T>) -> RepositoryResult<R>,
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
    pub fn get(&self, id: u64) -> RepositoryResult<Option<Pet>> {
        self.pets.read(|pets| Ok(pets.get(&id).cloned()))
    }

    pub fn find<F>(&self, mut f: F) -> RepositoryResult<Vec<Pet>>
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

    pub fn add(&self, mut new_pet: Pet) -> RepositoryResult<Pet> {
        self.pets.write(|pets| {
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

    pub fn update(&self, pet: Pet) -> RepositoryResult<Pet> {
        self.pets.write(|pets| {
            let id = pet.id.unwrap();
            if !pets.contains_key(&id) {
                bail!(NotFound);
            }
            pets.insert(id, pet.clone());

            Ok(pet)
        })
    }

    pub fn delete(&self, id: u64) -> RepositoryResult<Option<Pet>> {
        self.pets.write(|pets| Ok(pets.remove(&id)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct TagRepository {
    tags: InMemoryDatabase<Tag>,
}

impl TagRepository {
    pub fn add(&self, mut tag: Tag) -> RepositoryResult<Tag> {
        self.tags.write(|tags| {
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
    pub fn add(&self, mut category: Category) -> RepositoryResult<Category> {
        self.categories.write(|categories| {
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
    pub fn get(&self, id: u64) -> RepositoryResult<Option<Order>> {
        self.orders.read(|orders| Ok(orders.get(&id).cloned()))
    }

    pub fn add(&self, mut order: Order) -> RepositoryResult<Order> {
        self.orders.write(|orders| {
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

    pub fn delete(&self, id: u64) -> RepositoryResult<Option<Order>> {
        self.orders.write(|orders| Ok(orders.remove(&id)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserRepository {
    users: InMemoryDatabase<User>,
}

impl UserRepository {
    pub fn add(&self, mut new_user: User) -> RepositoryResult<User> {
        self.users.write(|users| {
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

    pub fn find_one<F>(&self, mut f: F) -> RepositoryResult<Option<User>>
    where
        F: FnMut(&User) -> bool,
    {
        self.users
            .read(|users| Ok(users.values().find(|u| f(u)).cloned()))
    }

    pub fn delete(&self, name: String) -> RepositoryResult<Option<User>> {
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

    pub fn update(&self, mut updated_user: User) -> RepositoryResult<User> {
        self.users.write(|users| {
            if let Some(user) = users
                .values_mut()
                .find(|user| user.username == updated_user.username)
            {
                updated_user.id = user.id;
                *user = updated_user.clone();
                Ok(updated_user)
            } else {
                bail!(NotFound)
            }
        })
    }
}
