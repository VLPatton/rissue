use std::sync::OnceLock;
use sqlx::sqlite::SqlitePool;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub fn get_connection() -> &'static SqlitePool {
    DB_POOL.get().unwrap()
}

pub fn init_connection(pool: SqlitePool) {
    DB_POOL.get_or_init(move || pool);
}

#[derive(Debug, PartialEq, Eq)]
pub struct IssueStatus {
    stat: i32
}

impl IssueStatus {
    pub fn as_str(&self) -> &str {
        STATUS_NAMES[self.stat as usize]
    }

    pub fn as_class_str(&self) -> &str {
        STATUS_CLASS_NAMES[self.stat as usize]
    }

    pub fn get_raw(&self) -> i32 {
        self.stat
    }

    pub fn new(stat: i32) -> Option<IssueStatus> {
        if (stat as usize) < STATUS_NAMES.len() {
            Some(IssueStatus { stat })
        } else {
            None
        }
    }

    pub fn unresolved() -> IssueStatus {
        IssueStatus {
            stat: 0
        }
    }

    pub fn wip() -> IssueStatus {
        IssueStatus {
            stat: 1
        }
    }

    pub fn wont_resolve() -> IssueStatus {
        IssueStatus {
            stat: 2
        }
    }

    pub fn resolved() -> IssueStatus {
        IssueStatus {
            stat: 3
        }
    }
}

pub const STATUS_NAMES: &'static [&'static str] = &[
    "Unresolved",
    "Work In Progress",
    "Won't Resolve",
    "Resolved"
];

pub const STATUS_CLASS_NAMES: &'static [&'static str] = &[
    "issue-unres",
    "issue-wip",
    "issue-wont",
    "issue-res"
];
