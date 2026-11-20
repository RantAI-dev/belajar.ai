//! An in-process fake of Google's OpenID endpoints for the sign-in tests: a token endpoint that
//! checks the client secret, the redirect URL and the PKCE verifier, and a JWKS endpoint. It
//! listens on 127.0.0.1, so the gate needs no network. The RSA keys are generated at test time;
//! nothing secret is committed.

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(dead_code, reason = "each test uses part of the fake")]

pub mod flow;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use belajar_server::config::{GoogleConfig, Secret};
use jsonwebtoken::jwk::{Jwk, JwkSet};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use rsa::RsaPrivateKey;
use rsa::pkcs1::EncodeRsaPrivateKey;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const CLIENT_ID: &str = "belajar-test.apps.example";
pub const KID: &str = "fake-key-1";
pub const REDIRECT_URL: &str = "http://localhost/v1/auth/google/callback";
/// Not a real credential: the fake only checks that the server sends what it was configured with.
const CLIENT_SECRET: &str = "fake-client-secret-for-tests";

/// Which key signs an ID token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signer {
    /// The key the JWKS publishes.
    Published,
    /// Another key, under the published `kid`: a forged token.
    Rogue,
}

struct Keys {
    published: EncodingKey,
    rogue: EncodingKey,
    jwks: JwkSet,
}

/// Two 2048-bit keys, made once per test binary.
fn keys() -> &'static Keys {
    static KEYS: OnceLock<Keys> = OnceLock::new();
    KEYS.get_or_init(|| {
        let make = || {
            let key = RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048).unwrap();
            EncodingKey::from_rsa_der(key.to_pkcs1_der().unwrap().as_bytes())
        };
        let published = make();
        let mut jwk = Jwk::from_encoding_key(&published, Algorithm::RS256).unwrap();
        jwk.common.key_id = Some(KID.to_owned());
        Keys {
            published,
            rogue: make(),
            jwks: JwkSet { keys: vec![jwk] },
        }
    })
}

/// What the token endpoint answers for one authorization code.
struct Grant {
    code_challenge: String,
    id_token: String,
}

#[derive(Default)]
struct Inner {
    grants: HashMap<String, Grant>,
    jwks_hits: usize,
    token_hits: usize,
    /// Answer every token request with this status instead (a Google outage).
    token_status: Option<u16>,
}

#[derive(Clone)]
pub struct FakeGoogle {
    pub base: String,
    inner: Arc<Mutex<Inner>>,
}

impl FakeGoogle {
    pub async fn start() -> FakeGoogle {
        let inner = Arc::new(Mutex::new(Inner::default()));
        let router = Router::new()
            .route("/token", post(token))
            .route("/certs", get(certs))
            .with_state(inner.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, router).await });
        FakeGoogle { base, inner }
    }

    pub fn issuer(&self) -> String {
        "https://accounts.google.com".to_owned()
    }

    /// The server's config, pointed at this fake.
    pub fn config(&self) -> GoogleConfig {
        GoogleConfig {
            client_id: CLIENT_ID.to_owned(),
            client_secret: Secret::new(CLIENT_SECRET.to_owned()),
            redirect_url: REDIRECT_URL.to_owned(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_owned(),
            token_url: format!("{}/token", self.base),
            jwks_url: format!("{}/certs", self.base),
            issuer: self.issuer(),
            timeout_ms: 5_000,
        }
    }

    /// Claims a real Google ID token for `sub` would carry for this attempt.
    pub fn claims(&self, sub: &str, name: &str, nonce: &str) -> Value {
        let now = chrono::Utc::now().timestamp();
        json!({
            "iss": self.issuer(),
            "azp": CLIENT_ID,
            "aud": CLIENT_ID,
            "sub": sub,
            "name": name,
            "nonce": nonce,
            "iat": now,
            "exp": now + 3_600,
        })
    }

    /// Registers an authorization code for the PKCE `code_challenge`, answered with `claims`
    /// signed by `signer`. Returns the code.
    pub fn issue_code(&self, code_challenge: &str, claims: &Value, signer: Signer) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(KID.to_owned());
        let key = match signer {
            Signer::Published => &keys().published,
            Signer::Rogue => &keys().rogue,
        };
        let id_token = encode(&header, claims, key).unwrap();
        let code = format!("code-{}", uuid::Uuid::now_v7());
        self.inner.lock().unwrap().grants.insert(
            code.clone(),
            Grant {
                code_challenge: code_challenge.to_owned(),
                id_token,
            },
        );
        code
    }

    pub fn fail_token_requests_with(&self, status: u16) {
        self.inner.lock().unwrap().token_status = Some(status);
    }

    pub fn jwks_hits(&self) -> usize {
        self.inner.lock().unwrap().jwks_hits
    }

    pub fn token_hits(&self) -> usize {
        self.inner.lock().unwrap().token_hits
    }
}

fn invalid_grant() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "invalid_grant" })),
    )
        .into_response()
}

async fn token(
    State(inner): State<Arc<Mutex<Inner>>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let mut inner = inner.lock().unwrap();
    inner.token_hits += 1;
    if let Some(status) = inner.token_status {
        return StatusCode::from_u16(status).unwrap().into_response();
    }
    let field = |name: &str| form.get(name).map(String::as_str).unwrap_or_default();
    if field("grant_type") != "authorization_code"
        || field("client_id") != CLIENT_ID
        || field("client_secret") != CLIENT_SECRET
        || field("redirect_uri") != REDIRECT_URL
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_client" })),
        )
            .into_response();
    }
    // A code is single use, like Google's.
    let Some(grant) = inner.grants.remove(field("code")) else {
        return invalid_grant();
    };
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(field("code_verifier").as_bytes()));
    if challenge != grant.code_challenge {
        return invalid_grant();
    }
    Json(json!({
        "access_token": "fake-access-token",
        "expires_in": 3599,
        "token_type": "Bearer",
        "scope": "openid profile",
        "id_token": grant.id_token,
    }))
    .into_response()
}

async fn certs(State(inner): State<Arc<Mutex<Inner>>>) -> Json<JwkSet> {
    inner.lock().unwrap().jwks_hits += 1;
    Json(keys().jwks.clone())
}
