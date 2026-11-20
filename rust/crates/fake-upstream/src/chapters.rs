//! The upstream's chapters endpoint, backed by the responses recorded from the real service.
//! Only the three subjects we recorded have chapters; other known subjects answer with an empty
//! list so the app's empty state is exercised, and an unknown mapel answers 400 like the real one.

use axum::Json;
use axum::extract::Query;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

const IPA: &str = include_str!("../../../fixtures/chapters/IPA.json");
const MATEMATIKA: &str = include_str!("../../../fixtures/chapters/Matematika.json");
const BAHASA_INGGRIS: &str = include_str!("../../../fixtures/chapters/Bahasa-Inggris.json");
const UNKNOWN_MAPEL: &str = include_str!("../../../fixtures/chapters/unknown.json");

#[derive(Debug, Deserialize)]
pub(crate) struct ChaptersQuery {
    mapel: String,
}

/// `GET /api/v1/mapel/chapters?mapel=…`, behind the fake's bearer-key guard.
pub(crate) async fn chapters(Query(query): Query<ChaptersQuery>) -> Response {
    let body: String = match query.mapel.as_str() {
        "IPA" => IPA.to_owned(),
        "Matematika" => MATEMATIKA.to_owned(),
        "Bahasa Inggris" => BAHASA_INGGRIS.to_owned(),
        "Bahasa Indonesia"
        | "IPS"
        | "Informatika"
        | "Pendidikan Pancasila"
        | "Pendidikan Agama Islam" => {
            json!({ "object": "list", "mapel": query.mapel, "data": [] }).to_string()
        }
        other => {
            // The recorded 400 already has the upstream's error shape; only name the mapel asked.
            let mut body: Value = serde_json::from_str(UNKNOWN_MAPEL).unwrap_or_else(|_| {
                json!({ "error": { "message": "Unknown mapel", "type": "invalid_request_error" } })
            });
            if let Some(message) = body.get_mut("error").and_then(|e| e.get_mut("message")) {
                *message = json!(format!(
                    "Unknown mapel \"{other}\" for this assistant. Call GET /api/v1/mapel for the list."
                ));
            }
            return (StatusCode::BAD_REQUEST, Json(body)).into_response();
        }
    };
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "tests")]

    use super::*;

    #[test]
    fn the_recorded_unknown_mapel_response_is_a_400_error_body() {
        let value: serde_json::Value = serde_json::from_str(UNKNOWN_MAPEL).unwrap();
        assert_eq!(value["error"]["type"], "invalid_request_error");
        assert!(
            value["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Unknown mapel")
        );
    }

    #[test]
    fn the_recorded_fixtures_are_the_real_books_not_the_built_in_titles() {
        for (raw, title) in [
            (IPA, "Sistem Organisasi Kehidupan Makhluk Hidup"),
            (MATEMATIKA, "Bilangan"),
            (BAHASA_INGGRIS, "Good morning. How are You?"),
        ] {
            let value: serde_json::Value = serde_json::from_str(raw).unwrap();
            assert_eq!(value["data"][0]["title"], title);
            assert!(value["data"].as_array().unwrap().len() >= 8);
        }
    }
}
