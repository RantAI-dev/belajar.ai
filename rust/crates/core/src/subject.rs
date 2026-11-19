//! Our eight subjects: the id the API and the web app use, the name we show, and the name the
//! upstream calls it ("mapel"). Order is the order every list shows them in.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subject {
    pub id: &'static str,
    pub name: &'static str,
    /// The upstream's `mapel` value, sent back on chat and practice requests.
    pub mapel: &'static str,
}

pub const SUBJECTS: [Subject; 8] = [
    Subject {
        id: "ipa",
        name: "IPA",
        mapel: "IPA",
    },
    Subject {
        id: "mtk",
        name: "Matematika",
        mapel: "Matematika",
    },
    Subject {
        id: "bing",
        name: "Bahasa Inggris",
        mapel: "Bahasa Inggris",
    },
    Subject {
        id: "bind",
        name: "Bahasa Indonesia",
        mapel: "Bahasa Indonesia",
    },
    Subject {
        id: "ips",
        name: "IPS",
        mapel: "IPS",
    },
    Subject {
        id: "inf",
        name: "Informatika",
        mapel: "Informatika",
    },
    Subject {
        id: "pp",
        name: "Pendidikan Pancasila",
        mapel: "Pendidikan Pancasila",
    },
    Subject {
        id: "pai",
        name: "Pendidikan Agama Islam",
        mapel: "Pendidikan Agama Islam",
    },
];

#[must_use]
pub fn by_id(id: &str) -> Option<&'static Subject> {
    SUBJECTS.iter().find(|s| s.id == id)
}

/// Matches the upstream's name ignoring case and surrounding space.
#[must_use]
pub fn by_mapel(mapel: &str) -> Option<&'static Subject> {
    let wanted = mapel.trim();
    SUBJECTS
        .iter()
        .find(|s| s.mapel.eq_ignore_ascii_case(wanted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_lookups_round_trip() {
        for s in &SUBJECTS {
            assert_eq!(by_id(s.id), Some(s));
            assert_eq!(by_mapel(s.mapel), Some(s));
        }
        let mut ids: Vec<&str> = SUBJECTS.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), SUBJECTS.len());
    }

    #[test]
    fn unknown_values_have_no_subject() {
        assert_eq!(by_id("kimia"), None);
        assert_eq!(by_mapel("Kimia"), None);
        assert_eq!(by_mapel(" ipa "), by_id("ipa"));
    }
}
