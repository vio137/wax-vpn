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
