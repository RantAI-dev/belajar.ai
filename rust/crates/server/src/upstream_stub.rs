//! The upstream until the upstream slice lands: every method answers `NotConfigured`, which maps
//! to 503 `unavailable`. Delete this file when `belajar-upstream` is wired into `main.rs`.

use async_trait::async_trait;
use belajar_core::grade::Grade;
use belajar_ports::upstream::{
    Assistant, Chapter, ChatRequest, ChatStream, Mapel, PageImage, PracticeAnswer, PracticeRequest,
    PracticeSet, PracticeStream, QuestionScore, Upstream, UpstreamError,
};

pub struct UpstreamStub;

#[async_trait]
impl Upstream for UpstreamStub {
    async fn mapel(&self, grade: Grade) -> Result<Vec<Mapel>, UpstreamError> {
        Err(not_configured(grade, Assistant::Ask))
    }

    async fn chapters(&self, grade: Grade, _mapel: &str) -> Result<Vec<Chapter>, UpstreamError> {
        Err(not_configured(grade, Assistant::Ask))
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatStream, UpstreamError> {
        Err(not_configured(request.grade, request.assistant))
    }

    async fn generate_practice(
        &self,
        request: PracticeRequest,
    ) -> Result<PracticeStream, UpstreamError> {
        Err(not_configured(request.grade, Assistant::Practice))
    }

    async fn grade_practice(
        &self,
        grade: Grade,
        _set: &PracticeSet,
        _answers: &[PracticeAnswer],
    ) -> Result<Vec<QuestionScore>, UpstreamError> {
        Err(not_configured(grade, Assistant::Practice))
    }

    // `page` has no grade in scope, so it reports the same fact as Unavailable instead.
    async fn page(&self, _page_url: &str) -> Result<PageImage, UpstreamError> {
        Err(UpstreamError::Unavailable(
            "the upstream learning API is not configured".to_owned(),
        ))
    }
}

fn not_configured(grade: Grade, assistant: Assistant) -> UpstreamError {
    UpstreamError::NotConfigured {
        grade: grade.get(),
        assistant,
    }
}
