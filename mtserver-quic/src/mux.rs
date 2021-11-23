use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mtinput::MtInput;
use quic_common::tokio::sync::{Mutex, MutexGuard, Notify};
use quic_common::tokio::time::sleep_until;
use quic_common::{ReportTransport, DATAGRAM_SIZE};
use utils::Report;

struct MultiplexerBackend {
    mt: MtInput,
}

impl MultiplexerBackend {
    fn new() -> Self {
        MultiplexerBackend { mt: MtInput::new() }
    }

    fn send(&mut self, events: &[Report; 10]) {
        self.mt.dispatch(events);
    }

    fn reset(&mut self) {
        const BLANK_EVENTS: [Report; 10] = [Report::None; 10];
        self.mt.dispatch(&BLANK_EVENTS);
    }
}

pub struct InputMultiplexer {
    current_connection: AtomicU64,
    id_counter: AtomicU64,
    backend: Mutex<MultiplexerBackend>,
    notify: Notify,
    epoch: Instant,
    next_gc: AtomicU64,
}

impl InputMultiplexer {
    pub fn new() -> (Arc<Self>, impl Future<Output = ()>) {
        let mux = Arc::new(InputMultiplexer {
            current_connection: AtomicU64::new(!0),
            id_counter: AtomicU64::new(0),
            backend: Mutex::new(MultiplexerBackend::new()),
            notify: Notify::new(),
            epoch: Instant::now(),
            next_gc: AtomicU64::new(0),
        });
        (mux.clone(), async move {
            loop {
                mux.notify.notified().await;
                loop {
                    let timeout = match mux.gc_wait() {
                        Some(x) => x,
                        None => break,
                    };
                    sleep_until(quic_common::tokio::time::Instant::from_std(timeout)).await;
                }
                mux.gc().await;
            }
        })
    }

    async fn gc(&self) {
        let mut backend = self.backend.lock().await;
        if self.gc_wait().is_some() {
            return;
        }
        self.current_connection.store(!0, Ordering::SeqCst);
        backend.reset();
    }

    #[must_use]
    fn gc_wait(&self) -> Option<Instant> {
        let timeout = self.epoch + Duration::from_millis(self.next_gc.load(Ordering::SeqCst));
        if Instant::now() >= timeout {
            return None;
        }
        Some(timeout)
    }

    fn kick_watchdog(&self) {
        let next = u64::try_from((Instant::now() - self.epoch).as_millis()).unwrap() + 1000;
        if self.next_gc.fetch_max(next, Ordering::SeqCst) < next {
            self.notify.notify_one();
        }
    }

    fn try_lock(&self, id: u64) -> Option<MutexGuard<MultiplexerBackend>> {
        if self.current_connection.load(Ordering::SeqCst) != id {
            return None;
        }
        let backend = match self.backend.try_lock() {
            Ok(x) => x,
            Err(_) => return None,
        };
        if self.current_connection.load(Ordering::SeqCst) != id {
            return None;
        }
        Some(backend)
    }

    fn send(&self, id: u64, events: &[Report; 10]) {
        let mut backend = match self.try_lock(id) {
            Some(x) => x,
            None => return,
        };
        backend.send(events);
        self.kick_watchdog();
    }

    async fn claim_send(&self, id: u64, events: &[Report; 10]) {
        self.current_connection.store(id, Ordering::SeqCst);
        let mut backend = self.backend.lock().await;
        self.current_connection.store(id, Ordering::SeqCst);
        backend.reset();
        backend.send(events);
        self.kick_watchdog();
    }

    fn release(&self, id: u64) {
        let mut backend = match self.try_lock(id) {
            Some(x) => x,
            None => return,
        };
        self.current_connection.store(!0, Ordering::SeqCst);
        backend.reset();
    }

    fn fork(&self) -> u64 {
        let result = self.id_counter.fetch_add(1, Ordering::SeqCst) + 1;
        assert_ne!(result, !0);
        result
    }
}

pub struct InputConnection {
    parent: Arc<InputMultiplexer>,
    transport: ReportTransport,
    id: u64,
    was_active: bool,
}

impl InputConnection {
    pub fn new(parent: Arc<InputMultiplexer>) -> Self {
        let id = parent.fork();
        InputConnection {
            parent,
            id,
            transport: ReportTransport::new(),
            was_active: false,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn send(&mut self, data: &[u8; DATAGRAM_SIZE]) -> Result<(), ()> {
        if !self.transport.deserialize(data)? {
            return Ok(());
        }

        let reports = self.transport.take();
        #[inline]
        fn are_events_active(events: &[Report; 10]) -> bool {
            !events.iter().all(|x| *x == Report::None)
        }
        let was_active = self.was_active;
        let is_active = are_events_active(reports);
        self.was_active = is_active;

        if !is_active {
            if was_active {
                self.parent.release(self.id);
            }
            return Ok(());
        }
        if was_active {
            self.parent.send(self.id, reports);
        } else {
            self.parent.claim_send(self.id, reports).await;
        }
        Ok(())
    }
}

impl Drop for InputConnection {
    fn drop(&mut self) {
        self.parent.release(self.id);
    }
}
