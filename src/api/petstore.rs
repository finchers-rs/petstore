use futures::Future;
use db::{DbError, PetstoreDb};

#[derive(Debug, Clone)]
pub struct Petstore {
    db: PetstoreDb,
}

impl Petstore {
    pub fn new(db: PetstoreDb) -> Self {
        Petstore { db }
    }
}

impl Petstore {
    pub fn add_users(&self, users: Vec<::model::User>) -> impl Future<Item = Vec<String>, Error = DbError> {
        use futures::future::join_all;
        let db = self.db.clone();
        join_all(users.into_iter().map(move |new_user| db.add_user(new_user)))
    }
}
