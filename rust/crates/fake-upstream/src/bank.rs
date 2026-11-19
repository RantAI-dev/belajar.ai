//! The fake's practice bank: at least twelve Indonesian questions of each kind, SMP-plausible,
//! spread over IPA topics (suhu, kalor, pemuaian, fotosintesis, ekosistem, pengukuran, zat).
//! The topic text selects nothing here yet — the bank is varied by topic in its seeds.

use serde_json::{Value, json};

use crate::data::{SeedSource, source_json};

pub(crate) struct Seed {
    story: Option<&'static str>,
    stem: &'static str,
    options: &'static [(&'static str, &'static str)],
    answer: &'static [&'static str],
    explanation: &'static str,
    source: Option<&'static SeedSource>,
}

const S_UKUR: SeedSource = SeedSource {
    title: "IPA Buku Siswa Kelas VII",
    section: "Bab 1: Metode Ilmiah dan Pengukuran",
    document_id: "kb-ipa-7",
    page: 12,
};

const S_ZAT: SeedSource = SeedSource {
    title: "IPA Buku Siswa Kelas VII",
    section: "Bab 2: Zat dan Wujudnya",
    document_id: "kb-ipa-7",
    page: 34,
};

const S_SUHU: SeedSource = SeedSource {
    title: "IPA Buku Siswa Kelas VII",
    section: "Bab 5: Suhu, Kalor, dan Pemuaian",
    document_id: "kb-ipa-7",
    page: 70,
};

const S_FOTO: SeedSource = SeedSource {
    title: "IPA Buku Siswa Kelas VII",
    section: "Bab 6: Fotosintesis pada Tumbuhan",
    document_id: "kb-ipa-7",
    page: 132,
};

const S_EKO: SeedSource = SeedSource {
    title: "IPA Buku Siswa Kelas VII",
    section: "Bab 7: Ekosistem",
    document_id: "kb-ipa-7",
    page: 156,
};

