//! Preparatory policy for local Windows underlay notifications, not wired to
//! callbacks or reconnect yet. No OS, address, profile or network side effects.
//! Caller supplies monotonic elapsed time and connection generation.
use std::time::Duration;

const QUIET: Duration = Duration::from_secs(2);
const MAX_BURST: Duration = Duration::from_secs(10);

#[derive(Debug, Default)]
pub struct Changes {
    generation: u64,
    own_interface: Option<u64>,
    enabled: bool,
    burst: Option<(Duration, Duration)>,
}

impl Changes {
    /// Reset on every new intent, ownership change or recovery preference change.
    pub fn reset(&mut self, generation: u64, own_interface: Option<u64>, enabled: bool) {
        self.generation = generation;
        self.own_interface = own_interface;
        self.enabled = enabled;
        self.burst = None;
    }

    /// Initial notifications are registration confirmation, not network changes.
    /// A missing interface identity is not enough evidence to schedule work.
    pub fn observe(&mut self, generation: u64, now: Duration, interface: Option<u64>, initial: bool) {
        if generation != self.generation || !self.enabled || self.own_interface.is_none()
            || initial || interface.is_none() || interface == self.own_interface { return; }
        match &mut self.burst {
            Some((_, last)) => *last = (*last).max(now),
            None => self.burst = Some((now, now)),
        }
    }

    /// A due burst requests a local underlay-state read, NEVER a reconnect.
    /// That read must establish a useful change before recovery is considered.
    pub fn take_due(&mut self, generation: u64, now: Duration) -> bool {
        if generation != self.generation { return false; }
        let Some((first, last)) = self.burst else { return false; };
        if now < last.saturating_add(QUIET).min(first.saturating_add(MAX_BURST)) { return false; }
        self.burst = None;
        true
    }
}

// Pure coordinator groundwork only. Tokens describe caller-supplied reads;
// this code never registers notifications, reads the OS or starts recovery.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct ReadToken { generation: u64, sequence: u64 }

#[cfg(test)]
struct Coordinator<T> {
    changes: Changes,
    generation: u64,
    enabled: bool,
    sequence: u64,
    pending: Option<ReadToken>,
    baseline: Option<T>,
}

