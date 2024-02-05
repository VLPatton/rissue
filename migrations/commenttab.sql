CREATE TABLE IF NOT EXISTS commenttab (
    comment_id INTEGER PRIMARY KEY,
    issue_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    submitter TEXT NOT NULL,
    post_date DATETIME NOT NULL,
    FOREIGN KEY(issue_id) REFERENCES issuetab(issue_id),
    FOREIGN KEY(submitter) REFERENCES usertab(username)
);
