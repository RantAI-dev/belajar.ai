//! Prints the OpenAPI document as pretty JSON, for `scripts/gen-contracts.sh` to commit.
//! Results on stdout; everything else on stderr.

use belajar_server::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    match ApiDoc::openapi().to_pretty_json() {
        Ok(doc) => println!("{doc}"),
        Err(err) => {
            eprintln!("openapi: cannot serialize the document: {err}");
            std::process::exit(1);
        }
    }
}
