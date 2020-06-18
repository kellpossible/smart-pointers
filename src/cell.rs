use std::cell::UnsafeCell;

#[derive(Debug)]
pub struct Cell<T> {
    value: UnsafeCell<T>
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value)
        }
    }
    pub fn set(&self, value: T) {
        // SAFETY: this is safe because there can be no references to the value, because get requires T to be Copy
        // SAFETY: this is not thread-safe, but that's okay because UnsafeCell is !Sync thus Cell is !Sync
        unsafe { *self.value.get() = value};
    }

    pub fn get(&self) -> T 
    where
        T: Copy
    {
        // SAFETY: this is safe because T implements copy, and will be copied. 
        // SAFETY: this is not thread-safe, but that's okay because UnsafeCell is !Sync thus Cell is !Sync
        unsafe { *self.value.get() }
    }
}

#[cfg(test)]
mod test {
    use super::Cell;

    #[test]
    fn test() {
        let x = Cell::new(5);
        assert_eq!(x.get(), 5);

        x.set(20);
        assert_eq!(x.get(), 20);
    }
}