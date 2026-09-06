pub struct User {
    pub id: u64,
    pub name: String,
}

pub enum Role {
    Admin,
    Member,
    Guest,
}

pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for User {
    fn describe(&self) -> String {
        format!("{}:{}", self.id, self.name)
    }
}

impl User {
    pub fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }

    pub fn display_name(&self) -> String {
        self.name.clone()
    }
}

pub const MAX_USERS: usize = 100;

pub fn find_user(id: u64) -> Option<User> {
    None
}