use std::{ops::{DerefMut, Deref}, cell::UnsafeCell};
use crate::cell::Cell;

#[derive(Copy, Clone, Debug)]
enum RefState {
    Unused,
    Shared(usize),
    Exclusive,
}

#[derive(Debug)]
pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<RefState>,
}

impl<T> RefCell<T> {
   pub fn new(value: T) -> Self {
       Self {
           value: UnsafeCell::new(value),
           state: Cell::new(RefState::Unused),
       }
   }

   pub fn borrow(&self) -> Option<Ref<'_, T>> {
       match self.state.get() {
           RefState::Unused => {
               self.state.set(RefState::Shared(1));
               Some(Ref {
                   refcell: self
               })
           }
           RefState::Shared(n) => {
               self.state.set(RefState::Shared(n + 1));
               Some(Ref {
                   refcell: self
               })
           }
           RefState::Exclusive => None,
       }
   }

   pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
       match self.state.get() {
           RefState::Unused => {
               self.state.set(RefState::Exclusive);
               Some(RefMut {
                   refcell: self
               })
           }
           RefState::Shared(_) => None,
           RefState::Exclusive => None,
       }
   }
}

pub struct Ref<'refcell, T> {
    refcell: &'refcell RefCell<T>
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Unused => unreachable!(),
            RefState::Shared(1) => {
                self.refcell.state.set(RefState::Unused);
            }
            RefState::Shared(n) => {
                self.refcell.state.set(RefState::Shared(n - 1));
            }
            RefState::Exclusive => unreachable!(),
        }
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: This reference should only exist when state is RefState::Shared,
        //         there are no mutable references in existence when it is dereferenced here.
        // SAFETY: This is not thread safe, but that's okay because RefCell is !Sync due
        //         to UnsafeCell being !Sync
        unsafe { &*self.refcell.value.get() }
    }
}

#[derive(Debug)]
pub struct RefMut<'refcell, T> {
    refcell: &'refcell RefCell<T>
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Unused => unreachable!(),
            RefState::Shared(_) => unreachable!(),
            RefState::Exclusive => {
                self.refcell.state.set(RefState::Unused);
            }
        }
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: This reference should only exist when state is RefState::Exclusive,
        //         and this is the only reference capable of mutating the value, because
        //         this is the reference holding the exlusive lock.
        // SAFETY: This is not thread safe, but that's okay because RefCell is !Sync due
        //         to UnsafeCell being !Sync
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: This reference should only exist when state is RefState::Exclusive,
        //         and this is the only reference capable of mutating the value, because
        //         this is the reference holding the exlusive lock. There shold be no
        //         other references around to be reading while this is mutating.
        // SAFETY: This is not thread safe, but that's okay because RefCell is !Sync due
        //         to UnsafeCell being !Sync
        unsafe { &mut *self.refcell.value.get() }
    }
}

#[cfg(test)]
mod test {
    use super::RefCell;
    #[test]
    fn test_deref() {
        let x = RefCell::new(20);

        // the borrow here is expected to be dropped immediately,
        // meaning that the subsequent borrow_mut should work.
        assert_eq!(x.borrow().map(|v| *v), Some(20));

        match x.borrow_mut() {
            Some(mut x_ref) => {
                *x_ref = 30;
            }
            None => panic!("expected to be able to borrow mut")
        };

        let x_ref1 = x.borrow();

        // there is still a reference x_ref1 around which hasn't been dropped
        // therefore we cannot expect to be able to borrow mutably
        assert!(x.borrow_mut().is_none()); 

        // after the borrow_mut to ensure it isn't dropped before
        assert_eq!(x_ref1.map(|v| *v), Some(30));
        let x_ref2 = x.borrow();
        assert_eq!(x_ref2.map(|v| *v), Some(30));
    }
}