//! External URL allowlist (ADR 0010, control 2): the one gate every link Heimdall opens outside
//! itself passes through, whether the user clicked it, a toast carries it, or the updater does.
//! Only `http`, `https` and `mailto` may pass.

use url::Url;

/// A practical safety cap, not a Poirot-derived figure: long enough for any legitimate link, short
/// enough to keep a crafted URL from hiding its real destination in an unreviewable wall of text.
pub const MAX_URL_LENGTH: usize = 2048;

/// True when `raw` may be handed to the OS's URL opener.
///
/// A backslash is rejected outright, before any parsing: the WHATWG URL spec treats `\` as `/` in
/// `http`/`https` for browser compatibility, so a parser can silently turn a UNC-shaped string
/// (`https:\\attacker\path`) into an ordinary host — accepting the raw text at all risks a
/// mismatch between what a person is shown and what a different consumer of the same string opens.
pub fn is_allowed_external_url(raw: &str) -> bool {
    if raw.is_empty() || raw.len() > MAX_URL_LENGTH || raw.contains('\\') {
        return false;
    }
    let Ok(url) = Url::parse(raw) else { return false };
    if !matches!(url.scheme(), "http" | "https" | "mailto") {
        return false;
    }
    // http(s) only: no embedded credentials (`user:pass@host`). `mailto` has no authority, so
    // these are always empty for it and never trigger a false rejection of a normal address.
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_links() {
        for ok in ["https://example.com", "http://example.com/path?q=1#frag", "mailto:someone@example.com", "HTTPS://Example.COM"] {
            assert!(is_allowed_external_url(ok), "{ok}");
        }
    }

    #[test]
    fn rejects_disallowed_schemes() {
        for bad in ["file:///etc/passwd", "javascript:alert(1)", "data:text/html,<script>alert(1)</script>", "vscode://file/x", "ftp://example.com", "tel:+15551234"] {
            assert!(!is_allowed_external_url(bad), "{bad}");
        }
    }

    #[test]
    fn rejects_backslash_smuggled_unc_hosts() {
        for bad in [r"https:\\attacker.example\path", r"https://good.example\@attacker.example", r"http:/\attacker.example"] {
            assert!(!is_allowed_external_url(bad), "{bad}");
        }
    }

    #[test]
    fn rejects_embedded_credentials() {
        assert!(!is_allowed_external_url("https://user:pass@example.com"));
        assert!(!is_allowed_external_url("https://user@example.com"));
    }

    #[test]
    fn accepts_a_mailto_address_containing_an_at_sign_without_flagging_credentials() {
        assert!(is_allowed_external_url("mailto:someone@example.com?subject=Hi"));
    }

    #[test]
    fn rejects_an_over_long_url() {
        let long = format!("https://example.com/{}", "a".repeat(MAX_URL_LENGTH));
        assert!(!is_allowed_external_url(&long));
        let borderline = format!("https://example.com/{}", "a".repeat(MAX_URL_LENGTH - 21));
        assert!(borderline.len() <= MAX_URL_LENGTH);
        assert!(is_allowed_external_url(&borderline));
    }

    #[test]
    fn rejects_empty_and_unparsable_strings() {
        assert!(!is_allowed_external_url(""));
        assert!(!is_allowed_external_url("not a url at all"));
    }
}
