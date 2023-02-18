mod parser;
mod scanner;
mod token;
use crate::token::Area as TokenArea;
use crate::token::County as TokenCounty;
use crate::token::Region as TokenRegion;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use use_cases::import_planned_blackouts::{Area, County, Region};
use web_page_extractor::pdf_extractor::TextExtractor;
use web_page_extractor::PdfExtractor;

struct ContentExtractor {}

#[async_trait]
impl TextExtractor for ContentExtractor {
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
            lines: value.lines,
            date: value.date,
            start: value.start,
            end: value.end,
            locations: value.locations,
        }
    }
}
