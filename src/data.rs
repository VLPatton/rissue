use std::sync::{Mutex, OnceLock};

use sqlite::Connection;

pub const CREATE_ISSUETAB: &'static str = r#"CREATE TABLE IF NOT EXISTS issuetab (
    issue_id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    submitter TEXT NOT NULL,
    post_date DATETIME NOT NULL
);"#;
pub const CREATE_COMMENTTAB: &'static str = r#"CREATE TABLE IF NOT EXISTS commenttab (
    comment_id INTEGER PRIMARY KEY,
    issue_id INTEGER,
    content TEXT NOT NULL,
    submitter TEXT NOT NULL,
    post_date DATETIME NOT NULL,
    FOREIGN KEY(issue_id) REFERENCES issuetab(issue_id)
);"#;

pub fn get_connection() -> &'static Mutex<Connection> {
    static CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();
    CONNECTION.get_or_init(|| Mutex::new(Connection::open("rissue.db").expect("couldn't open db connection")))
}

pub fn init_db_unchecked() {
    let conn = get_connection().lock().unwrap();
    conn.execute(CREATE_ISSUETAB).unwrap();
    conn.execute(CREATE_COMMENTTAB).unwrap();
}