/// Exactly one correct option.
const SINGLE: &[Seed] = &[
    Seed {
        story: None,
        stem: "Alat yang tepat untuk mengukur suhu air panas adalah ...",
        options: &[
            ("A", "Termometer"),
            ("B", "Barometer"),
            ("C", "Higrometer"),
            ("D", "Anemometer"),
        ],
        answer: &["A"],
        explanation: "Suhu diukur dengan termometer; barometer mengukur tekanan udara, higrometer \
            kelembapan, dan anemometer kecepatan angin.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: None,
        stem: "Bertambahnya ukuran suatu benda karena kenaikan suhu disebut ...",
        options: &[
            ("A", "Penguapan"),
            ("B", "Pemuaian"),
            ("C", "Pengembunan"),
            ("D", "Penyubliman"),
        ],
        answer: &["B"],
        explanation: "Pemuaian adalah bertambahnya ukuran benda saat dipanaskan; penguapan dan \
            penyubliman adalah perubahan wujud zat.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: None,
        stem: "Perpindahan kalor melalui zat perantara tanpa disertai perpindahan partikelnya \
            disebut ...",
        options: &[
            ("A", "Konveksi"),
            ("B", "Radiasi"),
            ("C", "Konduksi"),
            ("D", "Evaporasi"),
        ],
        answer: &["C"],
        explanation: "Pada konduksi, kalor berpindah melalui zat perantara sedangkan partikelnya \
            tidak ikut berpindah, contohnya besi yang dipanaskan di salah satu ujungnya.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: None,
        stem: "Proses tumbuhan hijau membuat makanan sendiri dengan bantuan cahaya matahari \
            disebut ...",
        options: &[
            ("A", "Respirasi"),
            ("B", "Fotosintesis"),
            ("C", "Transpirasi"),
            ("D", "Fermentasi"),
        ],
        answer: &["B"],
        explanation: "Fotosintesis mengubah air dan karbon dioksida menjadi glukosa dan oksigen \
            dengan energi dari cahaya matahari.",
        source: Some(&S_FOTO),
    },
    Seed {
        story: None,
        stem: "Bagian sel tumbuhan tempat berlangsungnya fotosintesis adalah ...",
        options: &[
            ("A", "Mitokondria"),
            ("B", "Vakuola"),
            ("C", "Ribosom"),
            ("D", "Kloroplas"),
        ],
        answer: &["D"],
        explanation: "Kloroplas mengandung klorofil, zat hijau daun yang menangkap energi cahaya \
            untuk fotosintesis.",
        source: Some(&S_FOTO),
    },
    Seed {
        story: None,
        stem: "Gas yang dihasilkan fotosintesis dan dilepaskan ke udara adalah ...",
        options: &[
            ("A", "Karbon dioksida"),
            ("B", "Oksigen"),
            ("C", "Nitrogen"),
            ("D", "Hidrogen"),
        ],
        answer: &["B"],
        explanation: "Fotosintesis menghasilkan glukosa yang dipakai tumbuhan dan oksigen yang \
            dilepaskan ke udara.",
        source: Some(&S_FOTO),
    },
    Seed {
        story: None,
        stem: "Satuan panjang dalam Sistem Internasional (SI) adalah ...",
        options: &[
            ("A", "Kilometer"),
            ("B", "Sentimeter"),
            ("C", "Meter"),
            ("D", "Milimeter"),
        ],
        answer: &["C"],
        explanation: "Satuan panjang SI adalah meter; kilometer, sentimeter, dan milimeter \
            adalah turunannya.",
        source: Some(&S_UKUR),
    },
    Seed {
        story: None,
        stem: "Kumpulan individu dari spesies yang sama yang hidup di satu tempat disebut ...",
        options: &[
            ("A", "Komunitas"),
            ("B", "Ekosistem"),
            ("C", "Populasi"),
            ("D", "Bioma"),
        ],
        answer: &["C"],
        explanation: "Populasi adalah kumpulan individu sejenis di satu tempat; kumpulan berbagai \
            populasi membentuk komunitas.",
        source: Some(&S_EKO),
    },
    Seed {
        story: None,
        stem: "Hubungan antara lebah dan bunga di kebun adalah contoh interaksi ...",
        options: &[
            ("A", "Kompetisi"),
            ("B", "Mutualisme"),
            ("C", "Predasi"),
            ("D", "Parasitisme"),
        ],
        answer: &["B"],
        explanation: "Lebah mendapat nektar dan bunga terbantu penyerbukannya; kedua pihak diuntungkan.",
        source: Some(&S_EKO),
    },
    Seed {
        story: None,
        stem: "Zat yang bentuk dan volumenya tetap adalah ...",
        options: &[
            ("A", "Zat padat"),
            ("B", "Zat cair"),
            ("C", "Zat gas"),
            ("D", "Plasma"),
        ],
        answer: &["A"],
        explanation: "Partikel zat padat tersusun rapat dan teratur sehingga bentuk dan \
            volumenya tetap.",
        source: Some(&S_ZAT),
    },
    Seed {
        story: None,
        stem: "Perubahan wujud dari padat langsung menjadi gas disebut ...",
        options: &[
            ("A", "Mencair"),
            ("B", "Mengembun"),
            ("C", "Menyublim"),
            ("D", "Membeku"),
        ],
        answer: &["C"],
        explanation: "Menyublim adalah perubahan wujud padat menjadi gas, misalnya kapur barus \
            yang lama-lama habis.",
        source: Some(&S_ZAT),
    },
    Seed {
        story: None,
        stem: "Banyaknya kalor yang dibutuhkan untuk menaikkan suhu 1 kg zat sebesar 1 derajat \
            Celsius disebut ...",
        options: &[
            ("A", "Kalor jenis"),
            ("B", "Kalor lebur"),
            ("C", "Kalor uap"),
            ("D", "Suhu"),
        ],
        answer: &["A"],
        explanation: "Kalor jenis menyatakan kalor yang dibutuhkan tiap satu kilogram zat untuk \
            naik satu derajat Celsius.",
        source: Some(&S_SUHU),
    },
];

