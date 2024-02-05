use std::sync::{mpsc, Arc, Mutex};
use crate::mpsc::Sender;
use crate::mpsc::Receiver;
use std::thread;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;


struct Task {
    id: i32,
    payload: String
}

fn create_task(id: i32, payload: String) -> Task {
    return Task { id, payload };
}

struct Worker {
    id: i32,
}

impl Worker {
    fn process_task(&self, task: Task) -> String {
        return format!("task with id:{} with payload: {} has been processed by worker: {}", task.id, task.payload, self.id); 
    }
}

fn create_worker(id: i32) -> Worker {
    return Worker{id};
}

static NWORKERS: i32 = 3;
static NTASKS: i32 = 30;

fn main() {
    let (sx, rx): (Sender<Task>, Receiver<Task>) = mpsc::channel();
    let (sx1, rx1): (Sender<String>, Receiver<String>) = mpsc::channel();
    // Wrap the receiver in an Arc and Mutex
    let receiver = Arc::new(Mutex::new(rx));

    let mut children = Vec::new();

    for id in 0..NWORKERS {
        let worker_rx = Arc::clone(&receiver);
        let worker = create_worker(id);
        let worker_sx1 = sx1.clone();
        let worker_handle = thread::spawn(move || {
            loop {
                let rx = worker_rx.lock().unwrap();
                match rx.recv() {
                    Ok(task) => worker_sx1.send(worker.process_task(task)),
                    Err(_) => break,
                };
                drop(rx);
            }
            drop(worker_sx1);
        });
        children.push(worker_handle);
    }

    for task_num in 1..=NTASKS {
        let random_string: String = thread_rng()
                                        .sample_iter(&Alphanumeric)
                                        .take(16)
                                        .map(char::from)
                                        .collect();
        sx.send(create_task(task_num, random_string));
    }

    drop(sx);

    for received in rx1 {
        println!("Received: {}", received);
    }

    // loop {
    //     match rx1.recv() {
    //         Ok(data) => println!("{}", data),
    //         Err(_) => break,
    //     }
    // }


    // Wait for the worker threads to finish
    for handle in children {
        handle.join().unwrap();
    }
}