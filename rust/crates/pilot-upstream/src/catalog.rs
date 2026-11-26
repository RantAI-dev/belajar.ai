//! Which knowledge bases a grade and a subject read from. The knowledge base names its bases
//! "<subject> — Kelas <n>"; belajar names eight subjects ("mapel") its own way.

use std::collections::BTreeMap;

use serde::Deserialize;

/// One knowledge base as the knowledge base lists it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeBase {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub document_count: u32,
}

/// The mapel belajar knows, each with the knowledge-base subjects that teach it. SD's
/// thematic book ("Tematik") covers several subjects at once; IPAS is SD's (and grade 10's)
/// science; SMA splits IPA and IPS into separate books.
const MAPEL: &[(&str, &[&str])] = &[
    (
        "IPA",
        &["IPA", "IPAS", "Fisika", "Kimia", "Biologi", "Tematik"],
    ),
    ("Matematika", &["Matematika"]),
    ("Bahasa Inggris", &["Bahasa Inggris"]),
    ("Bahasa Indonesia", &["Bahasa Indonesia", "Tematik"]),
    (
        "IPS",
        &[
            "IPS",
            "IPAS",
            "Ekonomi",
            "Geografi",
            "Sejarah",
            "Sosiologi",
            "Antropologi",
            "Tematik",
        ],
    ),
    (
        "Informatika",
        &["Informatika", "Koding & Kecerdasan Artifisial"],
    ),
    ("Pendidikan Pancasila", &["Pendidikan Pancasila", "Tematik"]),
    ("Pendidikan Agama Islam", &["Agama Islam"]),
];

/// Bases never read: teacher's books carry the answer keys.
const EXCLUDED: &[&str] = &["Arsip — Buku Guru"];

/// "IPA — Kelas 7" -> ("IPA", 7); "SMP Kelas 7" -> ("SMP", 7); anything else -> None.
fn subject_and_grade(name: &str) -> Option<(&str, u8)> {
    let (subject, grade) = name
        .rsplit_once(" — Kelas ")
        .or_else(|| name.rsplit_once(" Kelas "))?;
    let grade = grade.trim().parse::<u8>().ok()?;
    Some((subject.trim(), grade))
}

/// One mapel of a grade: belajar's name and the bases behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradeMapel {
    pub name: &'static str,
    pub knowledge_base_ids: Vec<String>,
    pub document_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    bases: Vec<KnowledgeBase>,
}

impl Catalog {
    pub fn new(bases: Vec<KnowledgeBase>) -> Self {
        Self { bases }
    }

    pub fn is_empty(&self) -> bool {
        self.bases.is_empty()
    }

    fn of_grade(&self, grade: u8) -> impl Iterator<Item = (&KnowledgeBase, &str)> {
        self.bases.iter().filter_map(move |kb| {
            if EXCLUDED.contains(&kb.name.as_str()) {
                return None;
            }
            let (subject, g) = subject_and_grade(&kb.name)?;
            (g == grade).then_some((kb, subject))
        })
    }

    /// The mapel a grade has books for, in belajar's order.
    pub fn mapel(&self, grade: u8) -> Vec<GradeMapel> {
        let mut found: BTreeMap<usize, GradeMapel> = BTreeMap::new();
        for (kb, subject) in self.of_grade(grade) {
            for (order, (name, subjects)) in MAPEL.iter().enumerate() {
                if !subjects.contains(&subject) {
                    continue;
                }
                let entry = found.entry(order).or_insert_with(|| GradeMapel {
                    name,
                    knowledge_base_ids: Vec::new(),
                    document_count: 0,
                });
                entry.knowledge_base_ids.push(kb.id.clone());
                entry.document_count += kb.document_count;
            }
        }
        // A mapel served only by the thematic book is still offered: SD learns it that way.
        found.into_values().collect()
    }

    /// The bases one question searches: the mapel's, or every base of the grade.
    pub fn bases_for(&self, grade: u8, mapel: Option<&str>) -> Vec<String> {
        match mapel {
            Some(name) => {
                let wanted = self
                    .mapel(grade)
                    .into_iter()
                    .find(|m| m.name.eq_ignore_ascii_case(name.trim()));
                match wanted {
                    Some(m) => m.knowledge_base_ids,
                    // An unknown mapel falls back to the whole grade rather than to nothing.
                    None => self.bases_for(grade, None),
                }
            }
            None => self.of_grade(grade).map(|(kb, _)| kb.id.clone()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "tests")]

    use super::*;

    fn kb(id: &str, name: &str, docs: u32) -> KnowledgeBase {
        KnowledgeBase {
            id: id.into(),
            name: name.into(),
            document_count: docs,
        }
    }

    fn catalog() -> Catalog {
        Catalog::new(vec![
            kb("ipa7", "IPA — Kelas 7", 3),
            kb("mtk7", "Matematika — Kelas 7", 2),
            kb("pai7", "Agama Islam — Kelas 7", 2),
            kb("pjok7", "PJOK — Kelas 7", 1),
            kb("smp7", "SMP Kelas 7", 1),
            kb("ipa8", "IPA — Kelas 8", 3),
            kb("tem3", "Tematik — Kelas 3", 4),
            kb("ipas3", "IPAS — Kelas 3", 1),
            kb("fis11", "Fisika — Kelas 11", 1),
            kb("bio11", "Biologi — Kelas 11", 1),
            kb("guru", "Arsip — Buku Guru", 70),
        ])
    }

    #[test]
    fn a_grade_offers_belajars_mapel_with_their_bases() {
        let mapel = catalog().mapel(7);
        let names: Vec<&str> = mapel.iter().map(|m| m.name).collect();
        assert_eq!(names, ["IPA", "Matematika", "Pendidikan Agama Islam"]);
        assert_eq!(mapel[0].knowledge_base_ids, ["ipa7"]);
        assert_eq!(mapel[0].document_count, 3);
    }

    #[test]
    fn sd_science_reads_ipas_and_the_thematic_book() {
        let c = catalog();
        let mut ipa = c.bases_for(3, Some("IPA"));
        ipa.sort();
        assert_eq!(ipa, ["ipas3", "tem3"]);
        assert_eq!(c.bases_for(11, Some("ipa")), ["fis11", "bio11"]);
    }

    #[test]
    fn all_subjects_read_the_whole_grade_but_never_the_teachers_books() {
        let mut all = catalog().bases_for(7, None);
        all.sort();
        assert_eq!(all, ["ipa7", "mtk7", "pai7", "pjok7", "smp7"]);
        assert!(
            !catalog()
                .bases_for(7, Some("Sejarah"))
                .contains(&"guru".to_owned())
        );
    }

    #[test]
    fn names_parse_with_and_without_the_dash() {
        assert_eq!(subject_and_grade("IPA — Kelas 7"), Some(("IPA", 7)));
        assert_eq!(subject_and_grade("SMA Kelas 10"), Some(("SMA", 10)));
        assert_eq!(subject_and_grade("Arsip — Buku Guru"), None);
    }
}
