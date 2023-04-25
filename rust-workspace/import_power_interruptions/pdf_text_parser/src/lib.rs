mod parser;
mod scanner;
mod token;
use crate::token::Area as TokenArea;
use crate::token::County as TokenCounty;
use crate::token::Region as TokenRegion;
use anyhow::anyhow;
use async_trait::async_trait;
use use_cases::import_affected_areas::{Area, County, Region};
use web_page_extractor::pdf_extractor::TextExtractor;

pub struct PDFContentExtractor;

#[async_trait]
impl TextExtractor for PDFContentExtractor {
    async fn extract(&self, text: String) -> anyhow::Result<Vec<Region>> {
        let tokens = scanner::scan(&text);
        let mut parser = parser::Parser::new(tokens);
        let result = parser.parse().map_err(|err| anyhow!("{err:?}"))?;

        Ok(result.into_iter().map(Into::into).collect())
    }
}

impl From<TokenRegion> for Region {
    fn from(value: TokenRegion) -> Self {
        Self {
            name: value.name,
            counties: value.counties.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TokenCounty> for County {
    fn from(value: TokenCounty) -> Self {
        Self {
            name: value.name,
            areas: value.areas.into_iter().map(From::from).collect(),
        }
    }
}

impl From<TokenArea> for Area {
    fn from(value: TokenArea) -> Self {
        Self {
            name: value.name,
            from: value.from,
            to: value.to,
            locations: value.locations,
        }
    }
}
