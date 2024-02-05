
use actix_web::{post, put};
use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::fs;
use jsonwebtoken::Validation;

mod auth;
mod data;

#[derive(Deserialize)]
struct CreateIssueRequest {
    title: String,
    comment: String,
    jwt: String
}

impl CreateIssueRequest {
    pub fn validate(self) -> Option<CreateIssueRequest> {
        if let Ok(_) = jsonwebtoken::decode::<auth::AuthPayload>(&self.jwt, auth::decode_key(), &Validation::default()) {
            Some(CreateIssueRequest {
                title: self.title,
                comment: self.comment,
                jwt: self.jwt
            })
        } else {
            None
        }
    }
}

#[derive(Deserialize)]
struct CreateCommentRequest {
    issue_id: i32,
    comment: String,
    jwt: String
}

impl CreateCommentRequest {
    pub fn validate(self) -> Option<CreateCommentRequest> {
        if let Ok(_) = jsonwebtoken::decode::<auth::AuthPayload>(&self.jwt, auth::decode_key(), &Validation::default()) {
            Some(CreateCommentRequest {
                issue_id: self.issue_id,
                comment: self.comment,
                jwt: self.jwt
            })
        } else {
            None
        }
    }
}

#[derive(Deserialize)]
struct StatusUpdate {
    pub stat: i32,
    pub issue_id: i32,
    pub jwt: String
}

#[derive(Deserialize)]
struct AuthRequest {
    pub user: String,
    pub passwd: String
}

#[derive(Deserialize)]
struct AuthNewUser {
    pub user: String,
    pub email: String,
    pub passwd: String
}

#[post("/auth/new_user")]
async fn new_user(web::Form(body): web::Form<AuthNewUser>) -> impl Responder {
    let conn = data::get_connection();

    let passwd = body.passwd.replace("-", "=");

    let passhash = auth::do_hash(passwd.as_bytes());

    sqlx::query!("INSERT INTO usertab (username, usermail, passhash, is_admin) VALUES (?, ?, ?, 0)", body.user, body.email, passhash).execute(conn).await.unwrap();

    let payload = auth::AuthPayload {
        sub: body.user,
        exp: (std::time::SystemTime::now() + std::time::Duration::from_secs(30 * 24 * 60 * 60)).duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize
    };

    let payload = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &payload, auth::encode_key()).unwrap();

    HttpResponse::Ok().body(payload)
}

#[post("/auth/new_tok")]
async fn authenticate(web::Form(body): web::Form<AuthRequest>) -> impl Responder {
    let conn = data::get_connection();

    let user_data = sqlx::query!("SELECT * FROM usertab WHERE username = ?", body.user).fetch_one(conn).await.unwrap();

    let passwd = body.passwd.replace("-", "=");

    if auth::do_hash(passwd.as_bytes()) == user_data.passhash {
        let payload = auth::AuthPayload {
            sub: body.user,
            exp: (std::time::SystemTime::now() + std::time::Duration::from_secs(30 * 24 * 60 * 60)).duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize
        };

        let payload = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &payload, auth::encode_key()).unwrap();

        return HttpResponse::Ok().body(payload);
    }

    HttpResponse::Forbidden().body("403 - Forbidden")
}

#[put("/status")]
async fn put_status(web::Form(body): web::Form<StatusUpdate>) -> impl Responder {
    if let None = data::IssueStatus::new(body.stat) {
        return HttpResponse::BadRequest().body("");
    }

    let status = data::IssueStatus::new(body.stat).unwrap().get_raw();

    let conn = data::get_connection();

    let jwt = jsonwebtoken::decode::<auth::AuthPayload>(&body.jwt, auth::decode_key(), &Validation::default());

    if let Err(_) = jwt {
        return HttpResponse::Forbidden().body("403 - Forbidden");
    }

    sqlx::query!("UPDATE issuetab SET status = ? WHERE issue_id = ?", status, body.issue_id).execute(conn).await.unwrap();

    HttpResponse::Ok().body("")
}

