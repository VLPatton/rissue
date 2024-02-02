use std::sync::{Mutex, OnceLock};

use sqlite::Connection;

pub const CREATE_ISSUETAB: &'static str = r#"CREATE TABLE IF NOT EXISTS issuetab (
    issue_id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    submitter TEXT NOT NULL
);"#;
pub const CREATE_COMMENTTAB: &'static str = r#"CREATE TABLE IF NOT EXISTS commenttab (
    comment_id INTEGER PRIMARY KEY,
    issue_id INTEGER FOREIGN KEY REFERENCES issuetab(issue_id),
    content TEXT NOT NULL,
    submitter TEXT NOT NULL
);"#;

pub fn get_connection() -> &'static Mutex<Connection> {
    static CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();
    CONNECTION.get_or_init(|| Mutex::new(Connection::open("rissue.db").expect("couldn't open db connection")))
}
