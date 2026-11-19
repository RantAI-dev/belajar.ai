use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    status: &'static str,
    version: &'static str,
}

#[utoipa::path(get, path = "/v1/health", responses((status = 200, body = Health)))]
pub async fn v1_health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}