#[get("/new/comment")]
async fn new_comment_prompt() -> impl Responder {
    let html = fs::read_to_string("comment_box.html").unwrap();

    HttpResponse::Ok().body(html)
}

#[post("/new/comment")]
async fn new_comment(web::Form(body): web::Form<CreateCommentRequest>) -> impl Responder {
    let conn = data::get_connection();

    let body = body.validate();

    if let None = body {
        return HttpResponse::Forbidden().body("");
    }

    let body = body.unwrap();

    let id = body.issue_id;

    // body.validate() checks this for us, so unwrap it
    let user_id = jsonwebtoken::decode::<auth::AuthPayload>(&body.jwt, auth::decode_key(), &Validation::default()).unwrap();

    let cid: i32 = sqlx::query!("SELECT COUNT(*) as cid FROM commenttab").fetch_one(conn).await.unwrap().cid;

    sqlx::query!(
        "INSERT INTO commenttab (comment_id, issue_id, content, submitter, post_date) VALUES (?, ?, ?, ?, DATETIME('now'))",
        cid,
        id,
        body.comment, 
        user_id.claims.sub
    ).execute(conn).await.unwrap();

    let mut html = String::new();

    let data = sqlx::query!("SELECT * FROM commenttab WHERE comment_id=?", cid).fetch_all(conn).await.unwrap();

    for comment in data {
        html.push_str(&std::format!("<div class=\"issue-comment\" id=\"id-{}\">", comment.issue_id));
        html.push_str(&std::format!("{}<br>", comment.content));

        html.push_str(&std::format!("(Submitted by: {} at {})<br><br>", comment.submitter, comment.post_date.and_utc()));
        html.push_str("</div>\n");
    }


    let redo_comment = fs::read_to_string("comment_box.html").unwrap();

    html.push_str(&redo_comment);

    HttpResponse::Ok().body(html)
}

#[post("/new/issue")]
async fn new_issue(web::Form(body): web::Form<CreateIssueRequest>) -> impl Responder {
    let conn = data::get_connection();

    let body = body.validate();

    if let None = body {
        return HttpResponse::Forbidden().body("403 - Forbidden");
    }

    let body = body.unwrap();

    let id: i32 = sqlx::query!("SELECT COUNT(*) as id FROM issuetab").fetch_one(conn).await.unwrap().id;

    let username = jsonwebtoken::decode::<auth::AuthPayload>(&body.jwt, auth::decode_key(), &Validation::default());

    if let Err(_) = username {
        return HttpResponse::Forbidden().body("403 - Forbidden");
    }

    let username = username.unwrap().claims.sub;

    sqlx::query!("INSERT INTO issuetab (issue_id, title, submitter, status, post_date) VALUES (?, ?, ?, 0, DATETIME('now'))", id, body.title, username).execute(conn).await.unwrap();

    let cid: i32 = sqlx::query!("SELECT COUNT(*) as cid FROM commenttab").fetch_one(conn).await.unwrap().cid;

    sqlx::query!("INSERT INTO commenttab (comment_id, issue_id, content, submitter, post_date) VALUES (?, ?, ?, ?, DATETIME('now'))", cid, id, body.comment, username).execute(conn).await.unwrap();

    HttpResponse::Ok().body("<p>Issue submitted!</p>")
}