/// Two correct options per question.
const MULTIPLE: &[Seed] = &[
    Seed {
        story: None,
        stem: "Pernyataan yang benar tentang pemuaian adalah ...",
        options: &[
            ("A", "Terjadi ketika benda dipanaskan"),
            ("B", "Hanya terjadi pada gas"),
            ("C", "Ukuran benda bertambah"),
            ("D", "Tidak dipengaruhi suhu"),
        ],
        answer: &["A", "C"],
        explanation: "Pemuaian terjadi pada padat, cair, dan gas; semakin tinggi suhu, semakin \
            besar pemuaiannya.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: None,
        stem: "Berikut ini yang termasuk cara perpindahan kalor adalah ...",
        options: &[
            ("A", "Konduksi"),
            ("B", "Konveksi"),
            ("C", "Reduksi"),
            ("D", "Refraksi"),
        ],
        answer: &["A", "B"],
        explanation: "Kalor berpindah secara konduksi, konveksi, dan radiasi; reduksi dan \
            refraksi bukan perpindahan kalor.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: None,
        stem: "Faktor yang memengaruhi laju fotosintesis adalah ...",
        options: &[
            ("A", "Intensitas cahaya"),
            ("B", "Ketersediaan air"),
            ("C", "Warna pot tanaman"),
            ("D", "Banyaknya meja di kelas"),
        ],
        answer: &["A", "B"],
        explanation: "Cahaya dan air adalah bahan serta energi fotosintesis, sehingga keduanya \
            paling memengaruhi lajunya.",
        source: Some(&S_FOTO),
    },
    Seed {
        story: None,
        stem: "Yang termasuk perubahan wujud zat adalah ...",
        options: &[
            ("A", "Mencair"),
            ("B", "Menyublim"),
            ("C", "Memuai"),
            ("D", "Menggulung"),
        ],
        answer: &["A", "B"],
        explanation: "Mencair dan menyublim mengubah wujud zat; memuai hanya mengubah ukuran, dan \
            menggulung bukan perubahan zat.",
        source: Some(&S_ZAT),
    },
    Seed {
        story: None,
        stem: "Ciri-ciri zat cair adalah ...",
        options: &[
            ("A", "Bentuknya mengikuti wadah"),
            ("B", "Volumenya tetap"),
            ("C", "Partikelnya sangat rapat dan teratur"),
            ("D", "Tidak dapat mengalir"),
        ],
        answer: &["A", "B"],
        explanation: "Zat cair bentuknya berubah mengikuti wadah tetapi volumenya tetap; \
            partikelnya tidak teratur seperti zat padat.",
        source: Some(&S_ZAT),
    },
    Seed {
        story: None,
        stem: "Contoh penerapan pemuaian dalam kehidupan sehari-hari adalah ...",
        options: &[
            ("A", "Celah pada rel kereta api"),
            (
                "B",
                "Memasang cincin logam pada roda dengan cara dipanaskan",
            ),
            ("C", "Menyimpan es di dalam kulkas"),
            ("D", "Mencampur air dengan sirup"),
        ],
        answer: &["A", "B"],
        explanation: "Keduanya memanfaatkan pemuaian dan penyusutan logam; dua sisanya bukan \
            penerapan pemuaian.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: None,
        stem: "Termometer yang biasa digunakan di laboratorium sekolah adalah ...",
        options: &[
            ("A", "Termometer raksa"),
            ("B", "Termometer alkohol"),
            ("C", "Barometer"),
            ("D", "Higrometer"),
        ],
        answer: &["A", "B"],
        explanation: "Termometer laboratorium diisi raksa atau alkohol; barometer mengukur tekanan \
            dan higrometer kelembapan.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: None,
        stem: "Hasil fotosintesis adalah ...",
        options: &[
            ("A", "Glukosa"),
            ("B", "Oksigen"),
            ("C", "Karbon dioksida"),
            ("D", "Metana"),
        ],
        answer: &["A", "B"],
        explanation: "Glukosa menjadi cadangan makanan tumbuhan dan oksigen dilepaskan ke udara; \
            karbon dioksida justru bahan fotosintesis.",
        source: Some(&S_FOTO),
    },
    Seed {
        story: None,
        stem: "Bagian tumbuhan yang berperan langsung dalam fotosintesis adalah ...",
        options: &[
            ("A", "Daun"),
            ("B", "Klorofil"),
            ("C", "Akar"),
            ("D", "Bunga"),
        ],
        answer: &["A", "B"],
        explanation: "Fotosintesis berlangsung di daun, tepatnya di klorofil di dalam kloroplas.",
        source: Some(&S_FOTO),
    },
    Seed {
        story: None,
        stem: "Kegiatan berikut yang termasuk pengukuran adalah ...",
        options: &[
            ("A", "Menimbang berat badan dengan neraca"),
            ("B", "Mengukur panjang meja dengan penggaris"),
            ("C", "Mengamati warna daun"),
            ("D", "Mendengarkan lagu"),
        ],
        answer: &["A", "B"],
        explanation: "Pengukuran membandingkan besaran dengan alat ukur dan satuan baku.",
        source: Some(&S_UKUR),
    },
    Seed {
        story: None,
        stem: "Pernyataan yang benar tentang populasi adalah ...",
        options: &[
            ("A", "Terdiri atas individu sejenis"),
            ("B", "Hidup di tempat yang sama"),
            ("C", "Selalu terdiri atas satu individu"),
            ("D", "Hanya berlaku pada tumbuhan"),
        ],
        answer: &["A", "B"],
        explanation: "Populasi adalah kumpulan individu sejenis di satu tempat, pada tumbuhan \
            maupun hewan.",
        source: Some(&S_EKO),
    },
    Seed {
        story: None,
        stem: "Peristiwa berikut yang menunjukkan perpindahan kalor secara konveksi adalah ...",
        options: &[
            ("A", "Air panas bergerak naik di dalam panci"),
            ("B", "Angin darat dan angin laut"),
            ("C", "Besi yang dipanaskan di salah satu ujungnya"),
            ("D", "Cahaya matahari sampai ke bumi"),
        ],
        answer: &["A", "B"],
        explanation: "Konveksi adalah perpindahan kalor bersama aliran zatnya; besi adalah \
            konduksi dan cahaya matahari adalah radiasi.",
        source: Some(&S_SUHU),
    },
];

