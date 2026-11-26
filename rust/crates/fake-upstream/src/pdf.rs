//! A whole book as a small PDF: what the real upstream serves for a book it stores as the
//! original upload, linked with `#page=<n>`. Text only, built by hand: no PDF library needed.

use std::fmt::Write as _;

/// How many pages a fake book has: enough to cover every page the seed sources cite.
pub(crate) const BOOK_PAGES: u32 = 160;

const WIDTH: u32 = 360;
const HEIGHT: u32 = 480;

/// One page's drawing: the book title, grey lines, two lines marked yellow, the page number.
fn page_content(title: &str, page: u32) -> String {
    let title = escape(title);
    let mut out = String::new();
    // Paper, then the two highlighted lines, under the text.
    let _ = writeln!(out, "1 0.99 0.95 rg 0 0 {WIDTH} {HEIGHT} re f");
    let _ = writeln!(out, "1 0.91 0.48 rg 20 347 320 15 re f 20 328 320 15 re f");
    let _ = writeln!(
        out,
        "0.13 0.13 0.13 rg BT /F1 16 Tf 24 438 Td ({title}) Tj ET"
    );
    let lines = [
        (
            410,
            "Pada bab ini kita mempelajari pengertian, contoh, dan penerapannya.",
            0.6,
        ),
        (
            394,
            "Materi disusun agar mudah dipahami oleh siswa kelas tujuh.",
            0.6,
        ),
        (
            378,
            "Setiap subbab diawali dengan pertanyaan pemantik sederhana.",
            0.6,
        ),
        (
            351,
            "Definisi utama dicetak tebal pada paragraf pertama bab ini.",
            0.2,
        ),
        (
            332,
            "Bagian ini sering muncul dalam soal ulangan akhir semester.",
            0.2,
        ),
        (
            308,
            "Ringkasan dan latihan berada di halaman-halaman terakhir.",
            0.6,
        ),
    ];
    for (y, text, grey) in lines {
        let _ = writeln!(
            out,
            "{grey} {grey} {grey} rg BT /F1 10 Tf 24 {y} Td ({text}) Tj ET"
        );
    }
    let _ = writeln!(
        out,
        "0.47 0.47 0.47 rg BT /F1 11 Tf 154 18 Td (Halaman {page}) Tj ET"
    );
    out
}

/// The book `document_id` as a PDF of [`BOOK_PAGES`] pages; page `n` says "Halaman n".
pub(crate) fn render_book(document_id: &str) -> Vec<u8> {
    // Objects: 1 catalog, 2 page tree, 3 font, then a page and its content stream per page.
    let page_obj = |page: u32| 4 + (page - 1) * 2;
    let mut objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        format!(
            "<< /Type /Pages /Count {BOOK_PAGES} /Kids [{}] >>",
            (1..=BOOK_PAGES)
                .map(|page| format!("{} 0 R", page_obj(page)))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
    ];
    for page in 1..=BOOK_PAGES {
        let content = page_content(document_id, page);
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {WIDTH} {HEIGHT}] \
             /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            page_obj(page) + 1
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{content}endstream",
            content.len()
        ));
    }

    let mut out = String::from("%PDF-1.4\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (object, number) in objects.iter().zip(1usize..) {
        offsets.push(out.len());
        let _ = write!(out, "{number} 0 obj\n{object}\nendobj\n");
    }
    let xref = out.len();
    let _ = write!(out, "xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in offsets {
        let _ = writeln!(out, "{offset:010} 00000 n ");
    }
    let _ = write!(
        out,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    );
    out.into_bytes()
}

/// A PDF string literal: `(`, `)` and `\` are escaped, anything outside printable ASCII dropped.
fn escape(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .flat_map(|c| match c {
            '(' | ')' | '\\' => vec!['\\', c],
            _ => vec![c],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::string_slice,
        reason = "tests"
    )]

    use super::*;

    #[test]
    fn the_book_is_a_pdf_whose_xref_points_at_every_object() {
        let pdf = render_book("kb-matematika-7");
        let text = String::from_utf8(pdf).unwrap();
        assert!(text.starts_with("%PDF-1.4\n") && text.ends_with("%%EOF\n"));
        assert!(text.contains(&format!("/Count {BOOK_PAGES}")));
        assert!(text.contains("(Halaman 122) Tj"));
        let xref: usize = text
            .rsplit("startxref\n")
            .next()
            .and_then(|tail| tail.lines().next())
            .and_then(|n| n.parse().ok())
            .unwrap();
        assert!(text[xref..].starts_with("xref\n"));
        // Every in-use entry's offset lands on that object's header.
        for (line, number) in text[xref..].lines().skip(3).zip(1usize..) {
            let Some(offset) = line.split(' ').next().and_then(|n| n.parse::<usize>().ok()) else {
                break;
            };
            if !line.ends_with(" n ") {
                break;
            }
            assert!(
                text[offset..].starts_with(&format!("{number} 0 obj\n")),
                "object {number}"
            );
        }
    }

    #[test]
    fn a_title_cannot_break_out_of_its_string() {
        assert_eq!(escape(r"a(b)c\d"), r"a\(b\)c\\d");
        assert_eq!(escape("é\nx"), "x");
    }
}
