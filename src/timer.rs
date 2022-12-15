use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct Timer {
    tx: mpsc::Sender<u8>,
    rx: mpsc::Receiver<u8>,
    last: u8,
}

impl Timer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let last = 0;
        Self { tx, rx, last }
    }

    pub fn set(&mut self, i: u8) {
        let tx = self.tx.clone();
        tx.send(i).unwrap();
        thread::spawn(move || {
            for j in (0 .. i).rev() {
                thread::sleep(Duration::from_micros(16600));
                tx.send(j).unwrap();
            }
        });
    }

    pub fn get(&mut self) -> u8 {
        while let Ok(i) = self.rx.try_recv() {
            self.last = i;
        }

        self.last
    }
}