/// A short story before a single-answer question.
const STORY: &[Seed] = &[
    Seed {
        story: Some(
            "Pak Rani menjemur sepinggan besi di teras rumah dari pagi hingga siang. \
            Sore hari, pinggan itu terasa panas dan tepinya sedikit lebih lebar daripada pagi \
            tadi.",
        ),
        stem: "Perubahan yang dialami pinggan besi Pak Rani terjadi karena ...",
        options: &[
            ("A", "Pemuaian karena kenaikan suhu"),
            ("B", "Pengembunan karena pendinginan"),
            ("C", "Perubahan wujud menjadi cair"),
            ("D", "Reaksi kimia dengan udara"),
        ],
        answer: &["A"],
        explanation: "Pinggan menyerap kalor matahari sehingga suhunya naik dan baja memuai.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: Some(
            "Di laboratorium, Bu Ani memasukkan termometer ke dalam gelas berisi air es. \
            Setelah beberapa saat, raksa pada termometer turun dan berhenti tepat di angka nol.",
        ),
        stem: "Suhu air es dalam gelas tersebut adalah ...",
        options: &[
            ("A", "0 derajat Celsius"),
            ("B", "4 derajat Celsius"),
            ("C", "10 derajat Celsius"),
            ("D", "100 derajat Celsius"),
        ],
        answer: &["A"],
        explanation: "Termometer menunjukkan suhu benda yang diukur; air es berada pada titik \
            bekunya, 0 derajat Celsius.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: Some(
            "Dina menanam dua tanaman tomat yang sama. Satu pot diletakkan di tempat \
            terang dan satu lagi di dalam kardus gelap. Setelah dua minggu, tanaman di tempat \
            terang tumbuh lebih subur.",
        ),
        stem: "Tanaman di tempat terang tumbuh lebih subur karena ...",
        options: &[
            ("A", "Cahaya membantu proses fotosintesis"),
            ("B", "Kardus mengandung pupuk alami"),
            ("C", "Tanaman di dalam kardus kekurangan air"),
            ("D", "Tempat gelap lebih lembap"),
        ],
        answer: &["A"],
        explanation: "Cahaya adalah energi fotosintesis; tanpa cahaya, tumbuhan tidak dapat membuat \
            makanan dengan baik.",
        source: Some(&S_FOTO),
    },
    Seed {
        story: Some(
            "Budi menuang air teh panas ke gelas kaca tebal dan gelas tetap utuh. \
            Saat ia menuang air yang sama panasnya ke gelas kaca tipis, gelas itu langsung retak.",
        ),
        stem: "Gelas tipis lebih mudah retak karena ...",
        options: &[
            (
                "A",
                "Kalornya menyebar tidak merata sehingga pemuaian tidak seragam",
            ),
            ("B", "Gelas tipis sama sekali tidak dapat memuai"),
            ("C", "Air panas bereaksi dengan kaca"),
            ("D", "Gelas tipis lebih ringan daripada gelas tebal"),
        ],
        answer: &["A"],
        explanation: "Kaca tipis menghantar kalor lebih cepat sehingga pemuaian antarbagian tidak \
            seragam dan kaca retak.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: Some(
            "Saat berkemah, Rudi menyalakan api unggun. Meskipun ia duduk agak jauh, \
            wajahnya terasa hangat sementara udara panas terlihat bergerak naik di atas api.",
        ),
        stem: "Rasa hangat di wajah Rudi terjadi karena perpindahan kalor secara ...",
        options: &[
            ("A", "Radiasi"),
            ("B", "Konduksi"),
            ("C", "Konveksi"),
            ("D", "Evaporasi"),
        ],
        answer: &["A"],
        explanation: "Wajah terasa hangat tanpa menyentuh api dan tanpa aliran udara panas ke \
            wajah: itulah radiasi, perpindahan kalor tanpa zat perantara.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: Some(
            "Ibu memasak sup dan mengaduknya terus-menerus. Bagian bawah panci lebih \
            cepat panas, tetapi setelah diaduk seluruh sup menjadi hangat merata.",
        ),
        stem: "Mengaduk sup membantu perpindahan kalor secara ...",
        options: &[
            ("A", "Konveksi"),
            ("B", "Konduksi"),
            ("C", "Radiasi"),
            ("D", "Refraksi"),
        ],
        answer: &["A"],
        explanation: "Mengaduk mendorong bagian air yang panas berganti tempat dengan yang \
            dingin, itulah konveksi.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: Some(
            "Di kebun sekolah terdapat banyak tanaman kacang. Tanah di sekitar kacang \
            menjadi lebih subur sehingga rumput di sampingnya ikut tumbuh subur.",
        ),
        stem: "Hubungan antara tanaman kacang dan rumput tersebut menunjukkan interaksi ...",
        options: &[
            ("A", "Mutualisme"),
            ("B", "Kompetisi"),
            ("C", "Predasi"),
            ("D", "Parasitisme"),
        ],
        answer: &["A"],
        explanation: "Kacang menyuburkan tanah yang dipakai bersama rumput, sehingga kedua pihak \
            diuntungkan.",
        source: Some(&S_EKO),
    },
    Seed {
        story: Some(
            "Saat berkunjung ke kebun binatang, Sinta melihat sekumpulan rusa yang sedang \
            mencari rumput bersama-sama di satu padang.",
        ),
        stem: "Kumpulan rusa di padang tersebut disebut ...",
        options: &[
            ("A", "Populasi"),
            ("B", "Komunitas"),
            ("C", "Ekosistem"),
            ("D", "Individu"),
        ],
        answer: &["A"],
        explanation: "Kumpulan individu satu spesies di satu tempat adalah populasi.",
        source: Some(&S_EKO),
    },
    Seed {
        story: Some(
            "Paman memanaskan cincin logam di tungku sebelum memasangkannya ke roda \
            gerabak kayu. Setelah cincin dingin, roda terkunci sangat kuat.",
        ),
        stem: "Cincin dipanaskan terlebih dahulu agar ...",
        options: &[
            (
                "A",
                "Memuai sehingga mudah dipasang, lalu mengunci roda saat menyusut",
            ),
            ("B", "Menyusut sehingga mudah dipasang"),
            ("C", "Berubah warna sehingga lebih indah"),
            ("D", "Tidak berkarat setelah dipasang"),
        ],
        answer: &["A"],
        explanation: "Logam memuai saat dipanaskan dan menyusut saat didinginkan, sehingga cincin \
            mengunci roda dengan kuat.",
        source: Some(&S_SUHU),
    },
    Seed {
        story: Some(
            "Pagi hari yang dingin, kaca jendela kamar Dika berembun di sisi dalam, \
            padahal semalaman tidak hujan.",
        ),
        stem: "Embun pada kaca jendela terbentuk karena ...",
        options: &[
            (
                "A",
                "Uap air di dalam ruangan mengembun pada kaca yang dingin",
            ),
            ("B", "Air hujan masuk melalui celah jendela"),
            ("C", "Kaca jendela menyerap air dari tanah"),
            ("D", "Angin membawa air dari laut"),
        ],
        answer: &["A"],
        explanation: "Uap air di udara menyentuh kaca yang dingin dan berubah menjadi titik air: \
            itulah pengembunan.",
        source: Some(&S_ZAT),
    },
    Seed {
        story: Some(
            "Untuk praktikum, Lina mengukur panjang meja tiga kali dan mendapat 90,2 cm; \
            90,3 cm; dan 90,2 cm.",
        ),
        stem: "Langkah paling tepat untuk mendapatkan hasil pengukuran yang baik adalah ...",
        options: &[
            (
                "A",
                "Mengulang pengukuran beberapa kali lalu menghitung rata-ratanya",
            ),
            ("B", "Mengukur satu kali saja supaya cepat selesai"),
            ("C", "Memperkirakan panjang meja tanpa penggaris"),
            ("D", "Menggunakan tangan sebagai alat ukur"),
        ],
        answer: &["A"],
        explanation: "Pengukuran yang baik diulang beberapa kali; nilai yang paling mendekati \
            benar adalah rata-ratanya.",
        source: Some(&S_UKUR),
    },
    Seed {
        story: Some(
            "Ayu memanaskan es batu di dalam wajan. Esnya mencair, lalu airnya mendidih \
            dan jumlahnya berkurang banyak.",
        ),
        stem: "Urutan perubahan wujud yang dialami es Ayu adalah ...",
        options: &[
            ("A", "Mencair lalu menguap"),
            ("B", "Menyublim lalu mencair"),
            ("C", "Membeku lalu mencair"),
            ("D", "Mengembun lalu menguap"),
        ],
        answer: &["A"],
        explanation: "Es mencair menjadi air, lalu air menguap menjadi gas saat mendidih.",
        source: Some(&S_ZAT),
    },
];

