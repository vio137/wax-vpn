//! Pure retry policy. No OS, profile, network or timer side effects.
//! Times are elapsed monotonic durations supplied by the caller.
use std::time::Duration;

const STABLE_SESSION: Duration = Duration::from_secs(60);
const DELAYS: [Duration; 3] = [Duration::from_secs(2), Duration::from_secs(4), Duration::from_secs(8)];

#[derive(Debug, Default)]
pub struct Recovery {
    generation: u64,
    attempts: usize,
    due: Option<Duration>,
    connected_at: Option<Duration>,
}

impl Recovery {
    /// A new explicit user intent invalidates all outstanding worker/timer tokens.
    pub fn new_intent(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.attempts = 0;
        self.due = None;
        self.connected_at = None;
        self.generation
    }

    pub fn generation(&self) -> u64 { self.generation }
    pub fn attempts(&self) -> usize { self.attempts }
    pub fn is_pending(&self) -> bool { self.due.is_some() }

    /// Disabling recovery cancels both timer wait and a running automatic
    /// attempt. It does not disconnect an established or manual connection.
    pub fn should_cancel(&self, enabled: bool) -> bool {
        !enabled && self.attempts > 0 && self.connected_at.is_none()
    }

    /// Adapter creation does not refill the retry budget.
    pub fn connected(&mut self, generation: u64, now: Duration) {
        if generation == self.generation {
            self.due = None;
            self.connected_at = Some(now);
        }
    }

    /// Only a stable owned session refills the budget. A silent handshake does
    /// not imply loss, and is not an input to this policy.
    pub fn schedule(&mut self, generation: u64, now: Duration, owned: bool, enabled: bool) -> bool {
        if generation != self.generation || !owned || !enabled || self.due.is_some() { return false; }
        if self.connected_at.take().is_some_and(|start| now.saturating_sub(start) >= STABLE_SESSION) {
            self.attempts = 0;
        }
        let Some(delay) = DELAYS.get(self.attempts) else { return false; };
        self.attempts += 1;
        self.due = Some(now.saturating_add(*delay));
        true
    }

    /// Returns one current-generation attempt at most once when due.
    pub fn take_due(&mut self, now: Duration) -> Option<u64> {
        if self.due.is_some_and(|due| now >= due) {
            self.due = None;
            Some(self.generation)
        } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sec(n: u64) -> Duration { Duration::from_secs(n) }
    #[test]
    fn delays_are_bounded_and_not_immediate() {
        let mut r = Recovery::default(); let g = r.new_intent();
        for (now, due) in [(0,2), (2,6), (6,14)] {
            assert!(r.schedule(g, sec(now), true, true));
            assert!(r.is_pending());
            assert!(!r.schedule(g, sec(now), true, true));
            assert_eq!(r.take_due(sec(due-1)), None);
            assert_eq!(r.take_due(sec(due)), Some(g));
            assert_eq!(r.take_due(sec(due)), None);
        }
        assert_eq!(r.attempts(),3);
        assert!(!r.schedule(g, sec(15), true, true));
    }
    #[test]
    fn flapping_does_not_refill_budget_but_stable_session_does() {
        let mut r = Recovery::default(); let g = r.new_intent();
        for n in 0..3 {
            r.connected(g, sec(n*10));
            assert!(r.schedule(g, sec(n*10+1), true, true));
            r.take_due(sec(n*10+9));
        }
        r.connected(g, sec(30));
        assert!(!r.schedule(g, sec(31), true, true));
        r.connected(g, sec(40));
        assert!(r.schedule(g, sec(100), true, true));
        assert_eq!(r.attempts(),1);
    }
    #[test]
    fn explicit_disconnect_or_new_intent_cancels_old_work() {
        let mut r = Recovery::default(); let old = r.new_intent();
        assert!(r.schedule(old,sec(0),true,true));
        let new = r.new_intent();
        assert_ne!(old,new);
        assert_eq!(r.generation(),new);
        assert_eq!(r.take_due(sec(100)),None);
        assert!(!r.schedule(old,sec(100),true,true));
        r.connected(old,sec(0));
        assert!(r.connected_at.is_none());
    }
    #[test]
    fn disabling_cancels_waiting_and_inflight_but_not_manual_or_established() {
        let mut r=Recovery::default(); let g=r.new_intent();
        assert!(!r.should_cancel(false));
        assert!(r.schedule(g,sec(0),true,true));
        assert!(r.should_cancel(false));
        assert!(!r.should_cancel(true));
        assert_eq!(r.take_due(sec(2)),Some(g));
        assert!(r.should_cancel(false));
        r.connected(g,sec(3));
        assert!(!r.should_cancel(false));
    }
    #[test]
    fn external_tunnel_and_disabled_recovery_never_schedule() {
        let mut r = Recovery::default(); let g = r.new_intent();
        assert!(!r.schedule(g,sec(0),false,true));
        assert!(!r.schedule(g,sec(0),true,false));
        assert_eq!(r.attempts(),0);
    }
}
