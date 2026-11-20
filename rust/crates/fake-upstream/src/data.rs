//! The fake's small world: the grade's subjects, and how it picks an answer from a question.

use serde_json::{Value, json};

/// A book passage an answer or question cites; the handler turns it into the wire source object.
#[derive(Clone)]
pub(crate) struct SeedSource {
    pub(crate) title: &'static str,
    pub(crate) section: &'static str,
    pub(crate) document_id: &'static str,
    pub(crate) page: u32,
}

/// One source object on the wire, exactly as the upstream spells its fields (mixed case on
/// purpose: `documentId` beside `page_url`).
pub(crate) fn source_json(source: &SeedSource, number: u32, cited: bool, base_url: &str) -> Value {
    json!({
        "title": source.title,
        "section": source.section,
        "documentId": source.document_id,
        "page": source.page,
        "number": number,
        "cited": cited,
        "page_url": format!("{base_url}/pages/{}/{}.svg?sig=fake", source.document_id, source.page),
    })
}

/// The sources of an answer. A source is cited when the answer text marks it `[n]`, like
/// the real upstream; the others were retrieved but not used.
pub(crate) fn sources_json(sources: &[SeedSource], text: &str, base_url: &str) -> Value {
    Value::Array(
        sources
            .iter()
            .zip(1u32..)
            .map(|(source, number)| {
                let cited = text.contains(&format!("[{number}]"));
                source_json(source, number, cited, base_url)
            })
            .collect(),
    )
}

/// The 8 subjects a grade's knowledge base covers, each with documents in it.
pub(crate) const MAPEL: &[(&str, &str, u32)] = &[
    ("IPA", "kb-ipa-7", 12),
    ("Matematika", "kb-matematika-7", 14),
    ("Bahasa Inggris", "kb-inggris-7", 10),
    ("Bahasa Indonesia", "kb-bahasa-indonesia-7", 11),
    ("IPS", "kb-ips-7", 13),
    ("Informatika", "kb-informatika-7", 8),
    ("Pendidikan Pancasila", "kb-pancasila-7", 6),
    ("Pendidikan Agama Islam", "kb-pai-7", 7),
];

pub(crate) fn mapel_json() -> Value {
    Value::Array(
        MAPEL
            .iter()
            .map(|(name, knowledge_base_id, document_count)| {
                json!({
                    "mapel": name,
                    "knowledge_base_id": knowledge_base_id,
                    "document_count": document_count,
                })
            })
            .collect(),
    )
}

/// Questions about things no schoolbook covers get a short kind refusal, not a made-up answer.
const OUT_OF_BOOK: &[&str] = &[
    "piala dunia",
    "presiden",
    "artis",
    "game",
    "sepak bola",
    "sepakbola",
];

const REFUSAL: &str = "Hmm, itu tidak ada di buku pelajaranmu. Aku hanya bisa menjawab pertanyaan \
    tentang materi sekolah, seperti IPA, Matematika, atau Bahasa Inggris. Coba tanyakan hal lain \
    dari bukumu, ya.";

const IPA: &str = "IPA Buku Siswa Kelas VII";
const MAT: &str = "Matematika Buku Siswa Kelas VII";
const ENG: &str = "English for Nusantara Kelas VII";

const fn source(
    title: &'static str,
    section: &'static str,
    document_id: &'static str,
    page: u32,
) -> SeedSource {
    SeedSource {
        title,
        section,
        document_id,
        page,
    }
}

struct Topic {
    keywords: &'static [&'static str],
    text: &'static str,
    sources: [SeedSource; 2],
}

