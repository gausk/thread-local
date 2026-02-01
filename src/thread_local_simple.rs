#![allow(unused)]
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

static STORAGE: OnceLock<Mutex<HashMap<ThreadId, Box<dyn std::any::Any + Send>>>> = OnceLock::new();

pub struct ThreadLocal<T> {
    init: fn() -> T,
}

impl<T: Send + 'static> ThreadLocal<T> {
    const fn new(init: fn() -> T) -> Self {
        Self { init }
    }

    fn with<R>(&self, caller: impl FnOnce(&T) -> R) -> R {
        let thread_id = std::thread::current().id();
        let storage = STORAGE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = storage.lock().unwrap();
        let entry = map
            .entry(thread_id)
            .or_insert_with(|| Box::new((self.init)()) as Box<dyn std::any::Any + Send>);
        let value = entry.downcast_ref::<T>().unwrap();
        caller(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::thread_local_simple::ThreadLocal;
    use std::cell::RefCell;
    use std::thread::spawn;

    #[test]
    fn test_simple_thread_local() {
        const RNG: ThreadLocal<RefCell<u32>> = ThreadLocal::new(|| RefCell::new(1));

        fn increment() {
            RNG.with(|c| {
                *c.borrow_mut() += 1;
            })
        }

        fn counter_value(i: i32) {
            RNG.with(|c| {
                println!(
                    "In simple implemented thread local, thread  {}, RNG value is {}",
                    i,
                    c.borrow()
                );
            })
        }

        let handles = (1..5)
            .map(|i| {
                spawn(move || {
                    for _ in 0..10 {
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
