use actix_web::web;
use crate::server::handlers::{hello};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/", web::get().to(hello))
    );
}
