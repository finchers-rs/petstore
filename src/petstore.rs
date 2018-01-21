use db::PetstoreDb;

#[derive(Debug, Clone)]
pub struct Petstore {
    db: PetstoreDb,
}

impl Petstore {
    pub fn new(db: PetstoreDb) -> Self {
        Petstore { db }
    }

    pub fn database(&self) -> &PetstoreDb {
        &self.db
    }
}
