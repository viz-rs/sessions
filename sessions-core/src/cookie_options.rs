use std::time::Duration;

#[derive(Debug)]
pub struct CookieOptions {
    /// 24H
    pub max_age: Duration,
    pub name: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<String>,
}

impl CookieOptions {
    pub fn new() -> Self {
        Self {
            name: "viz.sid".to_string(),
            max_age: Duration::from_secs(3600 * 24),
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain.replace(domain);
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path.replace(path);
        self
    }

    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure.replace(secure);
        self
    }

    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.http_only.replace(http_only);
        self
    }

    pub fn with_same_site(mut self, same_site: String) -> Self {
        self.same_site.replace(same_site);
        self
    }
}
