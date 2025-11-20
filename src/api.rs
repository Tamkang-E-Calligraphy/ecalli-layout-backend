use crate::feature::json::*;
use actix_web::{
    HttpResponse, Responder, delete,
    web::{self, ReqData}, post
};
use tracing::error;

#[post("/animation")]
pub async fn get_static_layout(
    body: web::Json<AnimationRequest>,
) -> impl Responder {

    todo!();
}
