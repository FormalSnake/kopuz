//! Cookie reader for the isolated YT Music profile: turns the cookies read by
//! [`crate::cookies`] into the `Cookie:` header YT Music expects, requiring
//! the 1P auth cookies to be present.

use std::path::Path;

use config::Browser;

use crate::cookies::Cookie;

pub(crate) const DOMAIN: &str = "youtube.com";

/// The launch flags the isolated sign-in profile is made with, which a later
/// headless read of it has to repeat to decrypt it.
fn profile_args() -> Vec<String> {
    vec!["--password-store=basic".to_string()]
}

#[tracing::instrument(name = "yt.cookies_extract", skip(profile_root), fields(browser = %browser))]
pub async fn extract_from(browser: Browser, profile_root: &Path) -> Result<String, String> {
    let cookies =
        crate::cookies::read_profile_cookies(browser, profile_root, DOMAIN, &profile_args())
            .await?;
    let header = header(&cookies);
    if !has_auth(&header) {
        return Err(format!(
            "no auth cookies found in {} profile — sign in to YouTube Music there first",
            browser.label()
        ));
    }
    Ok(header)
}

/// The cookies as a `Cookie:` header, leaving out any that cannot go in one.
pub(crate) fn header(cookies: &[Cookie]) -> String {
    cookies
        .iter()
        .filter(|c| !c.value.is_empty() && header_safe(&c.name) && header_safe(&c.value))
        .map(|c| format!("{}={}", c.name, c.value))
        .collect::<Vec<_>>()
        .join("; ")
}

fn has_auth(header: &str) -> bool {
    header.split(';').any(|p| {
        let Some((k, _)) = p.trim().split_once('=') else {
            return false;
        };
        k == "SAPISID" || k == "__Secure-3PAPISID"
    })
}

fn header_safe(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| (0x20..0x7f).contains(&b) && b != b';' && b != b',')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookie(name: &str, value: &str) -> Cookie {
        Cookie {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn the_header_leaves_out_what_cannot_be_sent() {
        let cookies = [
            cookie("SAPISID", "a/b"),
            cookie("EMPTY", ""),
            cookie("LIST", "a,b"),
            cookie("SEMI", "a;b"),
            cookie("PREF", "f6=40000000&tz=Europe.Lisbon"),
        ];
        let header = header(&cookies);
        assert_eq!(header, "SAPISID=a/b; PREF=f6=40000000&tz=Europe.Lisbon");
        assert!(has_auth(&header));
        assert!(!has_auth("VISITOR_INFO1_LIVE=v; YSC=y"));
    }
}