#[get("/issue/{number}")]
async fn issue_number(number: web::Path<i32>) -> impl Responder {
    let conn = data::get_connection();

    let mut html = String::new();
    html.push_str("<div><button hx-get=\"/issues\" hx-target=\"#latest-div\" hx-swap=\"innerHTML\" class=\"create-button\">Go home</button></div>\n<div>");

    let number = number.into_inner();
    let issue = sqlx::query!("SELECT * FROM issuetab WHERE issue_id = ?", number).fetch_one(conn).await.unwrap();

    html.push_str(&std::format!("<h1 class=\"issue-title\"><span id=\"issue-id\">{}</span> - {}</h1>", issue.issue_id, issue.title));
    html.push_str(&std::format!("<p class=\"issue-datetime\">date time: {}</p>", issue.post_date.and_utc()));

    let status = match data::IssueStatus::new(issue.status as i32) {
        Some(s) => {
            s
        }
        None => {
            data::IssueStatus::unresolved()
        }
    };
    html.push_str(&std::format!("<p class=\"issue-status\">Status: {}</p>", status.as_str()));

    html.push_str("</div><br>");

    let comments = sqlx::query!("SELECT * FROM commenttab WHERE issue_id = ?", issue.issue_id).fetch_all(conn).await.unwrap();

    for c in comments {
        html.push_str(&std::format!("<div class=\"issue-comment\" id=\"id-{}\">", c.comment_id));
        html.push_str(&std::format!("{}<br>", c.content));

        html.push_str(&std::format!("(Submitted by: {} at {})</div><br>\n", c.submitter, c.post_date.and_utc()));
    }

    html.push_str(&std::format!("<button hx-get=\"/new/comment\" hx-swap=\"outerHTML\" class=\"create-button\">Post new comment</button>\n"));

    HttpResponse::Ok().content_type(ContentType::html()).body(html)
}

#[get("/assets/{filename}")]
async fn index_file(filename: web::Path<String>) -> impl Responder {
    let content = fs::read_to_string(filename.to_string());

    if let Ok(content) = content {
        HttpResponse::Ok().content_type(if filename.to_string().ends_with(".js") { mime::TEXT_JAVASCRIPT } else { mime::TEXT_CSS }).body(content)
    } else {
        HttpResponse::InternalServerError().content_type(ContentType::html()).body("500 - INTERNAL SERVER ERROR")
    }
}

#[get("/")]
async fn index() -> impl Responder {
    let html = fs::read_to_string("index.html").unwrap();

    let mut builder = String::new();

    let conn = data::get_connection();

    let stat_res = data::IssueStatus::resolved().get_raw();

    let issue_list = sqlx::query!("SELECT * FROM issuetab WHERE post_date > DATETIME('now', '-1 month') AND status <> ?", stat_res).fetch_all(conn).await.unwrap();

    for issue in issue_list {
        let status = match data::IssueStatus::new(issue.status as i32) {
            Some(s) => s,
            None => data::IssueStatus::unresolved()
        };

        builder = builder.clone() + &std::format!(
            "<div class=\"issue-title {}\" hx-get=\"/issue/{}\" hx-swap=\"innerHTML\" hx-target=\"#latest-div\">{} - {}</div>\n",
            status.as_class_str(),
            issue.issue_id,
            issue.issue_id,
            issue.title
        );
    }

    println!("{builder}");

    let html = html.replace("{}", &builder);

    HttpResponse::Ok().content_type(ContentType::html()).body(html)
}

#[get("/issues")]
async fn issues() -> impl Responder {
    let mut builder = String::new();

    let conn = data::get_connection();

    let stat_res = data::IssueStatus::resolved().get_raw();

    let issue_list = sqlx::query!("SELECT * FROM issuetab WHERE post_date > DATETIME('now', '-1 month') AND status <> ?", stat_res).fetch_all(conn).await.unwrap();

    for issue in issue_list {
        let status = match data::IssueStatus::new(issue.status as i32) {
            Some(s) => s,
            None => data::IssueStatus::unresolved()
        };

        builder = builder.clone() + &std::format!(
            "<div class=\"issue-title {}\" hx-get=\"/issue/{}\" hx-swap=\"innerHTML\" hx-target=\"#latest-div\">{} - {}</div>\n",
            status.as_class_str(),
            issue.issue_id,
            issue.issue_id,
            issue.title
        );
    }

    println!("{builder}");

    HttpResponse::Ok().content_type(ContentType::html()).body(builder)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:rissue.db")
        .await.unwrap();

    data::init_connection(pool);

    HttpServer::new(
            || App::new()
                .service(issue_number)
                .service(issues)
                .service(index_file)
                .service(new_comment_prompt)
                .service(new_comment)
                .service(new_issue)
                .service(put_status)
                .service(authenticate)
                .service(new_user)
                .service(index)
        )
        .bind(("0.0.0.0", 9090))?
        .run()
        .await
}
