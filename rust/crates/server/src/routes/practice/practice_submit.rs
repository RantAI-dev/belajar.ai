//! `POST /v1/practice-sets/{id}/submit`: validate the answers against the stored set, grade them
//! through the upstream, then write the whole submission — answer rows, the set's status and
//! score, topic mastery and stars — in one transaction.

use std::collections::{HashMap, HashSet};

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use belajar_ports::practice::{GradedAnswer, PracticeSetStatus, SubmitRecord};
use belajar_ports::upstream::{PracticeAnswer, PracticeQuestion, QuestionKind};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody, FieldError};
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::routes::practice::internal;
use crate::routes::practice::practice_dto::{GradedQuestion, PracticeResultDto, result_dto};
use crate::state::AppState;
use crate::store_errors;
use crate::upstream_errors;
use crate::wire::extract::ApiJson;
use crate::wire::fields;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubmitPracticeRequest {
    pub answers: Vec<SubmitAnswerDto>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubmitAnswerDto {
    pub question_id: String,
    pub selected: Vec<String>,
}

/// Grades a submitted set. The answer key is revealed here and nowhere before.
#[utoipa::path(
    post,
    path = "/v1/practice-sets/{id}/submit",
    request_body = SubmitPracticeRequest,
    params(("id" = Uuid, Path, description = "The practice set")),
    responses(
        (status = 200, description = "The graded review: correct answers, explanations and sources", body = PracticeResultDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
        (status = 409, description = "The set was already submitted", body = ErrorBody),
        (status = 502, body = ErrorBody),
        (status = 503, body = ErrorBody),
        (status = 504, body = ErrorBody),
    )
)]
pub(crate) async fn submit_practice_set(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(set_id): Path<Uuid>,
    ApiJson(req): ApiJson<SubmitPracticeRequest>,
) -> Response {
    let row = match state.practice.get_set(student.student_id, set_id).await {
        Ok(Some(row)) => row,
        Ok(None) => return ApiError::new(ErrorCode::NotFound).into_response(&id.0),
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let errors = validate_answers(&row.practice.questions, &req.answers);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    if row.status == PracticeSetStatus::Submitted {
        return ApiError::new(ErrorCode::Conflict).into_response(&id.0);
    }

    // Answers in question order, so the review reads the way the set does.
    let mut by_id: HashMap<&str, &SubmitAnswerDto> = HashMap::new();
    for answer in &req.answers {
        by_id.insert(answer.question_id.as_str(), answer);
    }
    let mut pairs: Vec<(&PracticeQuestion, &SubmitAnswerDto)> = Vec::new();
    for question in &row.practice.questions {
        let Some(answer) = by_id.get(question.id.as_str()) else {
            return internal(&id, "an answer went missing after validation");
        };
        pairs.push((question, answer));
    }

    let account = match state.accounts.get_student(student.student_id).await {
        Ok(account) => account,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let upstream_answers: Vec<PracticeAnswer> = pairs
        .iter()
        .map(|(question, answer)| PracticeAnswer {
            question_id: question.id.clone(),
            selected: answer.selected.clone(),
        })
        .collect();
    let scores = match state
        .upstream
        .grade_practice(account.grade, &row.practice, &upstream_answers)
        .await
    {
        Ok(scores) => scores,
        Err(err) => return upstream_errors::into_response(&err, &id.0),
    };
    let mut score_by_id: HashMap<&str, bool> = HashMap::new();
    for score in &scores {
        score_by_id.insert(score.question_id.as_str(), score.correct);
    }

    let mut correct_count = 0u32;
    let mut graded: Vec<GradedQuestion<'_>> = Vec::with_capacity(pairs.len());
    let mut graded_answers: Vec<GradedAnswer> = Vec::with_capacity(pairs.len());
    for (question, answer) in pairs {
        // The upstream decides; a question it said nothing about falls back to comparing the
        // selection with the answer key as a set.
        let is_correct = score_by_id
            .get(question.id.as_str())
            .copied()
            .unwrap_or_else(|| same_set(&answer.selected, &question.answer));
        if is_correct {
            correct_count += 1;
        }
        graded.push(GradedQuestion {
            question,
            selected: &answer.selected,
            is_correct,
        });
        graded_answers.push(GradedAnswer {
            question_id: question.id.clone(),
            selected: answer.selected.clone(),
            is_correct,
        });
    }
    let stars_earned = 5 * correct_count;

    let record = SubmitRecord {
        student_id: student.student_id,
        practice_set_id: row.id,
        subject_id: row.subject_id.clone(),
        topic: row.topic.clone(),
        attempts: graded.len() as u32,
        correct: correct_count,
        stars_earned,
        submitted_at: state.clock.now(),
        answers: graded_answers,
    };
    if let Err(err) = state.practice.submit_set(&record).await {
        return store_errors::into_response(err, &id.0);
    }

    let dto = result_dto(row.id, &graded, correct_count, stars_earned);
    (StatusCode::OK, Json(dto)).into_response()
}

/// Two labels picked mean the same two labels keyed, whatever the order.
fn same_set(selected: &[String], key: &[String]) -> bool {
    let mut left: Vec<&str> = selected.iter().map(String::as_str).collect();
    let mut right: Vec<&str> = key.iter().map(String::as_str).collect();
    left.sort_unstable();
    right.sort_unstable();
    left == right
}

/// Exactly one answer per question, each label one of that question's options, single and story
/// questions with exactly one label, multiple-correct ones with one or more.
fn validate_answers(
    questions: &[PracticeQuestion],
    answers: &[SubmitAnswerDto],
) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut bad_set = false;
    for answer in answers {
        let known = questions
            .iter()
            .any(|question| question.id == answer.question_id);
        if !known || !seen.insert(answer.question_id.as_str()) {
            bad_set = true;
        }
    }
    if bad_set || seen.len() != questions.len() {
        errors.push(fields::invalid_format(
            "/answers",
            "exactly one answer per question of this set",
        ));
    }
    for (index, answer) in answers.iter().enumerate() {
        let Some(question) = questions
            .iter()
            .find(|question| question.id == answer.question_id)
        else {
            continue; // already one /answers error
        };
        let labels: HashSet<&str> = question
            .options
            .iter()
            .map(|option| option.label.as_str())
            .collect();
        if !answer
            .selected
            .iter()
            .all(|label| labels.contains(label.as_str()))
        {
            errors.push(fields::invalid_format(
                &format!("/answers/{index}/selected"),
                "only labels of that question's options",
            ));
        }
        let count_ok = match question.kind {
            QuestionKind::Single | QuestionKind::Story => answer.selected.len() == 1,
            QuestionKind::Multiple => !answer.selected.is_empty(),
        };
        if !count_ok {
            let field = format!("/answers/{index}/selected");
            let error = match question.kind {
                QuestionKind::Multiple => fields::out_of_count(&field, 1, None),
                _ => fields::out_of_count(&field, 1, Some(1)),
            };
            errors.push(error);
        }
    }
    errors
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use belajar_ports::upstream::{PracticeOption, PracticeQuestion, PracticeSet, QuestionKind};

    use super::{SubmitAnswerDto, same_set, validate_answers};

    fn question(id: &str, kind: QuestionKind, labels: &[&str]) -> PracticeQuestion {
        PracticeQuestion {
            id: id.to_owned(),
            kind,
            story: None,
            stem: "Stem?".into(),
            options: labels
                .iter()
                .map(|label| PracticeOption {
                    label: (*label).to_owned(),
                    text: "Option".into(),
                })
                .collect(),
            answer: vec!["A".to_owned()],
            explanation: "Because".into(),
            source: None,
        }
    }

    fn answer(question_id: &str, selected: &[&str]) -> SubmitAnswerDto {
        SubmitAnswerDto {
            question_id: question_id.to_owned(),
            selected: selected.iter().map(|label| (*label).to_owned()).collect(),
        }
    }

    fn set_of(questions: &[PracticeQuestion]) -> PracticeSet {
        PracticeSet {
            questions: questions.to_vec(),
            raw: serde_json::json!({}),
        }
    }

    #[test]
    fn answers_must_cover_each_question_exactly_once() {
        let questions = vec![
            question("q1", QuestionKind::Single, &["A", "B"]),
            question("q2", QuestionKind::Single, &["A", "B"]),
        ];
        // Missing q2.
        let errors = validate_answers(&questions, &[answer("q1", &["A"])]);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "/answers");
        // q1 twice.
        let errors = validate_answers(
            &questions,
            &[
                answer("q1", &["A"]),
                answer("q1", &["B"]),
                answer("q2", &["A"]),
            ],
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "/answers");
        // An id no question has.
        let errors = validate_answers(
            &questions,
            &[
                answer("q1", &["A"]),
                answer("q9", &["A"]),
                answer("q2", &["A"]),
            ],
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "/answers");
    }

    #[test]
    fn labels_and_counts_are_checked_per_answer() {
        let questions = vec![
            question("q1", QuestionKind::Single, &["A", "B"]),
            question("q2", QuestionKind::Multiple, &["A", "B", "C"]),
        ];
        // A label q1 does not have, and two labels on a single question.
        let errors = validate_answers(
            &questions,
            &[answer("q1", &["Z"]), answer("q2", &["A", "B"])],
        );
        let mut fields: Vec<&str> = errors.iter().map(|error| error.field.as_str()).collect();
        fields.sort_unstable();
        assert_eq!(fields, vec!["/answers/0/selected"]);
        // An empty selection is one per-answer error for either kind.
        let errors = validate_answers(&questions, &[answer("q1", &[]), answer("q2", &[])]);
        let fields: Vec<&str> = errors.iter().map(|error| error.field.as_str()).collect();
        assert_eq!(fields, vec!["/answers/0/selected", "/answers/1/selected"]);
        // A multiple-correct question accepts one or more.
        let errors = validate_answers(&questions, &[answer("q1", &["A"]), answer("q2", &["C"])]);
        assert!(errors.is_empty());
    }

    #[test]
    fn a_valid_submission_has_no_errors_even_on_many_labels() {
        let questions = vec![question("q1", QuestionKind::Multiple, &["A", "B", "C"])];
        let errors = validate_answers(&questions, &[answer("q1", &["A", "C"])]);
        assert!(errors.is_empty());
        // Not graded here, but a lone partial selection on a multiple question is well-formed.
        assert!(!same_set(
            &["A".to_owned()],
            &["A".to_owned(), "C".to_owned()]
        ));
        assert!(same_set(
            &["C".to_owned(), "A".to_owned()],
            &["A".to_owned(), "C".to_owned()]
        ));
        assert!(same_set(&["B".to_owned()], &["B".to_owned()]));
    }

    #[test]
    fn the_set_of_questions_is_only_used_through_its_borrow() {
        // The unused-set warning would appear if the helper kept a copy; one call pins the shape.
        let questions = vec![question("q1", QuestionKind::Story, &["A", "B"])];
        let set = set_of(&questions);
        let errors = validate_answers(&set.questions, &[answer("q1", &["A"])]);
        assert!(errors.is_empty());
    }
}
