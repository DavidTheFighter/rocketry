use std::{
    collections::HashMap, sync::{mpsc, RwLock}, thread, time::{Duration, Instant}
};

use dashmap::DashMap;
use rand::Rng;
use shared::comms_hal::{NetworkAddress, Packet};

#[derive(Debug, Clone)]
pub enum ObserverEvent {
    EventResponse(u64, Result<ObserverResponse, String>),
    PacketReceived {
        address: NetworkAddress,
        ip: [u8; 4],
        packet: Packet,
    },
    SendPacket {
        address: NetworkAddress,
        packet: Packet,
    },
    SetupBrowserStream {
        camera_address: NetworkAddress,
        browser_session: String,
    },
    UpdateBitrate {
        source_address: NetworkAddress,
        bitrate: u32,
    },
    AggregateTelemetry {
        controller: NetworkAddress,
        json: String,
    },
}

#[derive(Debug, Clone)]
pub enum ObserverResponse {
    Empty,
    BrowserStream { stream_session: String },
}

struct ObserverData {
    receive_tx: mpsc::Sender<(u64, ObserverEvent)>,
    receive_rx: mpsc::Receiver<(u64, ObserverEvent)>,
}

struct ObserverNotifyData {
    tx: mpsc::Sender<(u64, ObserverEvent)>,
    thread_id: thread::ThreadId,
    filter_fns: HashMap<String, Box<dyn Fn(&ObserverEvent) -> bool>>,
}

pub struct ObserverHandler {
    observers: DashMap<thread::ThreadId, ObserverData>,
    global_notify_txs: RwLock<Vec<ObserverNotifyData>>,
}

impl ObserverHandler {
    pub fn new() -> Self {
        Self {
            observers: DashMap::new(),
            global_notify_txs: RwLock::new(Vec::new()),
        }
    }

    pub fn register_observer_thread(&self) -> bool {
        if !self.observers.contains_key(&thread::current().id()) {
            let (tx, rx) = mpsc::channel();

            self.global_notify_txs
                .write()
                .expect("global_notify_txs write lock")
                .push(ObserverNotifyData {
                    tx: tx.clone(),
                    thread_id: thread::current().id(),
                    filter_fns: HashMap::new(),
                });

            self.observers.insert(
                thread::current().id(),
                ObserverData {
                    receive_tx: tx,
                    receive_rx: rx,
                },
            );

            true
        } else {
            false
        }
    }

    pub fn register_subscription_filter(&self, filter_id: &str, filter_fn: impl Fn(&ObserverEvent) -> bool + 'static) {
        if let Some(notify_data) = self
            .global_notify_txs
            .write()
            .expect("global_notify_txs write lock")
            .iter_mut()
            .find(|notify_data| notify_data.thread_id == thread::current().id())
        {
            notify_data.filter_fns.insert(String::from(filter_id), Box::new(filter_fn));
        } else {
            eprintln!(
                "register_subscription_filter: Observer thread {:?} not registered",
                thread::current().id()
            );
        }
    }

    pub fn notify(&self, event: ObserverEvent) -> u64 {
        let event_id = self.gen_event_id();

        for notify_data in self
            .global_notify_txs
            .read()
            .expect("global_notify_txs read lock")
            .iter()
        {
            if notify_data.thread_id != thread::current().id() {
                if notify_data.filter_fns.len() == 0 || notify_data.filter_fns.iter().any(|(_, filter_fn)| filter_fn(&event)) {
                    notify_data.tx.send((event_id, event.clone())).unwrap();
                }
            }
        }

        event_id
    }

    pub fn get_response(
        &self,
        event_id: u64,
        timeout: Duration,
    ) -> Option<Result<ObserverResponse, String>> {
        let thread_id = thread::current().id();

        if !self.observers.contains_key(&thread_id) {
            let msg = format!(
                "get_response: Observer thread {:?} not registered",
                thread_id
            );
            eprintln!("{msg}");
            return Some(Err(msg));
        }

        let start_time = Instant::now();

        while start_time.elapsed() < timeout {
            if let Some((_, event)) = self.wait_event(Duration::from_millis(1)) {
                if let ObserverEvent::EventResponse(response_id, response) = &event {
                    if *response_id == event_id {
                        return Some(response.clone());
                    } else {
                        // Put the event back in the queue so we don't miss it later
                        let observer = self.observers.get(&thread_id).unwrap();
                        observer.receive_tx.send((event_id, event)).unwrap();
                    }
                } else {
                    // Put the event back in the queue so we don't miss it later
                    let observer = self.observers.get(&thread_id).unwrap();
                    observer.receive_tx.send((event_id, event)).unwrap();
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
        if let Some(observer) = self.observers.get(&thread::current().id()) {
            observer.receive_rx.recv_timeout(timeout).ok()
        } else {
            eprintln!(
                "wait_event: Observer thread {:?} not registered",
                thread::current().id()
            );
            None
        }
    }

    pub fn get_event(&self) -> Option<(u64, ObserverEvent)> {
        if let Some(observer) = self.observers.get(&thread::current().id()) {
            observer.receive_rx.try_recv().ok()
        } else {
            eprintln!(
                "wait_event: Observer thread {:?} not registered",
                thread::current().id()
            );
            None
        }
    }

    pub fn get_num_observers(&self) -> usize {
        self.observers.len()
    }

    fn gen_event_id(&self) -> u64 {
        rand::thread_rng().gen()
    }
}

unsafe impl Send for ObserverHandler {}
unsafe impl Sync for ObserverHandler {}
