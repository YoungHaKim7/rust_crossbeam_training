use std::{cell::Cell, iter, marker::PhantomData, sync::Arc};

use crossbeam::{
    deque::{Injector, Stealer, Worker},
    epoch::Atomic,
    utils::CachePadded,
};

const MIN_CAP: usize = 65000;

use std::sync::atomic::AtomicIsize;

struct Task; // Placeholder for Task struct

struct ThreadData<'a, T> {
    injector: &'a Injector<T>, // common global queue
    task_q: Worker<T>,         // local queue
    stealers: Vec<Stealer<T>>, // stealers for other threads local queue
}

impl<'a, T: std::marker::Send> ThreadData<'a, T> {
    fn spawn(mut self) -> Option<std::thread::JoinHandle<()>> {
        let thread = std::thread::spawn(move || {
            find_task(&mut self.task_q, &self.injector, &self.stealers);
        });
        Some(thread)
    }
}

// struct Worker<T> {
//     stealer: Stealer<T>, // to be shared with other threads
//     thread: Option<std::thread::JoinHandle<()>>,
// }

// impl<t> worker<t> {
//     pub fn new_fifo() -> worker<t> {
//         let buffer = buffer::alloc(min_cap);

//         let inner = arc::new(cachepadded::new(inner {
//             front: atomicisize::new(0),
//             back: atomicisize::new(0),
//             buffer: cachepadded::new(atomic::new(buffer)),
//         }));

//         worker {
//             inner,
//             buffer: cell::new(buffer),
//             flavor: flavor::fifo,
//             _marker: phantomdata,
//             stealer: todo!(), // not implemented in provided code
//             thread: none,     // not implemented in provided code
//         }
//     }
// }

struct Factory {
    injector: Injector<Task>, // owner of the global queue
    workers: Vec<Worker<Task>>,
}

impl Factory {
    fn new() -> Self {
        Self {
            injector: Injector::<Task>::new(),
            workers: Vec::new(),
        }
    }

    fn build_threadpool(mut self) {
        let mut t1 = ThreadData {
            injector: &self.injector,
            task_q: Worker::<Task>::new_fifo(),
            stealers: Vec::new(),
        };
        let w1 = Worker {
            stealer: t1.task_q.stealer(),
            thread: None,
        };

        let mut t2 = ThreadData {
            injector: &self.injector,
            task_q: Worker::<Task>::new_fifo(),
            stealers: Vec::new(),
        };
        let w2 = Worker {
            stealer: t2.task_q.stealer(),
            thread: None,
        };

        let mut t3 = ThreadData {
            injector: &self.injector,
            task_q: Worker::<Task>::new_fifo(),
            stealers: Vec::new(),
        };
        let w3 = Worker {
            stealer: t3.task_q.stealer(),
            thread: None,
        };

        t1.stealers.push(w2.stealer.clone());
        t1.stealers.push(w3.stealer.clone());

        t2.stealers.push(w1.stealer.clone());
        t2.stealers.push(w3.stealer.clone());

        t3.stealers.push(w1.stealer.clone());
        t3.stealers.push(w2.stealer.clone());

        // launch threads and save workers
        w1.thread = t1.spawn();
        w2.thread = t2.spawn();
        w3.thread = t3.spawn();

        self.workers.push(w1);
        self.workers.push(w2);
        self.workers.push(w3);
    }
}

fn find_task<T>(local: &Worker<T>, global: &Injector<T>, stealers: &[Stealer<T>]) -> Option<T> {
    // Placeholder function, implementation not provided in the code
    None
}

fn main() {
    let my_test = Task.into();
}
