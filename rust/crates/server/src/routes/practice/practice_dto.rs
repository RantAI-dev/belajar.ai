//! The wire types of Latihan, and their conversions from the stored and upstream types. The
//! answer key never appears here: nothing in this module serializes `PracticeQuestion::answer`,
//! `explanation` or the upstream's `page_url` until a set is submitted.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use belajar_ports::practice::{PracticeSetStatus, PracticeSetSummary, StoredPracticeSet};
use belajar_ports::upstream::{PracticeQuestion, PracticeType, QuestionKind, Source};

use crate::wire::instant;
use crate::wire::mastery::MasteryLevelDto;

/// One question's kind as the client sees it.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKindDto {
    Single,
    Multiple,
    Story,
}

impl From<QuestionKind> for QuestionKindDto {
    fn from(kind: QuestionKind) -> Self {
        match kind {
            QuestionKind::Single => Self::Single,
            QuestionKind::Multiple => Self::Multiple,
            QuestionKind::Story => Self::Story,
        }
    }
}

/// One answer option.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OptionDto {
    pub label: String,
    pub text: String,
}

/// One question, with nothing that reveals the answer. `number` is 1-based.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionDto {
    pub id: String,
    pub number: u32,
    pub kind: QuestionKindDto,
    pub story: Option<String>,
    pub stem: String,
    pub options: Vec<OptionDto>,
}

/// The same source shape Tanya answers carry, so the web app has one type for both.
pub use crate::routes::tanya::SourceDto;

/// One graded question of the review, in question order.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResultItemDto {
    pub question_id: String,
    pub is_correct: bool,
    pub selected: Vec<String>,
    pub correct_answer: Vec<String>,
    pub explanation: String,
    pub source: Option<SourceDto>,
}

/// The review a submission returns, and what `GET /v1/practice-sets/{id}` keeps showing after.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PracticeResultDto {
    pub correct_count: u32,
    pub total_count: u32,
    pub stars_earned: u32,
    pub items: Vec<ResultItemDto>,
}

/// A set's status: `ready` until it is submitted, `submitted` for ever after.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PracticeStatusDto {
    Ready,
    Submitted,
}

impl From<PracticeSetStatus> for PracticeStatusDto {
    fn from(status: PracticeSetStatus) -> Self {
        match status {
            PracticeSetStatus::Ready => Self::Ready,
            PracticeSetStatus::Submitted => Self::Submitted,
        }
    }
}

/// One practice set. `result` is `null` until the student submits.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PracticeSetDto {
    pub id: Uuid,
    pub subject_id: String,
    pub topic: String,
    pub practice_type: String,
    pub total_questions: u16,
    pub status: PracticeStatusDto,
    pub questions: Vec<QuestionDto>,
    pub result: Option<PracticeResultDto>,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub created_at: DateTime<Utc>,
}

/// One row of the list: nothing from the stored practice JSON.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PracticeSetSummaryDto {
    pub id: Uuid,
    pub subject_id: String,
    pub topic: String,
    pub practice_type: String,
    pub total_questions: u16,
    pub status: PracticeStatusDto,
    pub score_correct: Option<u16>,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PracticeSetListDto {
    pub data: Vec<PracticeSetSummaryDto>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicMasteryDto {
    pub subject_id: String,
    pub topic: String,
    pub attempts: u32,
    pub correct: u32,
    pub mastery_percent: u32,
    /// The level `belajar_core::mastery::mastery_level` gives these answers.
    pub level: MasteryLevelDto,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicMasteryListDto {
    pub data: Vec<TopicMasteryDto>,
    pub next_cursor: Option<String>,
}

/// One question with its grade, in question order: what the review is built from.
pub(crate) struct GradedQuestion<'a> {
    pub question: &'a PracticeQuestion,
    pub selected: &'a [String],
    pub is_correct: bool,
}

/// The four practice type names, in both directions. Validated by hand, so a bad request value
/// is one field error beside the others, not a decode failure.
pub(crate) fn practice_type_from_name(name: &str) -> Option<PracticeType> {
    match name {
        "mcq_normal" => Some(PracticeType::McqNormal),
        "mcq_multiple_correct" => Some(PracticeType::McqMultipleCorrect),
        "mcq_story" => Some(PracticeType::McqStory),
        "mcq_mixed" => Some(PracticeType::McqMixed),
        _ => None,
    }
}

pub(crate) fn practice_type_name(practice_type: PracticeType) -> &'static str {
    match practice_type {
        PracticeType::McqNormal => "mcq_normal",
        PracticeType::McqMultipleCorrect => "mcq_multiple_correct",
        PracticeType::McqStory => "mcq_story",
        PracticeType::McqMixed => "mcq_mixed",
    }
}

