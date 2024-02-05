CREATE TABLE IF NOT EXISTS issuetab (
    issue_id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    submitter TEXT NOT NULL,
    status INTEGER NOT NULL,
    post_date DATETIME NOT NULL,
    FOREIGN KEY(submitter) REFERENCES usertab(username)
);
