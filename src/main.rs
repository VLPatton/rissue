
use actix_web::post;
use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::fs;

mod data;

#[derive(Deserialize)]
struct CreateIssueRequest {
    title: String,
    comment: String,
    email: String
}

impl CreateIssueRequest {
    pub fn sanitize(&self) -> CreateIssueRequest {
        let title = self.title
            .replace("%", "%%")
            .replace("'", "%quote%")
            .replace("@", "%at%");

        let comment = self.comment
            .replace("%", "%%")
            .replace("'", "%quote%")
            .replace("@", "%at%");

        let email = self.email
            .replace("%", "%%")
            .replace("'", "%quote%")
            .replace("@", "%at%");

        CreateIssueRequest {
            title,
            comment,
            email
        }
    }
}

#[derive(Deserialize)]
struct CreateCommentRequest {
    issue_id: i32,
    comment: String,
    email: String
}

impl CreateCommentRequest {
    pub fn sanitize(&self) -> CreateCommentRequest {
        let comment = self.comment
            .replace("%", "%%")
            .replace("'", "%quote%")
            .replace("@", "%at%");

        let email = self.email
            .replace("%", "%%")
            .replace("'", "%quote%")
            .replace("@", "%at%");

        CreateCommentRequest {
            issue_id: self.issue_id,
            comment,
            email
        }
    }
}

fn unsanitize(s: &str) -> String {
    s.replace("%%", "%")
        .replace("%quote%", "'")
        .replace("%at%", "@")
}

#[get("/new/comment")]
async fn new_comment_prompt() -> impl Responder {
    let html = fs::read_to_string("comment_box.html").unwrap();

    HttpResponse::Ok().body(html)
}

#[post("/new/comment")]
async fn new_comment(web::Form(body): web::Form<CreateCommentRequest>) -> impl Responder {
    let conn = data::get_connection().lock().unwrap();

    let body = body.sanitize();

    let id = body.issue_id;

    let mut cid = 0i32;

    conn.iterate("SELECT COUNT(*) FROM commenttab;", |pairs| {
        for &(_, value) in pairs.iter() {
            cid = value.unwrap().parse::<i32>().unwrap_or(0);
        }

        println!("cid = {cid}");
        true
    }).unwrap();

    conn.execute(std::format!(
        "INSERT INTO commenttab (comment_id, issue_id, content, submitter, post_date) VALUES ({cid}, {id}, '{}', '{}', DATETIME('now'));",
        body.comment,
        body.email
    )).unwrap();

    HttpResponse::Ok().body("<p>Comment submitted!</p>")
}

#[post("/new/issue")]
async fn new_issue(web::Form(body): web::Form<CreateIssueRequest>) -> impl Responder {
    let conn = data::get_connection().lock().unwrap();

    let body = body.sanitize();

    let mut id = 0i32;

    conn.iterate("SELECT COUNT(*) FROM issuetab;", |pairs| {
        for &(_, value) in pairs.iter() {
            id = value.unwrap().parse::<i32>().unwrap_or(0);
        }

        println!("id = {id}");
        true
    }).unwrap();

    let req = std::format!("INSERT INTO issuetab (issue_id, title, submitter, post_date) VALUES ('{}', '{}', '{}', DATETIME('now'));", id, body.title, body.email);

    println!("req = {req}");

    conn.execute(req).unwrap();

    let mut cid = 0i32;

    conn.iterate("SELECT COUNT(*) FROM commenttab;", |pairs| {
        for &(_, value) in pairs.iter() {
            cid = value.unwrap().parse::<i32>().unwrap_or(0);
        }

        println!("cid = {cid}");
        true
    }).unwrap();

    conn.execute(std::format!(
        "INSERT INTO commenttab (comment_id, issue_id, content, submitter, post_date) VALUES ({cid}, {id}, '{}', '{}', DATETIME('now'));",
        body.comment,
        body.email
    )).unwrap();

    HttpResponse::Ok().body("<p>Issue submitted!</p>")
}

