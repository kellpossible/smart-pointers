use std::{ops::Deref, ptr::NonNull};
use crate::cell::Cell;

struct RcInner<T> {
    value: T,
    references: Cell<usize>,
}

pub struct Rc<T> {
    inner: NonNull<RcInner<T>>,
    /// TODO: PhantomData needed here because...
    /// See https://youtu.be/8O0Nt9qY_vo?t=5870 and 
    /// https://doc.rust-lang.org/nomicon/dropck.html for more info.
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        let inner = RcInner {
            value,
            references: Cell::new(1),
        };

        Self {
            // SAFETY: Box::new does not give a null pointer
            // We need into_raw because otherwise the Box would 
            // be dropped at the end of the new() method.
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(inner))) }
        }
    }

    pub fn ptr_eq(rc1: &Self, rc2: &Self) -> bool {
        rc1.inner == rc2.inner
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        let n_references = inner.references.get();
        inner.references.set(n_references + 1);

        Self {
            inner: self.inner
        }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let n_references = inner.references.get();

        if n_references == 1 {
            // Being paranoid here, ensuring that this pointer to inner gets dropped before the 
            // box is dropped. If code after this if block attempted to use this inner pointer,
            // it would be invalid.
            drop(inner);

            // SAFETY: we hold the only reference to inner, so we can
            //         safely dereference it and then drop it
            let inner_box = unsafe { Box::from_raw(self.inner.as_ptr()) };
            drop(inner_box);
        } else {
            // SAFETY: there are other references to inner around, so don't drop inner
            inner.references.set(n_references - 1);
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.inner is a Box on the heap, that is only deallocated when the last Rc
        //         that references it gets dropped. This Rc exists, therefore there should still
        //         be a self.inner allocated.
        unsafe { &self.inner.as_ref().value }
    }
}

#[cfg(test)]
mod test {
    use super::Rc;

    #[test]
    fn test() {
        let x = Rc::new(5);
        assert_eq!(*x, 5);

        let y = Rc::clone(&x);
        assert_eq!(*x, 5);
        assert_eq!(*y, 5);

        assert!(Rc::ptr_eq(&x, &y));
    }

    #[test]
    fn test_drop_check() {
        let (y, x);
        x = String::from("foo");
        y = Rc::new(&x);
    }
}