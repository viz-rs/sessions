use std::time::Duration;

use cookie::SameSite;

/// Cookie's Options
#[derive(Debug)]
pub struct CookieOptions {
    /// Cookie's name, `viz.sid` by defaults
    pub name: String,
    /// Cookie's path
    pub path: String,
    /// Cookie's maximum age, `24H` by defaults
    pub max_age: Duration,
    /// Cookie's secure
    pub secure: Option<bool>,
    /// Cookie's domain
    pub domain: Option<String>,
    /// Cookie's http_only
    pub http_only: Option<bool>,
    /// Cookie's same_site
    pub same_site: Option<SameSite>,
}

impl CookieOptions {
    /// Creates new `CookieOptions`
    pub fn new() -> Self {
        Self {
            domain: None,
            secure: None,
            http_only: None,
            same_site: None,
            path: "/".into(),
            name: "viz.sid".into(),
            max_age: Duration::from_secs(3600 * 24),
        }
    }

    /// Creates new `CookieOptions` with `name`
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Creates new `CookieOptions` with `max_age`
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Creates new `CookieOptions` with `domain`
    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain.replace(domain);
        self
    }

    /// Creates new `CookieOptions` with `path`
    pub fn with_path(mut self, path: String) -> Self {
        self.path = path;
        self
    }

    /// Creates new `CookieOptions` with `secure`
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure.replace(secure);
        self
    }

    /// Creates new `CookieOptions` with `http_only`
    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.http_only.replace(http_only);
        self
    }

    /// Creates new `CookieOptions` with `same_site`
    pub fn with_same_site(mut self, same_site: SameSite) -> Self {
        self.same_site.replace(same_site);
        self
    }
}

impl Default for CookieOptions {
    fn default() -> Self {
        Self::new()
    }
}
