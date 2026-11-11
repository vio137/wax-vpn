//! Local profile-list cache. No file or network I/O.
use crate::{settings::Settings, Profile};
#[derive(Default)]
pub struct ProfileView {
    rows: Vec<(usize, Vec<String>)>,
    query: String,
    filtered: Vec<usize>,
    valid: bool,
}
impl ProfileView {
    pub fn invalidate(&mut self) {
        self.valid = false;
    }
    pub fn indices(&mut self, profiles: &[Profile], settings: &Settings, query: &str) -> &[usize] {
        let query = query.trim().to_lowercase();
        let rebuild = !self.valid;
        if rebuild {
            self.rows = profiles
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let mut terms = vec![p.name.to_lowercase()];
                    terms.extend(
                        p.config
                            .peers
                            .iter()
                            .filter_map(|p| p.endpoint.as_ref())
                            .map(|s| s.to_lowercase()),
                    );
                    (i, terms)
                })
                .collect();
            self.rows
                .sort_by_key(|(i, _)| settings.profile_rank(&profiles[*i].name));
            self.valid = true;
        }
        if rebuild || self.query != query {
            self.filtered = self
                .rows
                .iter()
                .filter(|(_, text)| {
                    query.is_empty() || text.iter().any(|term| term.contains(&query))
                })
                .map(|(i, _)| *i)
                .collect();
            self.query = query;
        }
        &self.filtered
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn profile(name: &str, endpoint: &str) -> Profile {
        let config = wax_core::conf::WireGuardConfig {
            interface: wax_core::conf::InterfaceConfig {
                private_key: [0; 32],
                addresses: vec![],
                dns: vec![],
                mtu: None,
                listen_port: None,
            },
            peers: vec![wax_core::conf::PeerConfig {
                public_key: [0; 32],
                preshared_key: None,
                allowed_ips: vec![],
                endpoint: Some(endpoint.into()),
                persistent_keepalive: None,
            }],
        };
        Profile {
            name: name.into(),
            config,
        }
    }
    #[test]
    fn cache_rebuilds_order_and_matches_names_and_endpoints() {
        let profiles = vec![
            profile("Office", "office.invalid:123"),
            profile("Travel", "travel.invalid:456"),
        ];
        let mut settings = Settings::default();
        let mut view = ProfileView::default();
        assert_eq!(view.indices(&profiles, &settings, ""), &[0, 1]);
        settings.pinned_profiles.insert("Travel".into());
        view.invalidate();
        assert_eq!(view.indices(&profiles, &settings, ""), &[1, 0]);
        assert_eq!(view.indices(&profiles, &settings, " OFFICE.INVALID "), &[0]);
        assert_eq!(view.indices(&profiles, &settings, "travel"), &[1]);
        assert!(view.indices(&profiles, &settings, "absent").is_empty());
        assert!(view
            .indices(&profiles, &settings, "office office.invalid")
            .is_empty());
        let mut renamed = profiles.clone();
        renamed[1].name = "Away".into();
        view.invalidate();
        assert_eq!(view.indices(&renamed, &settings, "away"), &[1]);
        assert!(view.indices(&renamed, &settings, "travel").contains(&1)); // endpoint still matches
        settings.record_connection("Office");
        view.invalidate();
        assert_eq!(view.indices(&renamed, &settings, ""), &[0, 1]);
        view.invalidate();
        assert!(view.indices(&[], &settings, "").is_empty());
    }
}
