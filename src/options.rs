#[derive(Debug)]
pub struct Options {
    path: String,
    domain: String,
    // max_age=0 means no max_age attribute specified and the cookie will be
    // deleted after the browser session ends.
    // max_age<0 means delete cookie immediately.
    // max_age>0 means max_age attribute present and given in seconds.
    max_age: i32,
    secure: bool,
    http_only: bool,
}