#[cfg(test)]
impl<T: Eq> Coordinator<T> {
    fn new() -> Self {
        Self { changes: Changes::default(), generation: 0, enabled: false,
            sequence: 0, pending: None, baseline: None }
    }
    // A new intent/owner/preference always discards the old baseline and token.
    // The optional token requests a baseline from the caller, not an OS read.
    fn reset(&mut self, generation: u64, own: Option<u64>, enabled: bool) -> Option<ReadToken> {
        self.changes.reset(generation, own, enabled);
        self.generation = generation;
        self.enabled = enabled && own.is_some();
        self.pending = None;
        self.baseline = None;
        if self.enabled { Some(self.token()) } else { None }
    }
    fn token(&mut self) -> ReadToken {
        self.sequence = self.sequence.wrapping_add(1);
        let token = ReadToken { generation: self.generation, sequence: self.sequence };
        self.pending = Some(token);
        token
    }
    fn observe(&mut self, generation: u64, now: Duration, interface: Option<u64>, initial: bool) {
        self.changes.observe(generation, now, interface, initial);
    }
    fn take_due(&mut self, generation: u64, now: Duration) -> Option<ReadToken> {
        if self.enabled && self.changes.take_due(generation, now) { Some(self.token()) } else { None }
    }
    // Only a current requested success can advance the baseline. A failure
    // preserves it; the first success establishes it without indicating change.
    // true means local state changed, NEVER that a reconnect should be started.
    fn complete(&mut self, token: ReadToken, result: Result<T, ()>) -> bool {
        if !self.enabled || token.generation != self.generation || self.pending != Some(token) { return false; }
        self.pending = None;
        let Ok(snapshot) = result else { return false; };
        let changed = self.baseline.as_ref().is_some_and(|previous| previous != &snapshot);
        self.baseline = Some(snapshot);
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t(n: u64) -> Duration { Duration::from_secs(n) }
    fn active() -> Changes { let mut c=Changes::default(); c.reset(7,Some(11),true); c }
    #[test]
    fn initial_own_unknown_and_stale_notifications_do_nothing() {
        let mut c=active();
        for (g,i,initial) in [(7,Some(22),true),(7,Some(11),false),(7,None,false),(6,Some(22),false)] {
            c.observe(g,t(0),i,initial);
        }
        assert!(!c.take_due(7,t(100)));
    }
    #[test]
    fn physical_events_coalesce_until_quiet_then_fire_once() {
        let mut c=active(); c.observe(7,t(0),Some(22),false);
        c.observe(7,t(1),Some(33),false);
        assert!(!c.take_due(7,t(2)));
        assert!(c.take_due(7,t(3)));
        assert!(!c.take_due(7,t(100)));
    }
    #[test]
    fn noisy_network_cannot_postpone_read_forever() {
        let mut c=active();
        for n in 0..10 { c.observe(7,t(n),Some(22),false); assert!(!c.take_due(7,t(n))); }
        assert!(c.take_due(7,t(10)));
    }
    #[test]
    fn disconnect_disable_and_new_intent_cancel_pending_burst() {
        for (g,own,enabled) in [(8,None,true),(8,Some(11),false),(8,Some(44),true)] {
            let mut c=active(); c.observe(7,t(0),Some(22),false); c.reset(g,own,enabled);
            assert!(!c.take_due(7,t(100))); assert!(!c.take_due(g,t(100)));
            c.observe(7,t(100),Some(22),false); assert!(!c.take_due(g,t(200)));
        }
    }
    #[test]
    fn external_owner_and_disabled_never_start_a_burst() {
        for (own,enabled) in [(None,true),(Some(11),false)] {
            let mut c=Changes::default(); c.reset(7,own,enabled);
            c.observe(7,t(0),Some(22),false); assert!(!c.take_due(7,t(100)));
        }
    }
    fn due(c: &mut Coordinator<u8>, generation: u64, now: u64) -> ReadToken {
        c.observe(generation,t(now),Some(22),false);
        assert!(c.take_due(generation,t(now + 1)).is_none());
        c.take_due(generation,t(now + 2)).expect("quiet burst requests one supplied read")
    }
    #[test]
    fn coordinator_baselines_then_reports_only_changed_successful_reads() {
        let mut c=Coordinator::new();
        let first=c.reset(7,Some(11),true).unwrap();
        assert!(!c.complete(first,Ok(1)));
        let same=due(&mut c,7,0); assert!(!c.complete(same,Ok(1)));
        let failed=due(&mut c,7,3); assert!(!c.complete(failed,Err(())));
        let changed=due(&mut c,7,6); assert!(c.complete(changed,Ok(2)));
        assert!(!c.complete(changed,Ok(3))); // completion is single-use
        let same=due(&mut c,7,9); assert!(!c.complete(same,Ok(2)));
    }
    #[test]
    fn coordinator_failed_initial_read_does_not_invent_a_change() {
        let mut c=Coordinator::new();
        let first=c.reset(7,Some(11),true).unwrap();
        assert!(!c.complete(first,Err(())));
        let baseline=due(&mut c,7,0); assert!(!c.complete(baseline,Ok(2)));
        let changed=due(&mut c,7,3); assert!(c.complete(changed,Ok(3)));
    }
    #[test]
    fn coordinator_discards_stale_generation_and_superseded_reads() {
        let mut c=Coordinator::new();
        let old=c.reset(7,Some(11),true).unwrap();
        let current=c.reset(8,Some(11),true).unwrap();
        assert!(!c.complete(old,Ok(99))); assert!(!c.complete(current,Ok(1)));
        let stale=due(&mut c,8,0); let newest=due(&mut c,8,3);
        assert!(!c.complete(stale,Ok(99))); assert!(c.complete(newest,Ok(2)));
        let same=due(&mut c,8,6); assert!(!c.complete(same,Ok(2)));
        // Even resetting with the same generation invalidates the old token.
        let stale=due(&mut c,8,9); let baseline=c.reset(8,Some(33),true).unwrap();
        assert!(!c.complete(stale,Ok(99))); assert!(!c.complete(baseline,Ok(4)));
    }
    #[test]
    fn coordinator_reset_and_filtered_events_never_request_work() {
        let mut c=Coordinator::new();
        let baseline=c.reset(7,Some(11),true).unwrap(); assert!(!c.complete(baseline,Ok(1)));
        for (g,i,initial) in [(7,Some(22),true),(7,Some(11),false),(7,None,false),(6,Some(22),false)] {
            c.observe(g,t(0),i,initial);
        }
        assert!(c.take_due(7,t(100)).is_none());
        for (g,own,enabled) in [(8,None,true),(9,Some(11),false)] {
            let generation=c.generation; let stale=due(&mut c,generation,101);
            assert!(c.reset(g,own,enabled).is_none());
            assert!(!c.complete(stale,Ok(99)));
            c.observe(g,t(200),Some(22),false); assert!(c.take_due(g,t(300)).is_none());
            let baseline=c.reset(g,Some(11),true).unwrap(); assert!(!c.complete(baseline,Ok(2)));
        }
    }

}

// Test-only cancellation ownership contract. No Windows adapter or quit wiring.
#[cfg(test)]
mod cancellation_owner {
    use std::sync::{mpsc, Arc, Mutex};

