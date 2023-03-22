use std::{sync::{RwLock, mpsc, Mutex}, collections::HashMap, thread, time::{Duration, Instant}};

use rand::Rng;
use hal::comms_hal::{Packet, NetworkAddress};

#[derive(Debug, Clone)]
pub enum ObserverEvent {
    EventResponse(u64, Result<(), String>),
    PacketReceived(Packet),
    SendPacket {
        address: NetworkAddress,
        packet: Packet,
    },
}

struct ObserverData {
    notify_txs: Vec<mpsc::Sender<(u64, ObserverEvent)>>,
    receive_tx: mpsc::Sender<(u64, ObserverEvent)>,
    receive_rx: mpsc::Receiver<(u64, ObserverEvent)>,
}

pub struct ObserverHandler {
    observers: RwLock<HashMap<thread::ThreadId, ObserverData>>,
    global_notify_txs: Mutex<Vec<mpsc::Sender<(u64, ObserverEvent)>>>,
}

impl ObserverHandler {
    pub fn new() -> Self {
        Self {
            observers: RwLock::new(HashMap::new()),
            global_notify_txs: Mutex::new(Vec::new()),
        }
    }

    pub fn register_observer_thread(&self) {
        let mut observers = self.observers.write().unwrap();

        if !observers.contains_key(&thread::current().id()) {
            let (tx, rx) = mpsc::channel();

            // Update the other observers with the new tx handle
            for observer in observers.values_mut() {
                observer.notify_txs.push(tx.clone());
            }

            // Make a new observer
            let notify_txs = observers
                .values()
                .map(|observer| observer.receive_tx.clone())
                .collect();

            self.global_notify_txs.lock().unwrap().push(tx.clone());
            observers.insert(thread::current().id(), ObserverData {
                notify_txs,
                receive_tx: tx,
                receive_rx: rx,
            });
        }
    }

    pub fn notify(&self, event: ObserverEvent) -> u64 {
        let observers = self.observers.read().unwrap();
        let event_id = self.gen_event_id();

        if let Some(observer) = observers.get(&thread::current().id()) {
            for tx in &observer.notify_txs {
                tx.send((event_id, event.clone())).unwrap();
            }
        } else {
            eprintln!("notify_observer: Observer thread {:?} not registered", thread::current().id());
        }

        event_id
    }

    pub fn notify_global(&self, event: ObserverEvent) -> u64 {
        let event_id = self.gen_event_id();
        let global_notify_txs = self.global_notify_txs.lock().unwrap();

        for tx in global_notify_txs.iter() {
            tx.send((event_id, event.clone())).unwrap();
        }

        event_id
    }

    pub fn get_response(&self, event_id: u64, timeout: Duration) -> Option<Result<(), String>> {
        let thread_id = thread::current().id();

        if !self.observers.read().unwrap().contains_key(&thread_id) {
            let msg = format!("get_response: Observer thread {:?} not registered", thread_id);
            eprintln!("{msg}");
            return Some(Err(msg));
        }

        let start_time = Instant::now();

        while start_time.elapsed() < timeout {
            if let Some((_, event)) = self.wait_event(Duration::from_millis(1)) {
                if let ObserverEvent::EventResponse(response_id, response) = event {
                    if response_id == event_id {
                        return Some(response);
                    }
                }
            }
        }

        None
    }

    // pub fn get_events(&self) -> Vec<(u64, ObserverEvent)> {
    //     let observers = self.observers.read().unwrap();

    //     if let Some(observer) = observers.get(&thread::current().id()) {
    //         let mut events = Vec::new();

    //         while let Ok(event) = observer.receive_rx.try_recv() {
    //             events.push(event);
    //         }

    //         events
    //     } else {
    //         eprintln!("get_events: Observer thread {:?} not registered", thread::current().id());
    //         Vec::new()
    //     }
    // }

    pub fn wait_event(&self, timeout: Duration) -> Option<(u64, ObserverEvent)> {
        let observers = self.observers.read().unwrap();

        if let Some(observer) = observers.get(&thread::current().id()) {
            observer.receive_rx.recv_timeout(timeout).ok()
        } else {
            eprintln!("wait_event: Observer thread {:?} not registered", thread::current().id());
            None
        }
    }

    fn gen_event_id(&self) -> u64 {
        rand::thread_rng().gen()
    }
}

unsafe impl Send for ObserverHandler {}
unsafe impl Sync for ObserverHandler {}