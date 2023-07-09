mod db_access;

use crate::web_page_reader::db_access::WebPageReaderDbAccess;

use url::Url;

pub struct WebPageReader {}

impl WebPageReader {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn fetch_links(&self) -> anyhow::Result<Vec<Url>> {
        let db = WebPageReaderDbAccess::new();
        get_pdf_links::execute(&db).await
    }
}

mod get_pdf_links {
    use crate::web_page_reader::db_access::WebPageReaderDbAccess;
    use anyhow::Context;
    use chrono::{Datelike, Utc};
    use itertools::Itertools;
    use lazy_static::lazy_static;
    use regex::Regex;
    use shared_kernel::http_client::HttpClient;
    use url::Url;

    pub(super) async fn execute(db: &WebPageReaderDbAccess) -> anyhow::Result<Vec<Url>> {
        let pdf_links = get_pdf_links_from_web_page().await?;
        let manually_added_links = db.get_manually_added_source_files().await?;
        let pdf_links = pdf_links
            .into_iter()
            .chain(manually_added_links.into_iter())
            .unique()
            .collect();
        db.return_unprocessed_files(pdf_links).await
    }

    async fn get_page_contents() -> anyhow::Result<String> {
        let url = Url::parse("https://kplc.co.ke/category/view/50/planned-power-interruptions")
            .context("Invalid URL")?;
        HttpClient::get_text(url).await
    }

    async fn get_pdf_links_from_web_page() -> anyhow::Result<Vec<Url>> {
        lazy_static! {
            static ref PDF_LINKS_REGEX: Regex =
                Regex::new(r#""https://.*kplc\.co\.ke/img/full/.*\.pdf""#)
                    .expect("PDF_LINKS_REGEX to compile");
        }

        let this_year = Utc::now().year().to_string();

        let page_content = get_page_contents().await?;
        let urls = PDF_LINKS_REGEX
            .find_iter(&page_content)
            .map(|a_match| {
                let link = a_match.as_str().replace('\"', "");
                Url::parse(link.as_str())
            })
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid URL")?;

        let this_years_pdf_urls = urls
            .into_iter()
            .filter(|url| url.to_string().contains(&this_year))
            .collect();
        Ok(this_years_pdf_urls)
    }
}
