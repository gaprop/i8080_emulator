pub trait Device<T> {
    // Better names...
    fn fetch(&mut self) -> u8;
    fn exec(&mut self, op: u8) -> T;
}