const TOPICS: &[Topic] = &[
    Topic {
        keywords: &["rel", "memuai", "pemuaian"],
        text: "Rel kereta api diberi celah karena baja akan memuai ketika cuaca panas, sehingga \
            relnya bertambah panjang [1]. Kalau tidak ada celah, rel bisa melengkung dan berbahaya \
            untuk kereta yang lewat [2]. Pemuaian artinya ukuran benda bertambah karena suhunya \
            naik.",
        sources: [
            source(IPA, "Bab 5: Suhu, Kalor, dan Pemuaian", "kb-ipa-7", 78),
            source(IPA, "Bab 5: Suhu, Kalor, dan Pemuaian", "kb-ipa-7", 81),
        ],
    },
    Topic {
        keywords: &["suhu", "termometer"],
        text: "Suhu adalah ukuran derajat panas atau dinginnya suatu benda [1]. Alat untuk \
            mengukur suhu adalah termometer, dan satuan yang sering dipakai di sekolah adalah \
            derajat Celsius [2]. Air membeku pada suhu 0 derajat Celsius dan mendidih pada 100 \
            derajat Celsius.",
        sources: [
            source(IPA, "Bab 5: Suhu, Kalor, dan Pemuaian", "kb-ipa-7", 70),
            source(IPA, "Bab 5: Suhu, Kalor, dan Pemuaian", "kb-ipa-7", 72),
        ],
    },
    Topic {
        keywords: &["fotosintesis"],
        text: "Fotosintesis adalah proses tumbuhan hijau membuat makanan sendiri dengan bantuan \
            cahaya matahari [1]. Bahan yang dibutuhkan adalah air dan karbon dioksida, sedangkan \
            hasilnya adalah glukosa dan oksigen [2]. Proses ini berlangsung di dalam kloroplas, \
            bagian sel daun yang mengandung klorofil.",
        sources: [
            source(IPA, "Bab 6: Fotosintesis pada Tumbuhan", "kb-ipa-7", 132),
            source(IPA, "Bab 6: Fotosintesis pada Tumbuhan", "kb-ipa-7", 135),
        ],
    },
    // Maths and a diagram, the way the real API writes them: LaTeX between dollars, mermaid fences.
    Topic {
        keywords: &["jumlahkan pecahan"],
        text: "Kalau penyebutnya sama, kita cukup menjumlahkan pembilangnya [1]. Jadi \
            $ \\frac{1}{4} + \\frac{3}{4} = \\frac{4}{4} = 1 $, artinya satu utuh [2]. Kalau \
            harganya Rp5.000 dan $3, itu uang, bukan rumus.",
        sources: [
            source(MAT, "Bab 2: Bilangan Pecahan", "kb-matematika-7", 44),
            source(MAT, "Bab 2: Bilangan Pecahan", "kb-matematika-7", 45),
        ],
    },
    Topic {
        keywords: &["rumus abc"],
        text: "Akar persamaan kuadrat $ax^2 + bx + c = 0$ dicari dengan rumus abc [1]: \
            $$x_{1,2} = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a} \\quad \\text{dengan} \\quad \
            D = b^2 - 4ac,\\; a \\neq 0,\\; x_1 + x_2 = -\\frac{b}{a},\\; \
            x_1 \\cdot x_2 = \\frac{c}{a}$$ Nilai $D$ disebut diskriminan [2].",
        sources: [
            source(MAT, "Bab 6: Persamaan Kuadrat", "kb-matematika-7", 120),
            source(MAT, "Bab 6: Persamaan Kuadrat", "kb-matematika-7", 122),
        ],
    },
    Topic {
        keywords: &["siklus air"],
        text: "Air di bumi terus berputar dalam siklus air [1]:\n```mermaid\nflowchart LR\n  \
            A[Air laut] -->|menguap| B[Uap air]\n  B -->|mengembun| C[Awan]\n  \
            C -->|hujan| D[Sungai]\n  D --> A\n```\nPanas matahari membuat air menguap [2].",
        sources: [
            source(IPA, "Bab 7: Siklus Air", "kb-ipa-7", 150),
            source(IPA, "Bab 7: Siklus Air", "kb-ipa-7", 152),
        ],
    },
    Topic {
        keywords: &["pecahan"],
        text: "Pecahan menunjukkan berapa bagian dari keseluruhan yang kita ambil [1]. Angka atas \
            disebut pembilang dan angka bawah disebut penyebut [2]. Contohnya, tiga perempat \
            artinya kita mengambil tiga bagian dari empat bagian yang sama besar.",
        sources: [
            source(MAT, "Bab 2: Bilangan Pecahan", "kb-matematika-7", 40),
            source(MAT, "Bab 2: Bilangan Pecahan", "kb-matematika-7", 43),
        ],
    },
    Topic {
        keywords: &["persamaan"],
        text: "Persamaan linear satu variabel adalah persamaan dengan satu peubah dan pangkat \
            tertingginya satu [1]. Contohnya, pada persamaan x + 3 = 7 nilainya x = 4 [2]. Cara \
            menyelesaikannya adalah dengan memindahkan bilangan ke sisi lain tanda sama dengan.",
        sources: [
            source(
                MAT,
                "Bab 4: Persamaan Linear Satu Variabel",
                "kb-matematika-7",
                92,
            ),
            source(
                MAT,
                "Bab 4: Persamaan Linear Satu Variabel",
                "kb-matematika-7",
                95,
            ),
        ],
    },
    Topic {
        keywords: &["noun", "verb", "kata benda", "kata kerja"],
        text: "A noun is a word for a person, place, or thing, for example teacher, school, and \
            book [1]. A verb shows an action, for example read, play, and study [2]. Dalam bahasa \
            Indonesia, noun artinya kata benda dan verb artinya kata kerja.",
        sources: [
            source(ENG, "Unit 2: Nouns and Pronouns", "kb-inggris-7", 24),
            source(ENG, "Unit 3: Verbs and Tenses", "kb-inggris-7", 31),
        ],
    },
];

const FALLBACK_SOURCES: [SeedSource; 2] = [
    source("Buku Pelajaran Kelas VII", "Ringkasan Bab", "kb-umum-7", 12),
    source("Buku Pelajaran Kelas VII", "Ringkasan Bab", "kb-umum-7", 15),
];

