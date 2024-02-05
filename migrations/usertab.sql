CREATE TABLE IF NOT EXISTS usertab (
    username TEXT PRIMARY KEY,
    usermail TEXT NOT NULL,
    passhash TEXT NOT NULL,
    is_admin BOOL NOT NULL
);
