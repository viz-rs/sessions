use std::time::Duration;

#[derive(Debug)]
pub struct CookieOptions {
    name: String,
    domain: Option<String>,
    path: Option<String>,
    secure: Option<bool>,
    max_age: Option<Duration>,
    http_only: Option<bool>,
    same_site: Option<String>,
}

impl CookieOptions {
    pub fn new() -> Self {
        Self {
            name: "viz.sid".to_string(),
            domain: None,
            path: None,
            secure: None,
            max_age: None,
            http_only: None,
            same_site: None,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn domain(mut self, domain: String) -> Self {
        self.domain.replace(domain);
        self
    }

    pub fn path(mut self, path: String) -> Self {
        self.path.replace(path);
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure.replace(secure);
        self
    }

    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.max_age.replace(max_age);
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only.replace(http_only);
        self
    }

    pub fn same_site(mut self, same_site: String) -> Self {
        self.same_site.replace(same_site);
        self
    }
}
