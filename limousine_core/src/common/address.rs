#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(usize);

pub struct Arena<T> {
    data: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }

    #[inline(always)]
    pub fn add(&mut self, entry: T) -> Address {
        self.data.push(entry);
        Address(self.data.len() - 1)
    }

    #[inline(always)]
    pub fn deref(&self, address: Address) -> &T {
        // self.data.get_unchecked(address.0)
        &self.data[address.0]
    }

    #[inline(always)]
    pub fn deref_mut(&mut self, address: Address) -> &mut T {
        // self.data.get_unchecked_mut(address.0)
        &mut self.data[address.0]
    }
}