/// The set as a client sees it: questions without any answer material, plus the review once
/// there is one.
pub(crate) fn set_dto(
    row: &StoredPracticeSet,
    result: Option<PracticeResultDto>,
) -> PracticeSetDto {
    PracticeSetDto {
        id: row.id,
        subject_id: row.subject_id.clone(),
        topic: row.topic.clone(),
        practice_type: practice_type_name(row.practice_type).to_owned(),
        total_questions: row.total_questions,
        status: row.status.into(),
        questions: row
            .practice
            .questions
            .iter()
            .enumerate()
            .map(|(index, question)| question_dto(question, (index + 1) as u32))
            .collect(),
        result,
        created_at: row.created_at,
    }
}

fn question_dto(question: &PracticeQuestion, number: u32) -> QuestionDto {
    QuestionDto {
        id: question.id.clone(),
        number,
        kind: question.kind.into(),
        story: question.story.clone(),
        stem: question.stem.clone(),
        options: question
            .options
            .iter()
            .map(|option| OptionDto {
                label: option.label.clone(),
                text: option.text.clone(),
            })
            .collect(),
    }
}

pub(crate) fn summary_dto(summary: &PracticeSetSummary) -> PracticeSetSummaryDto {
    PracticeSetSummaryDto {
        id: summary.id,
        subject_id: summary.subject_id.clone(),
        topic: summary.topic.clone(),
        practice_type: practice_type_name(summary.practice_type).to_owned(),
        total_questions: summary.total_questions,
        status: summary.status.into(),
        score_correct: summary.score_correct,
        created_at: summary.created_at,
    }
}

pub(crate) fn source_dto(source: &Source, set_id: Uuid, question_id: &str) -> SourceDto {
    SourceDto {
        number: source.number,
        title: source.title.clone(),
        section: source.section.clone(),
        page: source.page,
        is_cited: source.cited,
        // The upstream's signed `page_url` stays server-side; the client fetches through us.
        page_preview_url: source
            .page_url
            .as_ref()
            .map(|_| format!("/v1/practice-sets/{set_id}/questions/{question_id}/page")),
    }
}

/// The review: items in question order, with the key, the explanation and the citation revealed.
pub(crate) fn result_dto(
    set_id: Uuid,
    graded: &[GradedQuestion<'_>],
    correct_count: u32,
    stars_earned: u32,
) -> PracticeResultDto {
    PracticeResultDto {
        correct_count,
        total_count: graded.len() as u32,
        stars_earned,
        items: graded
            .iter()
            .map(|graded| ResultItemDto {
                question_id: graded.question.id.clone(),
                is_correct: graded.is_correct,
                selected: graded.selected.to_vec(),
                correct_answer: graded.question.answer.clone(),
                explanation: graded.question.explanation.clone(),
                source: graded
                    .question
                    .source
                    .as_ref()
                    .map(|source| source_dto(source, set_id, &graded.question.id)),
            })
            .collect(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use super::*;

    fn source(page_url: Option<&str>) -> Source {
        Source {
            number: 1,
            title: "IPA VII".into(),
            section: Some("Bab 5".into()),
            document_id: "kb-ipa-7".into(),
            page: Some(70),
            cited: true,
            page_url: page_url.map(str::to_owned),
        }
    }

    #[test]
    fn the_source_never_carries_the_upstreams_url() {
        let set_id = Uuid::nil();
        let dto = source_dto(
            &source(Some("http://upstream/pages/kb-ipa-7/70.svg?sig=x")),
            set_id,
            "q1",
        );
        assert_eq!(
            dto.page_preview_url.as_deref(),
            Some("/v1/practice-sets/00000000-0000-0000-0000-000000000000/questions/q1/page")
        );
        let wire = serde_json::to_value(&dto).unwrap();
        let text = wire.to_string();
        assert!(!text.contains("upstream"), "{text}");
        assert!(!text.contains("sig"), "{text}");
        assert_eq!(
            wire["pagePreviewUrl"],
            serde_json::json!(dto.page_preview_url)
        );
    }

    #[test]
    fn a_source_without_a_page_has_no_preview_url() {
        let dto = source_dto(&source(None), Uuid::nil(), "q1");
        assert_eq!(dto.page_preview_url, None);
    }

    #[test]
    fn practice_type_names_round_trip() {
        for name in [
            "mcq_normal",
            "mcq_multiple_correct",
            "mcq_story",
            "mcq_mixed",
        ] {
            let practice_type = practice_type_from_name(name).unwrap();
            assert_eq!(practice_type_name(practice_type), name);
        }
        assert!(practice_type_from_name("essay").is_none());
    }
}
