pub trait Keyboard: Send + Sync {
    fn press(&self, key: char);
}