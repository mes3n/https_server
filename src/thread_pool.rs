use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

type Job = Box<dyn FnOnce() + Send>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool {
    pub fn new(limit: usize) -> ThreadPool {
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(limit);
        for _ in 0..limit {
            workers.push(Worker::new(receiver.clone()));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute(&self, job: Job) {
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // println!("Sending terminate message to all workers.");
        for _ in &self.workers {
            match self.sender.send(Message::Terminate) {
                Err(e) => println!("Failed to send terminate message to worker. {e:?}"),
                _ => {}
            };
        }

        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                match handle.join() {
                    Err(e) => println!("Error joining worker thread. {e:?}"),
                    _ => {}
                };
            }
        }
        println!("All workers shut down.");
    }
}

struct Worker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let handle = thread::spawn(move || loop {
            let receiver = match receiver.lock() {
                Ok(receiver) => receiver,
                Err(_) => {
                    println!("Couldn't lock thread for worker.");
                    continue;
                }
            };
            let message = match receiver.recv() {
                Ok(message) => message,
                Err(_) => {
                    println!("Couldn't receive message for worker.");
                    continue;
                }
            };
            match message {
                Message::NewJob(job) => {
                    println!("Worker got a job.");
                    job();
                    println!("Job done.");
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker {
            handle: Some(handle),
        }
    }
}
