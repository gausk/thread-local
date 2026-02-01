#![allow(unused)]
use libc::{c_void, pthread_getspecific, pthread_key_create, pthread_key_t, pthread_setspecific};
use std::marker::PhantomData;
use std::sync::OnceLock;

pub struct ThreadLocal<T> {
    key: pthread_key_t,
    init: fn() -> T,
    _marker: PhantomData<fn() -> T>,
}

unsafe extern "C" fn destructor<T>(ptr: *mut c_void) {
    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr as *mut T) })
    }
}

impl<T> ThreadLocal<T> {
    pub fn new(init: fn() -> T) -> ThreadLocal<T> {
        let mut key: pthread_key_t = 0;
        let ret = unsafe { pthread_key_create(&mut key, Some(destructor::<T>)) };
        assert_eq!(ret, 0);
        Self {
            init,
            key,
            _marker: PhantomData,
        }
    }

    pub fn with<R>(&self, caller: impl FnOnce(&T) -> R) -> R {
        let ptr = unsafe { pthread_getspecific(self.key) };
        let value_ptr = if ptr.is_null() {
            let boxed = Box::new((self.init)());
            let raw = Box::into_raw(boxed);
            unsafe { pthread_setspecific(self.key, raw as *mut c_void) };
            raw
        } else {
            ptr as *mut T
        };
        caller(unsafe { &*value_ptr })
    }
}

pub struct LocalKey<T> {
    inner: OnceLock<ThreadLocal<T>>,
    init: fn() -> T,
}

impl<T> LocalKey<T> {
    const fn new(init: fn() -> T) -> LocalKey<T> {
        LocalKey {
            inner: OnceLock::new(),
            init,
        }
    }

    fn get(&'static self) -> &'static ThreadLocal<T> {
        self.inner.get_or_init(|| ThreadLocal::new(self.init))
    }

    fn with<R>(&'static self, caller: impl FnOnce(&T) -> R) -> R {
        self.get().with(caller)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::thread::spawn;

    #[test]
    fn test_thread_local() {
        static RNG: LocalKey<RefCell<u32>> = LocalKey::new(|| RefCell::new(1));

        fn increment() {
            RNG.with(|c| {
                *c.borrow_mut() += 1;
            })
        }

        fn counter_value(i: i32) {
            RNG.with(|c| {
                println!(
                    "In libc implemented thread local, thread  {}, RNG value is {}",
                    i,
                    c.borrow()
                );
            })
        }

        let handles = (1..5)
            .map(|i| {
                spawn(move || {
                    for _ in 0..5 {
                        increment();
                    }
                    counter_value(i);
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }

        counter_value(0);
    }
}
