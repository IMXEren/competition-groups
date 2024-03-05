use std::env;
use std::sync::OnceLock;

pub static USER: OnceLock<String> = OnceLock::new();
pub static PASS: OnceLock<String> = OnceLock::new();

pub const USER_CSS: &str = "#user_login";
pub const PASS_CSS: &str = "#user_password";
pub const REMEMBER_CSS: &str = "#user_remember_me";
pub const HOMEPAGE: &str = "https://www.competitiongroups.com/";
pub const LOGIN_BTN_CSS: &str = "#root > div > header > button";
pub const SIGN_IN_BTN_CSS: &str = "#new_user > input.btn.btn-primary";

pub fn init_consts() {
    let _ = USER.set(env::var("WCA_USER").expect("user id or email required"));
    let _ = PASS.set(env::var("WCA_PASS").expect("password required"));
}

pub trait Open<T> {
    fn open(&self) -> T;
}

impl Open<String> for OnceLock<String> {
    fn open(&self) -> String {
        self.get().cloned().unwrap()
    }
}
