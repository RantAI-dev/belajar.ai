//! Retrieval for one question: search the knowledge base, rerank, and keep what fits the model's
//! window. The model has about 8k tokens per request; the books get at most CONTEXT_CHARS.

use crate::kb::Chunk;

/// How many passages the knowledge base is asked for; the reranker narrows them down.
pub const SEARCH_CHUNKS: u32 = 12;
/// At most this many passages reach the model.
pub const MAX_PASSAGES: usize = 5;
/// About 5k tokens of book text (Indonesian runs near 3.5 characters per token).
pub const CONTEXT_CHARS: usize = 17_000;
/// One passage never takes more than this, so one long chunk cannot crowd out the rest.
pub const PASSAGE_CHARS: usize = 4_000;

/// A passage chosen for the answer, numbered as the model cites it: `[1]`, `[2]`...
#[derive(Debug, Clone, PartialEq)]
pub struct Passage {
    pub number: u32,
    pub chunk: Chunk,
    pub score: f64,
}

impl Passage {
    /// The page as a reader counts it in the PDF, from 1. The knowledge base counts from 0:
    /// its page 117 of the grade 7 science book is the PDF's 118th page.
    pub fn pdf_page(&self) -> Option<u32> {
        self.chunk.page.map(|p| p + 1)
    }
}

/// Page-layout noise the PDF extraction leaves: carriage returns and runs of blank space.
pub fn clean(text: &str) -> String {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank += 1;
            if blank == 1 {
                out.push('\n');
            }
            continue;
        }
        blank = 0;
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_owned()
}

/// Cut at a character boundary, at most `max` characters.
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(max).collect();
    cut.push('…');
    cut
}

/// The best passages, in score order, numbered from 1, within the character budget. Two
/// passages from the same page of the same book count once: the second adds nothing new.
pub fn select(chunks: Vec<Chunk>, scores: &[f64]) -> Vec<Passage> {
    let mut ranked: Vec<(Chunk, f64)> = chunks
        .into_iter()
        .zip(
            scores
                .iter()
                .copied()
                .chain(std::iter::repeat(f64::NEG_INFINITY)),
        )
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut used = 0;
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for (mut chunk, score) in ranked {
        if out.len() >= MAX_PASSAGES {
            break;
        }
        if !seen.insert((chunk.document_id.clone(), chunk.page)) {
            continue;
        }
        chunk.content = truncate(&clean(&chunk.content), PASSAGE_CHARS);
        let len = chunk.content.chars().count();
        if used + len > CONTEXT_CHARS && !out.is_empty() {
            continue;
        }
        used += len;
        out.push(Passage {
            number: out.len() as u32 + 1,
            chunk,
            score,
        });
    }
    out
}

/// The book passages as the model reads them.
pub fn context_block(passages: &[Passage]) -> String {
    passages
        .iter()
        .map(|p| {
            let page = p
                .pdf_page()
                .map(|n| format!(", halaman {n}"))
                .unwrap_or_default();
            format!(
                "[{}] {}{page}\n{}",
                p.number, p.chunk.document_title, p.chunk.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Which `[n]` the answer actually cites.
pub fn cited_numbers(answer: &str) -> std::collections::BTreeSet<u32> {
    let mut out = std::collections::BTreeSet::new();
    let bytes = answer.as_bytes();
    let mut i = 0;
    while let Some(rel) = answer.get(i..).and_then(|rest| rest.find('[')) {
        let start = i + rel + 1;
        let digits: String = answer
            .get(start..)
            .unwrap_or_default()
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let end = start + digits.len();
        if !digits.is_empty()
            && bytes.get(end) == Some(&b']')
            && let Ok(n) = digits.parse::<u32>()
        {
            out.insert(n);
        }
        i = start;
    }
    out
}

/// The search query for a follow-up: a short question ("kenapa?") leans on the one before it.
pub fn search_query(question: &str, previous_question: Option<&str>) -> String {
    let words = question.split_whitespace().count();
    match previous_question {
        Some(prev) if words < 5 => format!("{prev} {question}"),
        _ => question.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "tests")]

    use super::*;

    fn chunk(doc: &str, page: u32, text: &str) -> Chunk {
        Chunk {
            content: text.into(),
            document_id: doc.into(),
            document_title: format!("Buku {doc}"),
            section: None,
            page: Some(page),
            retrieval: None,
        }
    }

    #[test]
    fn the_best_passages_are_kept_numbered_and_one_per_page() {
        let chunks = vec![
            chunk("a", 1, "satu"),
            chunk("a", 2, "dua"),
            chunk("a", 2, "dua lagi"),
            chunk("b", 9, "tiga"),
        ];
        let picked = select(chunks, &[0.1, 0.9, 0.8, 0.5]);
        let texts: Vec<&str> = picked.iter().map(|p| p.chunk.content.as_str()).collect();
        assert_eq!(texts, ["dua", "tiga", "satu"]);
        let numbers: Vec<u32> = picked.iter().map(|p| p.number).collect();
        assert_eq!(numbers, [1, 2, 3]);
    }

    #[test]
    fn the_context_stays_within_its_budget() {
        let long = "x".repeat(PASSAGE_CHARS * 2);
        let chunks = (0..8).map(|i| chunk("a", i, &long)).collect();
        let picked = select(chunks, &[1.0; 8]);
        let total: usize = picked.iter().map(|p| p.chunk.content.chars().count()).sum();
        assert!(total <= CONTEXT_CHARS);
        assert!(
            picked
                .iter()
                .all(|p| p.chunk.content.chars().count() <= PASSAGE_CHARS + 1)
        );
    }

    #[test]
    fn pages_are_counted_from_one_for_the_reader() {
        let p = Passage {
            number: 1,
            chunk: chunk("a", 117, "x"),
            score: 1.0,
        };
        assert_eq!(p.pdf_page(), Some(118));
    }

    #[test]
    fn extraction_noise_is_cleaned() {
        assert_eq!(clean("a \r\nb\r\n\r\n\r\n\r\nc  "), "a\nb\n\nc");
    }

    #[test]
    fn cited_numbers_are_found() {
        let cited = cited_numbers("Rel memuai [1]. Juga [3][2]. Bukan [x] atau [4");
        assert_eq!(cited.into_iter().collect::<Vec<_>>(), [1, 2, 3]);
    }

    #[test]
    fn a_short_follow_up_searches_with_the_question_before_it() {
        assert_eq!(
            search_query("kenapa begitu?", Some("apa itu pemuaian")),
            "apa itu pemuaian kenapa begitu?"
        );
        assert_eq!(
            search_query("bagaimana cara kerja termometer raksa", Some("x")),
            "bagaimana cara kerja termometer raksa"
        );
    }
}
