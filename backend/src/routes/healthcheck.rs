use crate::define_route_error;
use actix_web::HttpResponse;
use reqwest::StatusCode;

define_route_error! {
    HealthCheckError {
        ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, "Service is unavailable"),
    }
}

pub async fn healthcheck() -> Result<HttpResponse, HealthCheckError> {
    Ok(HttpResponse::Ok().finish())
}
