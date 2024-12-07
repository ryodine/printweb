mod printers;
mod util;

use actix_files::Files;
use actix_web::{middleware::Logger, HttpResponse, Responder};
use env_logger::Env;
use util::{AppConfig, Message};

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(Message {
        message: "Not Found".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{web, App, HttpServer};

    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let server_host = dotenv::var("SERVER_HOST").unwrap();
    let server_port = dotenv::var("SERVER_PORT").unwrap();
    let server_location = server_host + ":" + &server_port;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppConfig {
                ipp_uri: dotenv::var("IPP_URI").unwrap(),
            }))
            .wrap(Logger::default())
            .configure(printers::init_routes)
            .default_service(
                Files::new(".", "static")
                    .index_file("index.html")
                    .default_handler(web::to(not_found)),
            )
    })
    .bind(&server_location)?
    .run()
    .await
}
