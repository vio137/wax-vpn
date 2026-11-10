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
}
