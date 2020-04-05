use cookie::Cookie;

#[derive(Clone, Debug)]
pub struct Options {
    /// prefix in store
    prefix: String,
    /// cookie's Name
    name: String,
    path: String,
    domian: String,
    max_age: usize,
    secure: bool,
    http_only: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            prefix: "".to_owned(),
            name: "session.id".to_owned(),
            path: "/".to_owned(),
            domain: "".to_owned(),
            max_age: 60 * 60 * 24 * 7 * 2, // Two weeks
            secure: true,
            http_only: true,
        }
    }
}

impl Options {
    #[cfg(feature = "cookie")]
    pub fn cookie() -> Cookie<'_> {}
}
