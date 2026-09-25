//! Which pages may talk to the signer, checked during the WebSocket handshake.

/// The GST portal's registrable domain.
const GST_DOMAIN: &str = "gst.gov.in";

/// Suffix every GST portal subdomain ends with.
const GST_SUBDOMAIN_SUFFIX: &str = ".gst.gov.in";

/// Scheme every allowed origin must use.
const HTTPS_PREFIX: &str = "https://";

/// Decides which page origins may request signatures.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OriginPolicy {
    /// Additional exact origins, normalised to lowercase without a trailing slash.
    extra: Vec<String>,
}

impl OriginPolicy {
    /// Builds a policy that also allows each exact origin in `extra`.
    pub fn new(extra: &[String]) -> Self {
        let extra = extra
            .iter()
            .map(|origin| normalise(origin))
            .filter(|origin| !origin.is_empty())
            .collect();
        Self { extra }
    }

    /// True for `https://gst.gov.in`, any `https://<sub>.gst.gov.in`, and the
    /// extra origins. Browsers set `Origin` themselves; pages cannot forge it.
    pub fn allows(&self, origin: &str) -> bool {
        let origin = normalise(origin);
        let gst = origin.strip_prefix(HTTPS_PREFIX).is_some_and(|host| {
            is_hostname(host) && (host == GST_DOMAIN || host.ends_with(GST_SUBDOMAIN_SUFFIX))
        });
        gst || self.extra.contains(&origin)
    }

    /// True when the `Host` header names this signer: `127.0.0.1:<port>` or
    /// `localhost:<port>`. Anything else points at DNS rebinding.
    pub fn allows_host(host: &str, port: u16) -> bool {
        let host = host.trim().to_ascii_lowercase();
        host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
    }
}

/// Lowercases, trims and drops a trailing slash.
fn normalise(origin: &str) -> String {
    origin.trim().trim_end_matches('/').to_ascii_lowercase()
}

/// True for plain DNS names: letters, digits, hyphens, dots, no empty labels.
fn is_hostname(host: &str) -> bool {
    !host.is_empty()
        && host.split('.').all(|label| {
            !label.is_empty()
                && label
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
}

#[cfg(test)]
mod tests {
    use super::OriginPolicy;

    /// GST origins pass; lookalikes, plain http and ports do not.
    #[test]
    fn allows_only_gst_origins() {
        let policy = OriginPolicy::default();
        for allowed in [
            "https://services.gst.gov.in",
            "https://return.gst.gov.in",
            "https://gst.gov.in",
            "HTTPS://Reg.GST.gov.in/",
        ] {
            assert!(policy.allows(allowed), "{allowed}");
        }
        for denied in [
            "http://services.gst.gov.in",
            "https://gst.gov.in.evil.example",
            "https://evilgst.gov.in",
            "https://services.gst.gov.in:8443",
            "https://a..gst.gov.in",
            "null",
            "",
        ] {
            assert!(!policy.allows(denied), "{denied}");
        }
    }

    /// Extra origins are matched exactly after normalisation.
    #[test]
    fn allows_configured_extras() {
        let policy = OriginPolicy::new(&["https://Example.test/".to_owned()]);
        assert!(policy.allows("https://example.test"));
        assert!(!policy.allows("https://sub.example.test"));
    }

    /// Only loopback hosts with the right port pass.
    #[test]
    fn checks_host_header() {
        assert!(OriginPolicy::allows_host("127.0.0.1:1585", 1585));
        assert!(OriginPolicy::allows_host("LOCALHOST:1585", 1585));
        assert!(!OriginPolicy::allows_host("127.0.0.1:2095", 1585));
        assert!(!OriginPolicy::allows_host("attacker.example:1585", 1585));
    }
}
