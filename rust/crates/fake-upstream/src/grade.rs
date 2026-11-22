//! `POST /api/v1/practice/grade` as the real upstream answers it: the set comes back as
//! `practice`, the answers as an object `question_id -> letter | letters`, and the response is the
//! score, one verdict per question, mastery per `materi` and recommendations. The tests hold it to
//! the recorded responses in `fixtures/practice/`, byte for byte once parsed.

use std::collections::{BTreeMap, BTreeSet};

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct GradeIn {
    practice: PracticeIn,
    answers: BTreeMap<String, LettersIn>,
}

#[derive(Deserialize)]
struct PracticeIn {
    questions: Vec<QuestionIn>,
}

#[derive(Deserialize)]
struct QuestionIn {
    question_id: String,
    materi: String,
    answer_key: LettersIn,
    pembahasan: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LettersIn {
    One(String),
    Many(Vec<String>),
}

impl LettersIn {
    /// Trimmed, upper-cased, de-duplicated and sorted, as the real grader compares them.
    fn normalized(&self) -> Vec<String> {
        let raw: Vec<&String> = match self {
            Self::One(one) => vec![one],
            Self::Many(many) => many.iter().collect(),
        };
        raw.into_iter()
            .map(|letter| letter.trim().to_uppercase())
            .filter(|letter| !letter.is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

pub(crate) async fn grade_practice(Json(body): Json<Value>) -> Response {
    match serde_json::from_value::<GradeIn>(body) {
        Ok(request) => Json(grade(&request)).into_response(),
        // The real route answers a body it cannot read with this 400 (fixture
        // `grade-error-answers-as-list`); the details differ, the shape does not.
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "message": "Invalid request body",
                    "type": "invalid_request_error",
                    "details": { "formErrors": [error.to_string()], "fieldErrors": {} },
                }
            })),
        )
            .into_response(),
    }
}

/// Math.round(100 * n / d) for non-negative integers, 0 when there is nothing to divide.
fn percent(n: usize, d: usize) -> usize {
    if d == 0 { 0 } else { (200 * n + d) / (2 * d) }
}

fn weakness(mastery: usize) -> &'static str {
    match mastery {
        80.. => "low",
        50.. => "medium",
        _ => "high",
    }
}

fn grade(request: &GradeIn) -> Value {
    let mut questions = Vec::new();
    // In first-seen order, like the real grader's Map.
    let mut materi: Vec<(String, usize, usize)> = Vec::new();
    for question in &request.practice.questions {
        let key = question.answer_key.normalized();
        let given = request
            .answers
            .get(&question.question_id)
            .map(LettersIn::normalized)
            .unwrap_or_default();
        let correct = given == key;
        questions.push(json!({
            "question_id": question.question_id,
            "materi": question.materi,
            "correct": correct,
            "answered": !given.is_empty(),
            "your_answer": given,
            "answer_key": key,
            "pembahasan": question.pembahasan,
        }));
        match materi
            .iter_mut()
            .find(|(name, _, _)| *name == question.materi)
        {
            Some(entry) => {
                entry.1 += 1;
                entry.2 += usize::from(correct);
            }
            None => materi.push((question.materi.clone(), 1, usize::from(correct))),
        }
    }
    let total = questions.len();
    let correct = materi.iter().map(|(_, _, right)| right).sum::<usize>();

    let mut weak: Vec<(&str, usize, usize, usize)> = materi
        .iter()
        .map(|(name, all, right)| (name.as_str(), *all, *right, percent(*right, *all)))
        .filter(|(_, _, _, mastery)| *mastery < 80)
        .collect();
    // Weakest first, then by name, so the same answers always read the same.
    weak.sort_by(|a, b| a.3.cmp(&b.3).then_with(|| a.0.cmp(b.0)));
    let recommendations: Vec<Value> = weak
        .iter()
        .map(|(name, all, right, mastery)| {
            let level = weakness(*mastery);
            let verb = if level == "high" {
                "Pelajari lagi"
            } else {
                "Ulangi sebagian"
            };
            json!({
                "materi": name,
                "weakness_level": level,
                "mastery_percentage": mastery,
                "message": format!("{verb} materi \"{name}\": benar {right} dari {all} soal."),
            })
        })
        .collect();

    json!({
        "score": { "correct": correct, "total": total, "percentage": percent(correct, total) },
        "questions": questions,
        "materi": materi
            .iter()
            .map(|(name, all, right)| {
                let mastery = percent(*right, *all);
                json!({
                    "materi": name,
                    "total_questions": all,
                    "correct_answers": right,
                    "mastery_percentage": mastery,
                    "weakness_level": weakness(mastery),
                })
            })
            .collect::<Vec<_>>(),
        "recommendations": recommendations,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests"
    )]

    use super::*;
    use crate::recorded;

    #[test]
    fn grading_matches_every_recorded_grade_exactly() {
        for name in [
            "grade-mcq_normal",
            "grade-mcq_multiple_correct",
            "grade-mcq_story",
            "grade-mcq_mixed",
            "grade-mcq_multiple_correct-subset-and-lowercase",
        ] {
            let meta = recorded::json(&format!("{name}.meta.json"));
            let request: GradeIn = serde_json::from_value(meta["request"]["body"].clone())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(
                grade(&request),
                recorded::json(&format!("{name}.json")),
                "{name}"
            );
        }
    }

    #[test]
    fn a_list_of_answers_is_refused_like_the_real_grade() {
        let meta = recorded::json("grade-error-answers-as-list.meta.json");
        assert!(serde_json::from_value::<GradeIn>(meta["request"]["body"].clone()).is_err());
        assert_eq!(meta["status"], 400);
    }

    #[test]
    fn percentages_round_half_up_like_math_round() {
        assert_eq!(percent(2, 3), 67);
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(1, 8), 13);
        assert_eq!(percent(0, 0), 0);
    }
}