pub(crate) enum Answer {
    Grounded {
        text: String,
        sources: [SeedSource; 2],
    },
    Ungrounded {
        text: String,
    },
}

/// Picks the answer for one question: a refusal when it is not in any schoolbook, a canned
/// grounded answer for the keywords we know, and a generic grounded fallback mentioning the mapel.
pub(crate) fn answer_for(question: &str, mapel: Option<&str>) -> Answer {
    let question = question.to_lowercase();
    if OUT_OF_BOOK.iter().any(|keyword| question.contains(keyword)) {
        return Answer::Ungrounded {
            text: REFUSAL.to_owned(),
        };
    }
    for topic in TOPICS {
        if topic
            .keywords
            .iter()
            .any(|keyword| question.contains(keyword))
        {
            return Answer::Grounded {
                text: topic.text.to_owned(),
                sources: topic.sources.clone(),
            };
        }
    }
    let subject = mapel.unwrap_or("buku pelajaranmu");
    Answer::Grounded {
        text: format!(
            "Menurut buku {subject} kelas 7, materi seperti ini dijelaskan dengan \
            contoh-contoh di bab yang sedang kamu pelajari [1]. Coba buka babnya dulu, lalu \
            tanyakan bagian mana yang masih membingungkan [2]."
        ),
        sources: FALLBACK_SOURCES.clone(),
    }
}

/// The fake's grounded answers always cite, so the client's chat test can fetch a page.
pub(crate) fn grounded_score() -> f64 {
    0.87
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]

    use super::*;

    #[test]
    fn the_demo_questions_route_to_their_topics() {
        let answer = answer_for("Kenapa rel kereta api diberi celah?", Some("IPA"));
        let Answer::Grounded { text, .. } = answer else {
            panic!("the rel question is grounded");
        };
        assert!(text.contains("memuai"));

        let answer = answer_for("Alat untuk mengukur suhu adalah apa?", None);
        assert!(matches!(answer, Answer::Grounded { .. }));

        let answer = answer_for("Jelaskan fotosintesis pada tumbuhan!", Some("IPA"));
        let Answer::Grounded { text, .. } = answer else {
            panic!("fotosintesis is grounded");
        };
        assert!(text.contains("kloroplas"));
    }

    #[test]
    fn maths_and_diagram_answers_keep_their_markup() {
        let text = |question| match answer_for(question, Some("Matematika")) {
            Answer::Grounded { text, .. } => text,
            Answer::Ungrounded { text } => text,
        };
        assert!(text("Jumlahkan pecahan 1/4 dan 3/4").contains("$ \\frac{1}{4}"));
        assert!(text("Apa itu rumus abc?").contains("$$x_{1,2}"));
        assert!(text("Jelaskan siklus air").contains("\n```mermaid\nflowchart LR\n"));
    }

    #[test]
    fn questions_outside_the_books_are_kindly_refused() {
        for question in [
            "Siapa juara Piala Dunia 2022?",
            "Siapa presiden sekarang?",
            "Apa game favorit anak artis itu?",
        ] {
            let Answer::Ungrounded { text } = answer_for(question, None) else {
                panic!("{question} should be refused");
            };
            assert!(text.contains("tidak ada di buku pelajaranmu"));
        }
    }

    #[test]
    fn the_fallback_answer_mentions_the_mapel_and_stays_grounded() {
        let Answer::Grounded { text, sources } =
            answer_for("Bagaimana cara membuat karung beras?", Some("IPS"))
        else {
            panic!("an unknown school question still gets the grounded fallback");
        };
        assert!(text.contains("IPS"));
        assert_eq!(sources[0].document_id, "kb-umum-7");
    }

    #[test]
    fn the_subjects_are_exactly_the_eight_named_ones() {
        let names: Vec<&str> = MAPEL.iter().map(|(name, _, _)| *name).collect();
        assert_eq!(
            names,
            vec![
                "IPA",
                "Matematika",
                "Bahasa Inggris",
                "Bahasa Indonesia",
                "IPS",
                "Informatika",
                "Pendidikan Pancasila",
                "Pendidikan Agama Islam",
            ]
        );
        assert!(MAPEL.iter().all(|(_, _, count)| *count > 0));
    }
}

#[cfg(test)]
mod cited_tests {
    use super::*;

    const PAGE: SeedSource = SeedSource {
        title: "IPA",
        section: "Bab 5",
        document_id: "kb-ipa-7",
        page: 78,
    };

    #[test]
    fn a_source_is_cited_exactly_when_the_text_marks_its_number() {
        let json = sources_json(&[PAGE, PAGE, PAGE], "satu [1], dua [2].", "http://f");
        let cited: Option<Vec<Option<bool>>> = json.as_array().map(|all| {
            all.iter()
                .map(|s| s.get("cited").and_then(Value::as_bool))
                .collect()
        });
        assert_eq!(cited, Some(vec![Some(true), Some(true), Some(false)]));
    }
}