    trait Operations: Send + 'static {
        fn register(&mut self) -> Result<(), ()>;
        // Caller-supplied comparison, not an OS snapshot read.
        fn read_changed(&mut self) -> Result<bool, ()>;
        fn cancel(&mut self) -> Result<(), ()>;
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Outcome { Complete, Stopped, Unavailable }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Ack { lifetime: u64, outcome: Outcome }
    struct State { stopped: bool }
    struct Owner {
        lifetime: u64,
        state: Arc<Mutex<State>>,
        start: Option<mpsc::Sender<()>>,
        changes: mpsc::Receiver<u64>,
        acknowledgements: mpsc::Receiver<Ack>,
    }
    impl Owner {
        fn new(lifetime: u64, mut operations: impl Operations) -> Self {
            let state=Arc::new(Mutex::new(State { stopped: false }));
            let worker_state=state.clone();
            let (start_tx,start_rx)=mpsc::channel();
            let (change_tx,changes)=mpsc::channel();
            let (ack_tx,acknowledgements)=mpsc::channel();
            std::thread::spawn(move || {
                let started=start_rx.recv().is_ok();
                let stopped=worker_state.lock().unwrap().stopped;
                let outcome=if !started || stopped { Outcome::Stopped }
                else if operations.register().is_err() { Outcome::Unavailable }
                else {
                    let read=if worker_state.lock().unwrap().stopped { Ok(false) }
                        else { operations.read_changed() };
                    // Always cancel an acquired registration, even after read
                    // failure or stop. An adapter must retain uncertain callback
                    // context itself if this reports failure.
                    let cancelled=operations.cancel();
                    let state=worker_state.lock().unwrap();
                    if cancelled.is_err() || read.is_err() { Outcome::Unavailable }
                    else if state.stopped { Outcome::Stopped }
                    else {
                        if read==Ok(true) { let _=change_tx.send(lifetime); }
                        Outcome::Complete
                    }
                };
                // Acknowledgement follows the cancellation attempt. It never
                // claims that a failed cancellation freed the callback context.
                let _=ack_tx.send(Ack { lifetime, outcome });
            });
            Self { lifetime,state,start:Some(start_tx),changes,acknowledgements }
        }
        fn start(&mut self) -> bool {
            // A lifetime can start once only, including after cancellation error.
            self.start.take().is_some_and(|tx| tx.send(()).is_ok())
        }
        fn stop(&mut self) {
            self.state.lock().unwrap().stopped=true;
            // Wake a worker that has not started, without registering anything.
            self.start.take();
        }
        fn take_changed(&self, current_lifetime: u64) -> bool {
            let state=self.state.lock().unwrap();
            let event=self.changes.try_recv().ok();
            !state.stopped && current_lifetime==self.lifetime && event==Some(self.lifetime)
        }
        fn wait_ack(&self) -> Ack { self.acknowledgements.recv().unwrap() }
    }
    impl Drop for Owner { fn drop(&mut self) { self.stop(); } }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Fake {
            registered: Arc<AtomicUsize>, cancelled: Arc<AtomicUsize>,
            entered: mpsc::Sender<()>, release: mpsc::Receiver<()>,
            registration: Result<(), ()>, read: Result<bool, ()>, cancellation: Result<(), ()>,
        }
        impl Operations for Fake {
            fn register(&mut self) -> Result<(), ()> {
                self.registered.fetch_add(1,Ordering::SeqCst); self.registration
            }
            fn read_changed(&mut self) -> Result<bool, ()> {
                self.entered.send(()).unwrap(); self.release.recv().unwrap(); self.read
            }
            fn cancel(&mut self) -> Result<(), ()> {
                self.cancelled.fetch_add(1,Ordering::SeqCst); self.cancellation
            }
        }
        fn fixture(lifetime: u64, registration: Result<(), ()>, read: Result<bool, ()>, cancellation: Result<(), ()>)
            -> (Owner,mpsc::Receiver<()>,mpsc::Sender<()>,Arc<AtomicUsize>,Arc<AtomicUsize>) {
            let registered=Arc::new(AtomicUsize::new(0)); let cancelled=Arc::new(AtomicUsize::new(0));
            let (entered_tx,entered)=mpsc::channel(); let (release,release_rx)=mpsc::channel();
            let owner=Owner::new(lifetime,Fake { registered:registered.clone(),cancelled:cancelled.clone(),
                entered:entered_tx,release:release_rx,registration,read,cancellation });
            (owner,entered,release,registered,cancelled)
        }
        #[test]
        fn owner_stop_before_start_never_registers_and_acknowledges() {
            let (mut owner,_,_,registered,cancelled)=fixture(1,Ok(()),Ok(true),Ok(()));
            owner.stop(); owner.stop(); assert!(!owner.start());
            assert_eq!(owner.wait_ack(),Ack { lifetime:1,outcome:Outcome::Stopped });
            assert_eq!(registered.load(Ordering::SeqCst),0); assert_eq!(cancelled.load(Ordering::SeqCst),0);
            assert!(!owner.take_changed(1));
        }
        #[test]
        fn owner_stop_during_supplied_read_discards_result_then_cancels_and_acks() {
            let (mut owner,entered,release,registered,cancelled)=fixture(2,Ok(()),Ok(true),Ok(()));
            assert!(owner.start()); entered.recv().unwrap(); owner.stop(); owner.stop();
            assert!(!owner.take_changed(2)); assert!(owner.acknowledgements.try_recv().is_err());
            release.send(()).unwrap();
            assert_eq!(owner.wait_ack(),Ack { lifetime:2,outcome:Outcome::Stopped });
            assert_eq!(registered.load(Ordering::SeqCst),1); assert_eq!(cancelled.load(Ordering::SeqCst),1);
            assert!(!owner.take_changed(2)); assert!(!owner.start());
        }
        #[test]
        fn owner_success_and_read_failure_cancel_exactly_once() {
            for (read,outcome) in [(Ok(true),Outcome::Complete),(Err(()),Outcome::Unavailable)] {
                let (mut owner,entered,release,registered,cancelled)=fixture(3,Ok(()),read,Ok(()));
                assert!(owner.start()); entered.recv().unwrap(); release.send(()).unwrap();
                assert_eq!(owner.wait_ack(),Ack { lifetime:3,outcome });
                assert_eq!(owner.take_changed(3),read==Ok(true)); assert!(!owner.take_changed(3));
                owner.stop(); owner.stop(); assert!(!owner.start());
                assert_eq!(registered.load(Ordering::SeqCst),1); assert_eq!(cancelled.load(Ordering::SeqCst),1);
            }
        }
        #[test]
        fn owner_replacement_rejects_old_lifetime_and_stop_filters_queued_result() {
            let (mut owner,entered,release,_,cancelled)=fixture(4,Ok(()),Ok(true),Ok(()));
            assert!(owner.start()); entered.recv().unwrap(); release.send(()).unwrap();
            assert_eq!(owner.wait_ack().outcome,Outcome::Complete);
            assert!(!owner.take_changed(5)); assert_eq!(cancelled.load(Ordering::SeqCst),1);
            let (mut next,entered,release,_,_)=fixture(5,Ok(()),Ok(true),Ok(()));
            assert!(next.start()); entered.recv().unwrap(); release.send(()).unwrap();
            assert_eq!(next.wait_ack().outcome,Outcome::Complete); next.stop();
            assert!(!next.take_changed(5));
        }
        #[test]
        fn owner_registration_and_cancel_failure_never_publish_or_restart() {
            for registration in [Ok(()),Err(())] {
                let (mut owner,entered,release,registered,cancelled)=fixture(6,registration,Ok(true),Err(()));
                assert!(owner.start());
                if registration.is_ok() { entered.recv().unwrap(); release.send(()).unwrap(); }
                assert_eq!(owner.wait_ack(),Ack { lifetime:6,outcome:Outcome::Unavailable });
                assert!(!owner.take_changed(6)); assert!(!owner.start());
                assert_eq!(registered.load(Ordering::SeqCst),1);
                assert_eq!(cancelled.load(Ordering::SeqCst),usize::from(registration.is_ok()));
            }
        }
        // One-shot Windows bridge only. The empty adapter moves to the worker;
        // registration handles and private snapshots are created there and
        // never leave it. No address or interface identity is diagnostic data.
        #[cfg(windows)]
        struct WindowsOnce {
            own: u64,
            notifications: Option<crate::network_notify::Notifications>,
            reads: Arc<AtomicUsize>,
        }
        #[cfg(windows)]
        impl Operations for WindowsOnce {
            fn register(&mut self) -> Result<(), ()> {
                self.notifications=Some(crate::network_notify::Notifications::register(self.own).map_err(|_| ())?);
                Ok(())
            }
            fn read_changed(&mut self) -> Result<bool, ()> {
                let baseline=crate::underlay::Snapshot::read(self.own).map_err(|_| ())?;
                self.reads.fetch_add(1,Ordering::SeqCst);
                let current=crate::underlay::Snapshot::read(self.own).map_err(|_| ())?;
                self.reads.fetch_add(1,Ordering::SeqCst);
                Ok(baseline!=current)
            }
            fn cancel(&mut self) -> Result<(), ()> {
                let result=self.notifications.as_mut().ok_or(())?.shutdown().map_err(|_| ());
                // Notifications retains uncertain callback context on failure.
                // Its Drop owns that safeguard; acknowledgement is not proof
                // that a failed cancellation freed registration context.
                self.notifications.take();
                result
            }
        }
        #[cfg(windows)]
        #[test]
        fn windows_once_reads_then_shutdown_ack_and_zero_identity_rejection() {
            let reads=Arc::new(AtomicUsize::new(0));
            let mut invalid=WindowsOnce { own:0,notifications:None,reads:reads.clone() };
            assert!(invalid.register().is_err());
            assert!(invalid.notifications.is_none());
            assert_eq!(reads.load(Ordering::SeqCst),0);
            // Same nonzero sentinel as the existing notification fixture, not
            // an assertion about adapter ownership. External runner changes
            // may make the snapshots differ, so do not assert their equality.
            let mut owner=Owner::new(91,WindowsOnce { own:1,notifications:None,reads:reads.clone() });
            assert!(owner.start());
            assert_eq!(owner.wait_ack(),Ack { lifetime:91,outcome:Outcome::Complete });
            assert_eq!(reads.load(Ordering::SeqCst),2);
            owner.stop();
            assert!(!owner.take_changed(91));
            assert!(!owner.start());
        }

    }
}
