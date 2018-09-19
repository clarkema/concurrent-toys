#[macro_use]
extern crate clap;
extern crate beanstalkd;

use beanstalkd::Beanstalkd;
use std::{process, thread, time};
use std::sync::mpsc;

fn main() {
    let matches = clap_app!(bs_client =>
        (version: "1.0")
        (author: "Mike Clarke <clarkema@clarkema.org>")
        (about: "A testing client for beanstalkd")
        (@arg HOST: -h --host +takes_value "defaults to 'localhost'")
        (@arg PORT: -p --port +takes_value "defaults to 11300")
    ).get_matches();

    let host = matches.value_of("HOST").unwrap_or("localhost").to_owned();
    let port = matches.value_of("PORT").unwrap_or("11300");
    let port: u16 = port.trim().parse().expect("Port must be an integer");

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut bd = match Beanstalkd::connect(&host, port) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{:?}: could not connect to {} on port {}", e, host, port);
                process::exit(1);
            }
        };

        loop {
            let val: String = rx.recv().unwrap();
            bd.put(&val, 100, 0, 10000)
                .expect("Failed to send to beanstalkd");
        }
    });

    let mut workers = vec![];

    for i in 0..10 {
        let tx = mpsc::Sender::clone(&tx);
        let h = thread::spawn(move || {
            loop {
                tx.send(format!("Worker {}: hello!", i)).unwrap();
                thread::sleep(time::Duration::from_millis(1000));
            }
        });
        workers.push(h);
    }

    for worker in workers {
        let _ = worker.join();
    }
}
