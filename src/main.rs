use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::io::{Error, ErrorKind};
use serde::Serialize;
use log::{info};
use usvg::{fontdb, TreeParsing, TreeTextToPath};
use svg2pdf::Options;
use lazy_static::lazy_static;
use chrono::Utc;
use std::fs;
use usvg::fontdb::Database;
use vl_convert_pdf::svg_to_pdf;

#[derive(Serialize)]
struct MyObj {
    name: String,
}

lazy_static! {
    static ref DB: fontdb::Database = {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        info!("loaded fonts again");
        db
    };
}

#[get("/b/{name}")]
async fn make_pdf(name: web::Path<String>) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let folder = "C:\\Users\\yesid\\workspace\\svg-examples\\";
    let path = format!("{folder}\\{name}.svg");
    let svg = std::fs::read_to_string(path)?;
    let options = usvg::Options::default();
    let mut start_time = Utc::now().time();
    let mut tree = usvg::Tree::from_str(&svg, &options)?;
    let mut end_time = Utc::now().time();
    let mut diff = (end_time - start_time).num_milliseconds();
    info!("svg string to tree {diff} ms");

    start_time = Utc::now().time();
    tree.convert_text(&DB);
    end_time = Utc::now().time();
    diff = (end_time - start_time).num_milliseconds();
    info!("usvg convert text {diff} ms");

    start_time = Utc::now().time();
    let pdf = svg2pdf::convert_tree(&tree, Options::default());
    end_time = Utc::now().time();
    diff = (end_time - start_time).num_milliseconds();
    info!("svg2pdf convert {diff} ms");

    start_time = Utc::now().time();
    std::fs::write(format!("{folder}\\{name}.pdf"), pdf)?;
    end_time = Utc::now().time();
    diff = (end_time - start_time).num_milliseconds();
    info!("write pdf to disk {diff} ms");
    let obj = MyObj {
        name: name.to_string(),
    };
    Ok(web::Json(obj))
}

#[get("/a/{name}")]
async fn hello(name: web::Path<String>) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let folder = "C:\\Users\\yesid\\workspace\\svg-examples\\";
    let path = format!("{folder}\\{name}.svg");
    let svg = std::fs::read_to_string(path)?;
    info!("file was read from disk {}", svg.len());
// This can only fail if the SVG is malformed. This one is not.
    let pdf = svg2pdf::convert_str(&svg, svg2pdf::Options::default())
        .map_err(|_err| Error::new(ErrorKind::Other, "could not do post req"))?;
    info!("file has been converted to pdf {}", pdf.len());

// ... and now you have a Vec<u8> which you could write to a file or
// transmit over the network!
    std::fs::write(format!("{folder}\\{name}.pdf"), pdf)?;
    info!("file has been written to disk");
    let obj = MyObj {
        name: name.to_string(),
    };
    Ok(web::Json(obj))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(make_pdf)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}