//! Local profile-list cache. No file or network I/O.
use crate::{settings::Settings, Profile};
#[derive(Default)]
pub struct ProfileView {
    rows: Vec<(usize, Vec<String>)>,
    query: Vec<String>,
    raw_query: String,
    filtered: Vec<usize>,
    valid: bool,
    pinned_only: bool,
}
impl ProfileView {
    pub fn invalidate(&mut self) {
        self.valid = false;
    }
    pub fn indices(&mut self, profiles: &[Profile], settings: &Settings, query: &str) -> &[usize] {
        self.indices_filtered(profiles, settings, query, false)
    }
    pub fn indices_filtered(&mut self, profiles: &[Profile], settings: &Settings, query: &str, pinned_only: bool) -> &[usize] {
        let filter_changed = self.pinned_only != pinned_only;
        self.pinned_only = pinned_only;
        let query_changed = self.raw_query != query;
        if query_changed {
            self.raw_query.clear();
            self.raw_query.push_str(query);
            self.query = query.split_whitespace().map(str::to_lowercase).collect();
        }
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
                    // Search only configured display fields. Never index key material.
                    terms.extend(p.config.interface.addresses.iter().map(ToString::to_string));
                    terms.extend(p.config.interface.dns.iter().map(ToString::to_string));
                    terms.extend(p.config.peers.iter().flat_map(|peer| &peer.allowed_ips).map(ToString::to_string));
                    (i, terms)
                })
                .collect();
            self.rows
                .sort_by_key(|(i, _)| settings.profile_rank(&profiles[*i].name));
            self.valid = true;
        }
        if rebuild || query_changed || filter_changed {
            self.filtered = self
                .rows
                .iter()
                .filter(|(i, text)| {
                    (!self.pinned_only || settings.pinned_profiles.contains(&profiles[*i].name))
                        && self.query.iter().all(|word| text.iter().any(|term| term.contains(word)))
                })
                .map(|(i, _)| *i)
                .collect();
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
    fn configured_ip_search_is_literal_key_free_and_refreshes_after_reload() {
        let mut profiles = vec![profile("Office", "primary.invalid:123"), profile("Travel", "travel.invalid:456")];
        profiles[0].config.interface.addresses = vec!["10.44.0.2/32".parse().unwrap(), "fd00::2/128".parse().unwrap()];
        profiles[0].config.interface.dns = vec!["1.1.1.1".parse().unwrap(), "2606:4700:4700::1111".parse().unwrap()];
        profiles[0].config.peers[0].allowed_ips = vec!["10.55.0.0/24".parse().unwrap()];
        profiles[0].config.peers[0].preshared_key = Some([8; 32]);
        let settings = Settings::default();
        let mut view = ProfileView::default();
        for query in ["OFFICE 10.44.0.2/32", "fd00::2", "2606:4700:4700::1111", "10.55.0.0/24", "office 1.1.1.1 10.55.0.0"] {
            assert_eq!(view.indices(&profiles, &settings, query), &[0]);
        }
        // CIDRs are literal text, not an IP-membership, routing or live check.
        for query in ["10.55.0.42", "travel 1.1.1.1", "privatekey", "AAAAAAAAAAAA", "CAgICAgICAgI"] {
            assert!(view.indices(&profiles, &settings, query).is_empty());
        }
        let indexed = format!("{:?}", view.rows);
        for secret in ["AAAAAAAAAAAA", "CAgICAgICAgI", "private_key", "preshared_key"] { assert!(!indexed.contains(secret)); }
        profiles[0].config.interface.addresses = vec!["10.44.0.9/32".parse().unwrap()];
        view.invalidate();
        assert!(view.indices(&profiles, &settings, "10.44.0.2").is_empty());
        assert_eq!(view.indices(&profiles, &settings, "10.44.0.9"), &[0]);
        assert_eq!(view.indices(&profiles, &settings, ""), &[0, 1]);
    }

    #[test]
    fn cached_filter_matches_direct_search_across_query_changes() {
        let mut profiles = vec![
            profile("München Büro", "primary.invalid:123"),
            profile("Travel", "travel.invalid:456"),
        ];
        let mut second = profiles[0].config.peers[0].clone();
        second.endpoint = Some("backup.invalid:789".into());
        profiles[0].config.peers.push(second);
        let settings = Settings::default();
        let mut view = ProfileView::default();
        for query in [
            "",
            "MÜNCHEN",
            " BÜRO ",
            "backup.invalid",
            "primary.invalid",
            "invalid:123 backup",
            "travel",
            "absent",
            "",
            "travel",
        ] {
            let normalized = query.trim().to_lowercase();
            let expected: Vec<_> = profiles
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    normalized.split_whitespace().all(|word| {
                        p.name.to_lowercase().contains(word)
                            || p.config.peers.iter().any(|peer| {
                                peer.endpoint.as_deref().unwrap_or("").to_lowercase().contains(word)
                            })
                    })
                })
                .map(|(i, _)| i)
                .collect();
            assert_eq!(
                view.indices(&profiles, &settings, query),
                expected.as_slice()
            );
        }
    }
    #[test]
    fn words_match_across_fields_without_crossing_profiles() {
        let mut profiles = vec![profile("München Office 24", "primary.invalid:123"), profile("Travel 24", "backup.invalid:789")];
        let mut backup = profiles[0].config.peers[0].clone();
        backup.endpoint = Some("secondary.invalid:456".into());
        profiles[0].config.peers.push(backup);
        let mut settings = Settings::default();
        let mut view = ProfileView::default();
        for query in ["  OFFICE\t24  primary ", "MÜNCHEN 456", "primary secondary", "24 24 office"] {
            assert_eq!(view.indices(&profiles, &settings, query), &[0]);
        }
        for query in ["office backup", "travel primary", "24 missing"] {
            assert!(view.indices(&profiles, &settings, query).is_empty());
        }
        settings.pinned_profiles.insert("Travel 24".into());
        view.invalidate();
        assert!(view.indices_filtered(&profiles, &settings, "office 24", true).is_empty());
        assert_eq!(view.indices_filtered(&profiles, &settings, "24 backup", true), &[1]);
        assert_eq!(view.indices(&profiles, &settings, "\t \n"), &[1, 0]);
    }

    #[test]
    fn pinned_filter_combines_with_search_and_resets_for_quick_picker() {
        let profiles = vec![profile("Office", "office.invalid:123"), profile("Travel", "travel.invalid:456")];
        let mut settings = Settings::default();
        let mut view = ProfileView::default();
        assert!(view.indices_filtered(&profiles, &settings, "", true).is_empty());
        settings.pinned_profiles.insert("Travel".into());
        view.invalidate();
        assert_eq!(view.indices_filtered(&profiles, &settings, "", true), &[1]);
        assert!(view.indices_filtered(&profiles, &settings, "office", true).is_empty());
        assert_eq!(view.indices_filtered(&profiles, &settings, " TRAVEL.INVALID ", true), &[1]);
        assert_eq!(view.indices(&profiles, &settings, ""), &[1, 0]);
        assert_eq!(view.indices_filtered(&profiles, &settings, "", true), &[1]);
        settings.pinned_profiles.clear();
        view.invalidate();
        assert!(view.indices_filtered(&profiles, &settings, "", true).is_empty());
        assert_eq!(view.indices_filtered(&profiles, &settings, "", false), &[0, 1]);
    }

    #[test]
    fn unchanged_query_reuses_terms_and_filtered_storage() {
        let profiles = vec![profile("Office", "office.invalid:123")];
        let settings = Settings::default();
        let mut view = ProfileView::default();
        assert_eq!(view.indices(&profiles, &settings, " OFFICE "), &[0]);
        let query_ptr = view.query.as_ptr();
        let terms_ptr = view.rows.as_ptr();
        let filtered_ptr = view.filtered.as_ptr();
        for _ in 0..20 {
            assert_eq!(view.indices(&profiles, &settings, " OFFICE "), &[0]);
            assert_eq!(view.query.as_ptr(), query_ptr);
            assert_eq!(view.rows.as_ptr(), terms_ptr);
            assert_eq!(view.filtered.as_ptr(), filtered_ptr);
        }
        assert_eq!(view.indices(&profiles, &settings, "office"), &[0]);
        view.invalidate();
        assert_eq!(view.indices(&profiles, &settings, "office"), &[0]);
        assert_eq!(view.indices(&profiles, &settings, ""), &[0]);
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
        assert_eq!(view.indices(&profiles, &settings, "office office.invalid"), &[0]);
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