#[get("/issue/{number}")]
async fn issue_number(number: web::Path<u64>) -> impl Responder {
    let conn = data::get_connection().lock().unwrap();

    let mut html = String::from("<div>");

    conn.iterate(std::format!("SELECT * FROM issuetab WHERE issue_id={number};"), |pairs| {
        for &(name, value) in pairs.iter() {
            match name {
                "title" => {
                    let value = unsanitize(value.unwrap());

                    html.push_str(&std::format!("<h1 class=\"issue-title\"><span id=\"issue-id\">{number}</span> - {value}</h1>"));
                }
                "post_date" => {
                    let value = unsanitize(value.unwrap());

                    html.push_str(&std::format!("<p class=\"issue-datetime\">date time: {value}</p>"));
                }
                _ => {eprintln!("encountered unknown key: {name}")}
            }
        }

        true
    }).unwrap();

    html.push_str("</div><br>");

    conn.iterate(std::format!("SELECT * FROM commenttab WHERE issue_id={number};"), |pairs| {
        for &(name, value) in pairs.iter() {
            match name {
                "comment_id" => {
                    let value = unsanitize(value.unwrap());

                    html.push_str(&std::format!("<div class=\"issue-comment\" id=\"id-{value}\">"));
                }
                "content" => {
                    let value = unsanitize(value.unwrap());

                    html.push_str(&std::format!("{value}<br>"));
                }
                "submitter" => {
                    let value = unsanitize(value.unwrap());

                    html.push_str(&std::format!("(Submitted by: {value} "));
                }
                "post_date" => {
                    let value = unsanitize(value.unwrap());

                    html.push_str(&std::format!("at {value})<br><br>"))
                }
                _ => {}
            }
        }

        html.push_str("</div>");

        true
    }).unwrap();

    html.push_str(&std::format!("<button hx-get=\"/new/comment\" hx-swap=\"outerHTML\">Post new comment</button>"));

    HttpResponse::Ok().content_type(ContentType::html()).body(html)
}

#[get("/assets/{filename}")]
async fn index_file(filename: web::Path<String>) -> impl Responder {
    let content = fs::read_to_string(filename.to_string());

    if let Ok(content) = content {
        HttpResponse::Ok().content_type(if filename.to_string().ends_with(".js") { mime::TEXT_JAVASCRIPT } else { mime::TEXT_PLAIN_UTF_8 }).body(content)
    } else {
        HttpResponse::InternalServerError().content_type(ContentType::html()).body("500 - INTERNAL SERVER ERROR")
    }
}

#[get("/")]
async fn index() -> impl Responder {
    let html = fs::read_to_string("index.html").unwrap();

    let mut builder = String::new();

    let conn = data::get_connection().lock().unwrap();
    conn.iterate("SELECT * FROM issuetab WHERE post_date>DATETIME('now', '-1 month');", |pairs| {
        let mut id = "";
        let mut title = "";
        for &(name, value) in pairs.iter() {
            if name == "issue_id" {
                id = value.unwrap_or("");
            }
            if name == "title" {
                title = value.unwrap_or("");
            }
        }
        builder = builder.clone() + &std::format!("<div class=\"issue-title\" hx-get=\"/issue/{id}\" hx-swap=\"innerHTML\" hx-target=\"#latest-div\">{id} - {title}</div>\n");
        true
    }).unwrap();

    println!("{builder}");

    let html = html.replace("{}", &builder);

    HttpResponse::Ok().content_type(ContentType::html()).body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    data::init_db_unchecked();

    HttpServer::new(
            || App::new()
                .service(issue_number)
                .service(index_file)
                .service(new_comment_prompt)
                .service(new_comment)
                .service(new_issue)
                .service(index)
        )
        .bind(("0.0.0.0", 9090))?
        .run()
        .await
}
