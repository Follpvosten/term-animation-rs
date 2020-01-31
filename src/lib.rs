pub mod entity;
pub use entity::Entity;
pub mod animation;
pub use animation::Animation;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
