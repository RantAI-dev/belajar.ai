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

/// The sources of an answer: the first cited, the second not.
pub(crate) fn sources_json(sources: &[SeedSource], base_url: &str) -> Value {
    Value::Array(
        sources
            .iter()
            .enumerate()
            .map(|(index, source)| source_json(source, index as u32 + 1, index == 0, base_url))
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
