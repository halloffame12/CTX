use crate::models::user::User;

fn main() {
    let u = User::new(1, "ada".to_string());
    println!("{}", u.display_name());
}