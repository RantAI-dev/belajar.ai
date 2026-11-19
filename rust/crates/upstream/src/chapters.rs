//! Placeholder chapter lists per subject, in Indonesian, SMP-plausible. The upstream has no
//! chapters endpoint yet; the client serves this built-in list until it ships one. The same list
//! serves every grade, and a subject we do not know answers with an empty list.

use belajar_ports::upstream::Chapter;

const CHAPTER_TITLES: &[(&str, &[&str])] = &[
    (
        "IPA",
        &[
            "Metode Ilmiah dan Pengukuran",
            "Zat dan Wujudnya",
            "Klasifikasi Makhluk Hidup",
            "Suhu, Kalor, dan Pemuaian",
            "Energi dalam Kehidupan",
        ],
    ),
    (
        "Matematika",
        &[
            "Bilangan Bulat",
            "Bilangan Pecahan",
            "Persamaan Linear Satu Variabel",
            "Perbandingan dan Skala",
            "Sudut dan Garis Sejajar",
        ],
    ),
    (
        "Bahasa Inggris",
        &[
            "Greetings and Introductions",
            "Nouns and Pronouns",
            "Simple Present Tense",
            "Family and Daily Activities",
            "Describing Places",
        ],
    ),
    (
        "Bahasa Indonesia",
        &[
            "Teks Deskripsi",
            "Teks Narasi",
            "Teks Eksposisi",
            "Puisi dan Pantun",
            "Kalimat Efektif",
        ],
    ),
    (
        "IPS",
        &[
            "Peta dan Denah",
            "Interaksi Sosial",
            "Kondisi Geografis Indonesia",
            "Kegiatan Ekonomi dan Pasar",
            "Dinamika Penduduk",
        ],
    ),
    (
        "Informatika",
        &[
            "Teknologi Informasi dan Komunikasi",
            "Berpikir Komputasional",
            "Algoritma dan Pemrograman Dasar",
            "Keamanan Data dan Etika Digital",
            "Analisis Data dan Visualisasi",
        ],
    ),
    (
        "Pendidikan Pancasila",
        &[
            "Pancasila sebagai Dasar Negara",
            "Norma dan Keadilan",
            "Keberagaman Masyarakat Indonesia",
            "Gotong Royong dan Musyawarah",
            "Bela Negara",
        ],
    ),
    (
        "Pendidikan Agama Islam",
        &[
            "Iman kepada Allah",
            "Salat dan Ibadah Harian",
            "Akhlak Terpuji",
            "Sejarah Nabi Muhammad",
            "Al-Qur'an dan Hadis",
        ],
    ),
];

pub(crate) fn chapters_for(mapel: &str) -> Vec<Chapter> {
    CHAPTER_TITLES
        .iter()
        .find(|(name, _)| *name == mapel)
        .map(|(_, titles)| {
            titles
                .iter()
                .enumerate()
                .map(|(index, title)| Chapter {
                    id: format!("bab-{}", index + 1),
                    // `u32` is plenty for a chapter number; the table is ours and stays small.
                    number: (index + 1) as u32,
                    title: (*title).to_owned(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    #[test]
    fn every_subject_has_at_least_four_numbered_chapters() {
        for (name, titles) in CHAPTER_TITLES {
            let chapters = chapters_for(name);
            assert!(titles.len() >= 4, "{name} needs at least four chapters");
            for (index, chapter) in chapters.iter().enumerate() {
                assert_eq!(chapter.id, format!("bab-{}", index + 1));
                assert_eq!(chapter.number as usize, index + 1);
                assert!(!chapter.title.is_empty());
            }
        }
    }

    #[test]
    fn an_unknown_subject_answers_with_an_empty_list() {
        assert!(chapters_for("Fisika Kuantum").is_empty());
    }
}
