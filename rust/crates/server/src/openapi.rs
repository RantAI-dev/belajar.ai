//! The OpenAPI document: every route, every wire schema, in one place. Served at
//! `/v1/openapi.json`, printed by `--bin openapi`, and generated into `web/src/contracts` by
//! `scripts/gen-contracts.sh`.

use utoipa::OpenApi;

use crate::api_error::{ErrorBody, FieldError};
use crate::routes::auth::{
    LoginRequest, LogoutRequest, PasswordChangeRequest, RegisterRequest, StudentDto,
};
use crate::routes::google_auth::GoogleSignInDto;
use crate::routes::google_signup::{CompleteGoogleSignupRequest, PendingGoogleSignupDto};
use crate::routes::health::Health as V1Health;
use crate::routes::home::{ContinueItemDto, HomeDto, MissionDto};
use crate::routes::lessons::{
    CompleteLessonRequest, CompleteLessonResponse, CreateLessonRequest, LessonDto,
    PatchLessonRequest,
};
use crate::routes::me::PatchMeRequest;
use crate::routes::practice::practice_dto::{
    OptionDto, PracticeResultDto, PracticeSetDto, PracticeSetListDto, PracticeSetSummaryDto,
    PracticeStatusDto, QuestionDto, QuestionKindDto, ResultItemDto, TopicMasteryDto,
    TopicMasteryListDto,
};
use crate::routes::practice::practice_generate::{
    GeneratePracticeRequest, ProgressEventDto, ReadyEventDto,
};
use crate::routes::practice::practice_submit::{SubmitAnswerDto, SubmitPracticeRequest};
use crate::routes::practice::{practice_generate, practice_submit, practice_views};
use crate::routes::root::{Health as RootHealth, Ready};
use crate::routes::subjects::{ChapterDto, ChaptersDto, SubjectDto, SubjectsDto};
use crate::routes::tanya::{
    ConversationDto, ConversationListDto, ConversationModeDto, CreateConversationRequest,
    MessageDto, MessageListDto, MessageRoleDto, RenameConversationRequest, SourceDto,
};
use crate::routes::tanya_messages::{DeltaData, SendMessageRequest};
use crate::routes::{
    auth, google_auth, google_signup, health, home, lessons, me, openapi, root, subjects, tanya,
    tanya_messages, tanya_pages,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "belajar.ai",
        version = env!("CARGO_PKG_VERSION"),
        description = "Accounts, profiles, sessions, subjects, guided lessons, the home screen, conversations, streamed answers and practice sets for the learning app. Every error body has the same shape: { code, message, requestId, errors? }.",
    ),
    paths(
        auth::register,
        auth::login,
        auth::logout,
        auth::change_password,
        google_auth::google_status,
        google_auth::google_start,
        google_auth::google_callback,
        google_signup::get_pending,
        google_signup::complete,
        me::get_me,
        me::patch_me,
        subjects::list_subjects,
        subjects::list_chapters,
        lessons::create_lesson,
        lessons::get_lesson,
        lessons::patch_lesson,
        lessons::complete_lesson,
        home::get_home,
        tanya::create_conversation,
        tanya::get_conversation,
        tanya::patch_conversation,
        tanya::list_conversations,
        tanya_messages::post_message,
        tanya_messages::list_messages,
        tanya_pages::message_source_page,
        practice_generate::create_practice_set,
        practice_views::list_practice_sets,
        practice_views::get_practice_set,
        practice_submit::submit_practice_set,
        practice_views::question_page,
        practice_views::topic_mastery,
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
            GoogleSignInDto,
            PendingGoogleSignupDto,
            CompleteGoogleSignupRequest,
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
            CreateConversationRequest,
            RenameConversationRequest,
            ConversationModeDto,
            ConversationDto,
            ConversationListDto,
            MessageRoleDto,
            MessageDto,
            MessageListDto,
            SourceDto,
            SendMessageRequest,
            DeltaData,
            GeneratePracticeRequest,
            ProgressEventDto,
            ReadyEventDto,
            PracticeSetDto,
            PracticeSetSummaryDto,
            PracticeSetListDto,
            PracticeStatusDto,
            QuestionDto,
            QuestionKindDto,
            OptionDto,
            ResultItemDto,
            PracticeResultDto,
            SubmitPracticeRequest,
            SubmitAnswerDto,
            TopicMasteryDto,
            TopicMasteryListDto,
            ErrorBody,
            FieldError,
            RootHealth,
            Ready,
            V1Health,
        )
    ),
)]
pub struct ApiDoc;
