mod thread_local_simple;

use std::cell::RefCell;
use std::thread::spawn;

thread_local! {
    static RNG: RefCell<u32> = const { RefCell::new(0) };
}

fn increment() {
    RNG.with(|c| {
        *c.borrow_mut() += 1;
    })
}

fn counter_value(i: i32) {
    RNG.with(|c| {
        println!("In thread {}, RNG value is {}", i, c.borrow());
    })
}

fn main() {
    let handles = (1..5)
        .map(|i| {
            spawn(move || {
                for _ in 0..20 {
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
