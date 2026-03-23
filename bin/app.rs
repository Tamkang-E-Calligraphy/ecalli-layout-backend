use actix_cors::Cors;
use actix_http::header::HeaderName;
use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    get,
    http::header,
    middleware, web,
};
use ecalli_layout_backend::{
    DB, KEY,
    api::{self, StatusResponse},
};
use fjall::{Database, KeyspaceCreateOptions};
use std::io;

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(StatusResponse {
        code: "000".to_string(),
        message: "Server is running.".to_string(),
    })
}

fn create_server_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = Error,
        InitError = (),
    >,
> {
    let cors = Cors::default()
        .allow_any_origin()
        // .allowed_origin("localhost:3000")
        .allowed_methods(vec!["GET", "POST", "DELETE"])
        .allowed_headers(vec![
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            HeaderName::from_static("x-user-agent"),
        ])
        .supports_credentials();

    App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .service(
            web::scope("/api/v1")
                .service(health_check)
                .service(api::handle_poem_animation_generation),
        )
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let db = DB.get_or_init(|| {
        Database::builder("/opt/tku-website/backend_storage")
            .temporary(true)
            .open()
            .expect("Failed to open the storage!")
    });
    db.keyspace(KEY, KeyspaceCreateOptions::default)
        .expect("Failed to create the default keyspace!");
    HttpServer::new(create_server_app)
        .bind(("127.0.0.1", 18081))?
        .run()
        .await
}
