use std::collections::HashMap;
use std::sync::Arc;

use crate::db::Session;
use crate::models::{Model, User};

pub struct Repository<T>
where
    T: Model,
{
    session: Arc<Session>,
    cache: HashMap<String, T>,
}

impl<T: Model> Repository<T> {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session,
            cache: HashMap::new(),
        }
    }

    pub fn find(&self, pk: &str) -> Option<T> {
        self.cache.get(pk).cloned()
    }

    pub fn save(&mut self, obj: T) {
        let key = obj.id().to_string();
        self.cache.insert(key, obj);
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Active,
    Inactive,
}

pub trait RepositoryTrait {
    fn find(&self, pk: &str) -> Option<T>;
    fn save(&mut self, obj: T) {
        let _ = obj;
    }
}

pub fn lookup(id: &str) -> Result<Status, Box<dyn std::error::Error>> {
    Ok(Status::Active)
}
