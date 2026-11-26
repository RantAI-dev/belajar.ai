//! What the model is told. Indonesian, for school students, answering only from their books.

use crate::llm::Message;
use crate::rag::{Passage, context_block};

/// The kind refusal when the books do not cover a question. Grounded answers never use it.
pub const NOT_IN_BOOK: &str = "Hmm, aku belum menemukan jawabannya di buku pelajaranmu. Coba \
    tanyakan dengan kata lain, atau pilih mapel yang sesuai, ya.";

const RULES: &str = "Aturan menjawab:\n\
- Jawab HANYA dari kutipan buku di bawah. Jangan menambah fakta dari luar buku.\n\
- Setiap kalimat yang memakai isi buku diberi nomor kutipannya, misalnya [1] atau [2], tepat \
setelah kalimat itu.\n\
- Kalau kutipan tidak memuat jawabannya, katakan dengan jujur bahwa bukunya tidak membahas hal \
itu, tanpa menebak.\n\
- Pakai bahasa Indonesia yang sederhana dan ramah, sesuai kelas siswa. Jawab singkat: 2 sampai 5 \
kalimat, kecuali siswa minta penjelasan panjang.\n\
- Rumus matematika ditulis dengan LaTeX di antara $ ... $.";

/// The messages for one answer: belajar's own profile (its system message), the rules and the
/// passages, the recent turns, then the question.
pub fn answer_messages(
    profile: Option<&str>,
    history: &[(String, String)],
    question: &str,
    passages: &[Passage],
) -> Vec<Message> {
    let mut system = String::new();
    if let Some(profile) = profile {
        system.push_str(profile.trim());
        system.push_str("\n\n");
    }
    system.push_str(RULES);
    system.push_str("\n\nKutipan buku:\n\n");
    system.push_str(&context_block(passages));
    let mut messages = vec![Message {
        role: "system",
        content: system,
    }];
    for (role, content) in history {
        let role = if role == "assistant" {
            "assistant"
        } else {
            "user"
        };
        messages.push(Message {
            role,
            content: content.clone(),
        });
    }
    messages.push(Message {
        role: "user",
        content: question.to_owned(),
    });
    messages
}

/// How many characters of earlier turns travel with a question: enough to follow a
/// conversation, little enough to leave the books their room.
pub const HISTORY_CHARS: usize = 3_000;

/// The last turns that fit HISTORY_CHARS, oldest first.
pub fn recent_history(turns: &[(String, String)]) -> Vec<(String, String)> {
    let mut used = 0;
    let mut out = Vec::new();
    for (role, content) in turns.iter().rev() {
        let len = content.chars().count();
        if used + len > HISTORY_CHARS {
            break;
        }
        used += len;
        out.push((role.clone(), content.clone()));
    }
    out.reverse();
    out
}

/// The practice prompt: `count` questions of one kind on the topics, from the passages only.
pub fn practice_messages(
    grade: u8,
    topics: &str,
    kind: &str,
    count: usize,
    passages: &[Passage],
) -> Vec<Message> {
    let shape = match kind {
        "multiple" => {
            "Setiap soal punya 4 pilihan (A-D) dengan TEPAT 2 jawaban benar. Isi \"answer\" \
             dengan kedua label yang benar. Kosongkan \"story\"."
        }
        "story" => {
            "Setiap soal diawali cerita pendek 2-3 kalimat tentang kejadian sehari-hari di \
             \"story\", lalu pertanyaan tentang cerita itu. 4 pilihan (A-D), tepat 1 jawaban \
             benar."
        }
        _ => {
            "Setiap soal punya 4 pilihan (A-D) dengan tepat 1 jawaban benar. Kosongkan \
             \"story\"."
        }
    };
    let system = format!(
        "Kamu guru yang membuat soal latihan pilihan ganda untuk siswa kelas {grade}. Buat soal \
         HANYA dari kutipan buku di bawah, dalam bahasa Indonesia yang jelas. {shape} \
         Pilihan yang salah harus masuk akal. \"explanation\" menjelaskan jawaban benar dalam 1-2 \
         kalimat. \"source\" adalah nomor kutipan yang dipakai soal itu.\n\nKutipan buku:\n\n{}",
        context_block(passages)
    );
    vec![
        Message {
            role: "system",
            content: system,
        },
        Message {
            role: "user",
            content: format!("Buat {count} soal tentang: {topics}."),
        },
    ]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "tests")]

    use super::*;

    #[test]
    fn the_profile_rules_and_books_come_first_and_the_question_last() {
        let messages = answer_messages(
            Some("Kamu adalah guru pendamping untuk Budi, siswa kelas 7."),
            &[
                ("user".into(), "halo".into()),
                ("assistant".into(), "hai".into()),
            ],
            "apa itu kalor?",
            &[],
        );
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].role, "system");
        assert!(
            messages[0]
                .content
                .starts_with("Kamu adalah guru pendamping")
        );
        assert!(
            messages[0]
                .content
                .contains("Jawab HANYA dari kutipan buku")
        );
        assert_eq!(messages[3].content, "apa itu kalor?");
    }

    #[test]
    fn only_the_latest_turns_that_fit_are_kept() {
        let long = "x".repeat(HISTORY_CHARS - 10);
        let turns = vec![
            ("user".to_owned(), "lama sekali".to_owned()),
            ("assistant".to_owned(), long.clone()),
            ("user".to_owned(), "baru".to_owned()),
        ];
        let kept = recent_history(&turns);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[1].1, "baru");
    }
}
