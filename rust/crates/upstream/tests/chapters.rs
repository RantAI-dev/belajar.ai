//! The chapters client against the fake serving the responses recorded from the real upstream.
//! These fail against the old built-in list: it never produced a real book title, an empty list or
//! a 400 from the service.
#![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "tests")]
#![allow(clippy::panic, reason = "tests")]

use std::time::Duration;

use belajar_core::grade::Grade;
use belajar_fake_upstream::{FakeConfig, FakeServer};
use belajar_ports::Upstream;
use belajar_ports::upstream::UpstreamError;
use belajar_upstream::{HttpUpstream, UpstreamConfig, UpstreamKeys};

const KEYS: &str = "7.ask=fake-7-ask,7.learn=fake-7-learn,7.practice=fake-7-practice";

async fn setup() -> (FakeServer, HttpUpstream) {
    let fake = belajar_fake_upstream::spawn(FakeConfig::default())
        .await
        .unwrap();
    let client = HttpUpstream::new(UpstreamConfig {
        base_url: fake.base_url.clone(),
        keys: UpstreamKeys::parse(KEYS).unwrap(),
        timeout: Duration::from_secs(10),
    })
    .unwrap();
    (fake, client)
}

#[tokio::test]
async fn the_client_parses_every_recorded_chapters_fixture() {
    let (_fake, client) = setup().await;
    let cases = [
        ("IPA", 13, "Sistem Organisasi Kehidupan Makhluk Hidup"),
        ("Matematika", 14, "Bilangan"),
        ("Bahasa Inggris", 8, "Good morning. How are You?"),
    ];
    for (mapel, count, first_title) in cases {
        let chapters = client
            .chapters(Grade::new(7).unwrap(), mapel)
            .await
            .unwrap();
        assert_eq!(chapters.len(), count, "{mapel} chapter count");
        assert_eq!(chapters[0].title, first_title, "{mapel} first title");
        assert_ne!(chapters[0].title, "Metode Ilmiah dan Pengukuran");
        assert!(!chapters[0].document_title.is_empty());
        for (index, chapter) in chapters.iter().enumerate() {
            assert_eq!(chapter.id, format!("bab-{}", index + 1));
            assert_eq!(chapter.number as usize, index + 1);
        }
    }
}

#[tokio::test]
async fn a_subject_without_recorded_chapters_is_an_empty_list() {
    let (_fake, client) = setup().await;
    let chapters = client
        .chapters(Grade::new(7).unwrap(), "IPS")
        .await
        .unwrap();
    assert!(chapters.is_empty());
}

#[tokio::test]
async fn an_unknown_mapel_is_a_failure_with_the_upstreams_status() {
    let (_fake, client) = setup().await;
    let error = client
        .chapters(Grade::new(7).unwrap(), "Fisika Kuantum")
        .await
        .unwrap_err();
    let UpstreamError::Failed { status, detail } = error else {
        panic!("a 400 maps to Failed: {error:?}");
    };
    assert_eq!(status, 400);
    assert!(detail.contains("Fisika Kuantum"), "{detail}");
}
