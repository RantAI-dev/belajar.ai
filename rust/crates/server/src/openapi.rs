//! The OpenAPI document: every route, every wire schema, in one place. Served at
//! `/v1/openapi.json`, printed by `--bin openapi`, and generated into `web/src/contracts` by
//! `scripts/gen-contracts.sh`.

use utoipa::OpenApi;

use crate::api_error::{ErrorBody, FieldError};
use crate::routes::auth::{
    LoginRequest, LogoutRequest, PasswordChangeRequest, RegisterRequest, StudentDto,
};
use crate::routes::health::Health as V1Health;
use crate::routes::home::{ContinueItemDto, HomeDto, MissionDto};
use crate::routes::lessons::{
    CompleteLessonRequest, CompleteLessonResponse, CreateLessonRequest, LessonDto,
    PatchLessonRequest,
};
use crate::routes::me::PatchMeRequest;
use crate::routes::root::{Health as RootHealth, Ready};
use crate::routes::subjects::{ChapterDto, ChaptersDto, SubjectDto, SubjectsDto};
use crate::routes::{auth, health, home, lessons, me, openapi, root, subjects};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "belajar.ai",
        version = env!("CARGO_PKG_VERSION"),
        description = "Accounts, profiles, sessions, subjects, guided lessons and the home screen for the learning app. Every error body has the same shape: { code, message, requestId, errors? }.",
    ),
    paths(
        auth::register,
        auth::login,
        auth::logout,
        auth::change_password,
        me::get_me,
        me::patch_me,
        subjects::list_subjects,
        subjects::list_chapters,
        lessons::create_lesson,
        lessons::get_lesson,
        lessons::patch_lesson,
        lessons::complete_lesson,
        home::get_home,
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
            SubjectDto,
            SubjectsDto,
            ChapterDto,
            ChaptersDto,
            CreateLessonRequest,
            PatchLessonRequest,
            CompleteLessonRequest,
            LessonDto,
            CompleteLessonResponse,
            HomeDto,
            MissionDto,
            ContinueItemDto,
            ErrorBody,
            FieldError,
            RootHealth,
            Ready,
            V1Health,
        )
    ),
)]
pub struct ApiDoc;