/// The whole practice set, exactly as it travels inside the SSE `result` event.
pub(crate) fn set_json(
    practice_type: &str,
    total: usize,
    mapel: &str,
    topics: &str,
    base_url: &str,
) -> Value {
    let questions: Vec<Value> = (0..total)
        .map(|index| question_json(index, practice_type, base_url))
        .collect();
    json!({
        "practice_id": format!("fake-{}", uuid::Uuid::now_v7()),
        "practice_type": practice_type,
        "mapel": mapel,
        "topic": topics,
        "questions": questions,
    })
}

fn question_json(index: usize, practice_type: &str, base_url: &str) -> Value {
    let (bank, kind) = match practice_type {
        "mcq_multiple_correct" => (MULTIPLE, "mcq_multiple_correct"),
        "mcq_story" => (STORY, "mcq_story"),
        "mcq_mixed" => match index % 3 {
            0 => (SINGLE, "mcq_normal"),
            1 => (MULTIPLE, "mcq_multiple_correct"),
            _ => (STORY, "mcq_story"),
        },
        // Anything unknown serves single questions; the client only sends the four names anyway.
        _ => (SINGLE, "mcq_normal"),
    };
    // The bank is longer than any request, but cycling keeps a huge `total_questions` honest.
    let seed = match bank.iter().cycle().nth(index) {
        Some(seed) => seed,
        None => return Value::Null,
    };
    json!({
        "id": format!("q{}", index + 1),
        "type": kind,
        "story": seed.story,
        "question": seed.stem,
        "options": seed
            .options
            .iter()
            .map(|(label, text)| json!({ "label": label, "text": text }))
            .collect::<Vec<_>>(),
        "answer": seed.answer,
        "explanation": seed.explanation,
        "source": seed
            .source
            .map(|source| source_json(source, 1, true, base_url)),
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic,
        reason = "tests"
    )]

    use super::*;

    #[test]
    fn each_bank_holds_at_least_twelve_questions() {
        assert!(SINGLE.len() >= 12);
        assert!(MULTIPLE.len() >= 12);
        assert!(STORY.len() >= 12);
        assert!(STORY.iter().all(|seed| seed.story.is_some()));
        assert!(MULTIPLE.iter().all(|seed| seed.answer.len() == 2));
        assert!(SINGLE.iter().all(|seed| seed.answer.len() == 1));
    }

    #[test]
    fn every_answer_label_exists_among_the_options() {
        for bank in [SINGLE, MULTIPLE, STORY] {
            for seed in bank {
                for label in seed.answer {
                    assert!(
                        seed.options.iter().any(|(option, _)| option == label),
                        "{stem} misses option {label}",
                        stem = seed.stem
                    );
                }
            }
        }
    }

    #[test]
    fn the_set_has_unique_ids_and_the_requested_number_of_questions() {
        let set = set_json("mcq_mixed", 7, "IPA", "Pemuaian", "http://u");
        let questions = set["questions"].as_array().unwrap();
        assert_eq!(questions.len(), 7);
        let ids: Vec<&str> = questions
            .iter()
            .map(|q| q["id"].as_str().unwrap())
            .collect();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(ids.len(), unique.len());
        let kinds: Vec<&str> = questions
            .iter()
            .map(|q| q["type"].as_str().unwrap())
            .collect();
        assert!(kinds.contains(&"mcq_normal"));
        assert!(kinds.contains(&"mcq_multiple_correct"));
        assert!(kinds.contains(&"mcq_story"));
        assert!(
            questions[0]["source"]["page_url"]
                .as_str()
                .unwrap()
                .starts_with("http://u/pages/")
        );
    }
}
