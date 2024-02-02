
use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};

use std::fs;

mod data;

#[get("/issue/{number}")]
async fn issue_number(number: web::Path<u64>) -> impl Responder {
    let html = fs::read_to_string("issues/".to_owned() + &number.to_string() + "/index.html");

    if let Ok(html) = html {
        HttpResponse::Ok().content_type(ContentType::html()).body(html)
    } else {
        HttpResponse::BadRequest().content_type(ContentType::html()).body("400 - BAD REQUEST")
    }
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
    let html = fs::read_to_string("index.html");

    if let Ok(html) = html {
        HttpResponse::Ok().content_type(ContentType::html()).body(html)
    } else {
        HttpResponse::InternalServerError().content_type(ContentType::html()).body("500 - INTERNAL SERVER ERROR")
    }
}

#[get("/issues")]
async fn issues_list() -> impl Responder {
    let mut builder = String::new();

    for entry in fs::read_dir("issues").expect("no existing 'issues' dir") {
        let entry = entry.expect("unknown error reading dir entry");

        if !entry.file_type().expect("unknown error reading dir entry").is_dir() {
            continue;
        }

        let mut entry = entry.path().clone();
        entry.push("index.html");

        builder.push_str("<div class=\"issue-item\">");
        builder.push_str(&fs::read_to_string(entry).expect("unknown io error"));
        builder.push_str("</div>");
    }

    HttpResponse::Ok().content_type(ContentType::html()).body(builder)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(
            || App::new()
                .service(issue_number)
                .service(index_file)
                .service(issues_list)
                .service(index)
        )
        .bind(("0.0.0.0", 9090))?
        .run()
        .await
}
