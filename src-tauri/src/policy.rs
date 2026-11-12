use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub enum Destination {
    Embedded,
    Browser,
    Blocked,
}

/// Only top-level navigation is filtered; the official site's CDN resources
/// remain subject to WebView2's normal origin and TLS protections.
pub fn destination(url: &Url) -> Destination {
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Destination::Blocked;
    }
    if url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && (matches!(
            url.host_str(),
            Some(
                "music.youtube.com"
                    | "accounts.google.com"
                    | "accounts.youtube.com"
                    | "consent.google.com"
                    | "consent.youtube.com"
            )
        ) || (url.host_str() == Some("www.youtube.com") && url.path() == "/signin"))
    {
        Destination::Embedded
    } else {
        Destination::Browser
    }
}

pub fn sign_in_rejected(url: &Url) -> bool {
    url.host_str() == Some("accounts.google.com")
        && (url.path().contains("/rejected") || url.path().contains("/denied"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_trust_boundary() {
        for address in [
            "https://music.youtube.com/",
            "https://music.youtube.com/search?q=music",
            "https://accounts.google.com/v3/signin/identifier",
            "https://consent.youtube.com/",
            "https://www.youtube.com/signin?action_handle_signin=true",
        ] {
            assert_eq!(
                destination(&Url::parse(address).unwrap()),
                Destination::Embedded
            );
        }
        for address in [
            "https://music.youtube.com.evil.example/",
            "https://example.com/?next=https://music.youtube.com",
            "http://music.youtube.com/",
            "https://music.youtube.com:8443/",
            "https://www.youtube.com/",
        ] {
            assert_eq!(
                destination(&Url::parse(address).unwrap()),
                Destination::Browser
            );
        }
        for address in [
            "file:///C:/Windows/system.ini",
            "javascript:alert(1)",
            "data:text/html,test",
            "ms-settings:privacy",
            "https://music.youtube.com@evil.example/",
            "https://user:password@music.youtube.com/",
            "http://tauri.localhost/",
        ] {
            let result = destination(&Url::parse(address).unwrap());
            assert_ne!(result, Destination::Embedded);
            if !address.contains("tauri.localhost") {
                assert_eq!(result, Destination::Blocked);
            }
        }
    }
}
