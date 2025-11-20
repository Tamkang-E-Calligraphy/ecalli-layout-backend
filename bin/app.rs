use actix_cors::Cors;
use actix_http::header::HeaderName;
use actix_web::body::MessageBody;
use actix_web::dev::{Server, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{App, Error, HttpServer, web};
use ecalli_layout_backend::{AppError, BlobStorageConfig, CalliFont};
use std::io::Cursor;
use zip::ZipArchive;

fn create_server() -> App<
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

    /*
    App::new()
        .service(
            web::scope("/api")
    */

    todo!()
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let storage_config = BlobStorageConfig::from_local_env()?;
    let blob_request = storage_config.get_frame_client(CalliFont::SemiCursive, '觀');
    /* Pure getter method.
    let mut blob_stream = blob_request.get().into_stream();

    let mut result: Vec<u8> = Vec::new();

    // The stream is composed of individual calls to the get blob endpoint
    while let Some(value) = blob_stream.next().await {
        let mut body = value.inspect_err(|e| eprintln!("{e}")).unwrap().data;
        // For each response, we stream the body instead of collecting it all
        // into one large allocation.
        while let Some(value) = body.next().await {
            let value = value.inspect_err(|e| eprintln!("{e}")).unwrap();
            result.extend(&value);
        }
    }
    let reader = Cursor::new(result);
    */
    println!("{}", blob_request.blob_name());
    let reader = Cursor::new(blob_request.get_content().await.unwrap());

    let zipfile = ZipArchive::new(reader).unwrap();

    zipfile.file_names().for_each(|n| println!("{n}"));

    Ok(())
}
