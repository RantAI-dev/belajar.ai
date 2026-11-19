//! The OpenAPI document: every route, every wire schema, in one place. Served at
//! `/v1/openapi.json`, printed by `--bin openapi`, and generated into `web/src/contracts` by
//! `scripts/gen-contracts.sh`.

use utoipa::OpenApi;

use crate::api_error::{ErrorBody, FieldError};
use crate::routes::auth::{
    LoginRequest, LogoutRequest, PasswordChangeRequest, RegisterRequest, StudentDto,
};
use crate::routes::health::Health as V1Health;
use crate::routes::me::PatchMeRequest;
use crate::routes::root::{Health as RootHealth, Ready};
use crate::routes::{auth, health, me, openapi, root};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "belajar.ai",
        version = env!("CARGO_PKG_VERSION"),
        description = "Accounts, profiles and sessions for the learning app. Every error body has the same shape: { code, message, requestId, errors? }.",
    ),
    paths(
        auth::register,
        auth::login,
        auth::logout,
        auth::change_password,
        me::get_me,
        me::patch_me,
        health::v1_health,
        root::health,
        root::ready,
        openapi::openapi_json,
    ),
    components(
        schemas(
            StudentDto,
            RegisterRequest,
            LoginRequest,
            LogoutRequest,
            PasswordChangeRequest,
            PatchMeRequest,
            ErrorBody,
            FieldError,
            RootHealth,
            Ready,
            V1Health,
        )
    ),
)]
pub struct ApiDoc;
