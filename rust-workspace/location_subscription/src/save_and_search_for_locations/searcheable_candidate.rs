use entities::power_interruptions::location::AreaName;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;
use regex::RegexBuilder;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

lazy_static! {
    static ref ACRONYM_MAP: HashMap<String, &'static str> = HashMap::from([
        ("pri".to_string(), "Primary"),
        ("rd".to_string(), "Road"),
        ("est".to_string(), "Estate"),
        ("sch".to_string(), "School"),
        ("schs".to_string(), "Schools"),
        ("sec".to_string(), "Secondary"),
        ("stn".to_string(), "Station"),
        ("stns".to_string(), "Station"),
        ("apts".to_string(), "Apartments"),
        ("hqtrs".to_string(), "Headquaters"),
        ("mkt".to_string(), "Market"),
        ("fact".to_string(), "Factory"),
        ("t/fact".to_string(), "Tea Factory"),
        ("t /fact".to_string(), "Tea Factory"),
        ("t / fact".to_string(), "Tea Factory"),
        ("c/fact".to_string(), "Coffee Factory"),
        ("c /fact".to_string(), "Coffee Factory"),
        ("c / fact".to_string(), "Coffee Factory"),
        ("petro".to_string(), "Petrol"),
    ]);
    static ref REGEX_STR: String = {
        let keys = ACRONYM_MAP.keys().join("|");
        format!(r"\b(?:{})\b", keys)
    };
    static ref ACRONYMS_MATCHER: Regex = RegexBuilder::new(&REGEX_STR)
        .case_insensitive(true)
        .build()
        .expect("ACRONYMS_MATCHER to have been built successfully");
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NonAcronymString(String);

impl From<String> for NonAcronymString {
    fn from(value: String) -> Self {
        let result = ACRONYMS_MATCHER
            .replace_all(&value, |cap: &Captures| {
                let cap_as_lower_case = cap[0].to_lowercase();
                ACRONYM_MAP
                    .get(&cap_as_lower_case)
                    .cloned()
                    .unwrap_or_default()
                    .to_string()
            })
            .trim()
            .to_string();

        NonAcronymString(result)
    }
}

impl Display for NonAcronymString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for NonAcronymString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct SearcheableCandidateInner(Vec<NonAcronymString>);

impl SearcheableCandidateInner {
    fn new(candidate: String) -> Self {
        let split = candidate.split_once('&');
        let candidates = if let Some((before, after)) = split {
            let mut result = vec![];
            let before = before.trim();
            let after = after.trim();
            result.extend(Self::split_1(before, after));
            result.extend(Self::split_2(before, after));
            result.extend(Self::split_3(before, after));
            result.extend(Self::split_4(before, after));
            result.extend(Self::split_5(before, after));
            result
                .into_iter()
                .map(|data| {
                    let unique_string = data.trim().split(' ').unique().join(" ");
                    Self::replace_space_with_pg_search_symbol(unique_string)
                })
                .unique()
                .collect_vec()
        } else {
            vec![Self::replace_space_with_pg_search_symbol(candidate)]
        };

        SearcheableCandidateInner(candidates)
    }

    fn replace_space_with_pg_search_symbol(value: String) -> NonAcronymString {
        let non_acronym_string = NonAcronymString::from(value).to_string();
        let searcheable_str = non_acronym_string.trim().replace(' ', " <-> ");
        NonAcronymString::from(searcheable_str)
    }

    // Split Shell & Total Petro Stns Kiambu Road to vec![Shell Petro Stns Kiambu Road, Total Petro Stns Kiambu Road]
    fn split_1(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let after_as_list = after_amperstand.split(' ').collect_vec();
        // If after_amperstand is only one word, then this split method isnt applicable.
        // eg: Makueni boys & girls
        if after_as_list.len() == 1 || before_amperstand.split(' ').count() > 1 {
            return Default::default();
        }
        let data = after_as_list.split_first();
        if let Some((first, rest)) = data {
            let rest_as_str = rest.join(" ");
            let result = vec![
                format!("{} {}", before_amperstand, &rest_as_str),
                format!("{} {}", &first, &rest_as_str),
            ];
            result
        } else {
            vec![]
        }
    }

    // Split Kawangware DC & DO Offices to vec![Kawangware DC Offices, Kawangware DO Offices]
    fn split_2(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let mut result = vec![];
        let before_as_list = before_amperstand.split(' ').collect_vec();
        // If before_amperstand is only one word, then this split method isnt applicable.
        if before_as_list.len() == 1 {
            return Default::default();
        }
        if let Some(first) = before_as_list.first() {
            result.push(format!("{} {}", first, after_amperstand));
        }

        let after_as_list = after_amperstand.split(' ').collect_vec();
        // If after_amperstand is only one word, then this split method isnt applicable.
        if after_as_list.len() == 1 {
            return Default::default();
        }

        if let Some(last) = after_as_list.last() {
            result.push(format!("{} {}", before_amperstand, last));
        }

        result
    }

    // Split Makueni Boys & Girls to vec![Makueni Boys, Makueni Girls]
    fn split_3(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        // If before_amperstand is only one word, then this split method isnt applicable.
        // eg: 2nd & 3rd Parklands, we will end up with "2nd" as a candidate, which is too general.
        if before_amperstand.split(' ').count() == 1 {
            return Default::default();
        }
        let mut result = vec![before_amperstand.to_string()];

        let before_as_list = before_amperstand.split(' ').collect_vec();
        if let Some(first) = before_as_list.first() {
            result.push(format!("{} {}", first, after_amperstand));
        }

        result
    }

    // Split St Lwanga Catholic Church & School to vec![St Lwanga Catholic Church, St Lwanga Catholic School]
    fn split_4(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let before_as_list = before_amperstand.split(' ').collect_vec();
        // If before_amperstand is only one word, then this split method isnt applicable.
        // eg: 2nd & 3rd Parklands, we will end up with "2nd" as a candidate, which is too general.
        if before_as_list.len() == 1 {
            return Default::default();
        }
        let last_and_elements_before = before_as_list.split_last();
        if let Some((_last, elements_before_last_word)) = last_and_elements_before {
            let elements_before_last_word = elements_before_last_word.join(" ");
            let final_word = format!("{} {}", elements_before_last_word, after_amperstand);
            let result = vec![format!("{}", before_amperstand), final_word];

            result
        } else {
            vec![]
        }
    }

    // Split GSU & AP to vec![GSU, AP]
    fn split_5(before_amperstand: &str, after_amperstand: &str) -> Vec<String> {
        let phrase_without_amperstand = format!("{} {}", before_amperstand, after_amperstand);
        // only return result, if phrase_without_amperstand is two words.
        if phrase_without_amperstand.split(' ').count() == 2 {
            let result = vec![before_amperstand.to_string(), after_amperstand.to_string()];

            return result;
        }

        Default::default()
    }

    #[cfg(test)]
    fn into_inner(self) -> Vec<String> {
        self.0.into_iter().map(|data| data.0).collect_vec()
    }
}

pub struct SearcheableAreaName(Vec<String>);

impl SearcheableAreaName {
    pub fn new(area: &AreaName) -> Self {
        let area_name = area.as_ref().replace("AREA", "");
        // an areaName can have this format `MUTHAIGA & BALOZI ESTATE`  with amperstand
        // or this format `SEWAGE, GITHUNGURI, EASTERN BYPASS` with comma;
        if area_name.contains(',') {
            return Self(
                area_name
                    .split(',')
                    .map(|item| item.to_owned())
                    .collect::<Vec<_>>(),
            );
        }

        Self(area_name.split('&').map(|item| item.to_owned()).collect())
    }

    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SearcheableCandidates(SearcheableCandidateInner);

impl SearcheableCandidates {
    pub fn into_inner(self) -> Vec<String> {
        self.0
             .0
            .into_iter()
            .map(|data| data.to_string())
            .collect_vec()
    }

    pub fn inner(&self) -> Vec<String> {
        self.0 .0.iter().map(|data| data.to_string()).collect_vec()
    }
}

impl SearcheableCandidates {
    pub fn from_area_name(area: &AreaName) -> Vec<Self> {
        let searcheable_area = SearcheableAreaName::new(area);
        return searcheable_area
            .into_inner()
            .iter()
            .map(|data| SearcheableCandidates::from(data.as_ref()))
            .collect_vec();
    }
}

impl From<&str> for SearcheableCandidates {
    fn from(value: &str) -> Self {
        let value = value.to_owned();
        let value = SearcheableCandidateInner::new(value);
        SearcheableCandidates(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableCandidateInner;
    use rstest::rstest;
    use std::collections::HashSet;

    #[rstest]
    #[case("2nd & 3rd Parklands", vec!["2nd <-> Parklands", "3rd <-> Parklands"])]
    #[case("GSU & AP", vec!["GSU", "AP"])]
    #[case("Makueni Boys & Girls", vec!["Makueni <-> Boys", "Makueni <-> Girls"])]
    #[case("Kabare Market & Girls High School", vec!["Kabare <-> Market", "Kabare <-> Girls <-> High <-> School"])]
    #[case("Kimunye T /Fact & Market", vec!["Kimunye <-> Tea <-> Factory", "Kimunye <-> Market"])]
    #[case("St Lwanga Catholic Church & School", vec!["St <-> Lwanga <-> Catholic <-> Church", "St <-> Lwanga <-> Catholic <-> School"])]
    #[case("Shell & Total Petro Stns Kiambu Road", vec!["Shell <-> Petrol <-> Station <-> Kiambu <-> Road", "Total <-> Petrol <-> Station <-> Kiambu <-> Road"])]
    #[case("Kawangware DC & DO Offices", vec!["Kawangware <-> DC <-> Offices", "Kawangware <-> DO <-> Offices"])]
    #[case("MUTHAIGA & BALOZI ESTATE", vec!["MUTHAIGA <-> ESTATE", "BALOZI <-> ESTATE"])]
    fn test_searcheable_candidate_inner(#[case] input: &str, #[case] expected: Vec<&str>) {
        let candidate = SearcheableCandidateInner::new(input.to_string());
        let result = candidate.into_inner().into_iter().collect::<HashSet<_>>();
        let expected_as_hash_set = expected
            .into_iter()
            .map(|data| data.to_owned())
            .collect::<HashSet<_>>();
        println!("{result:?}");
        assert_eq!(expected_as_hash_set.difference(&result).count(), 0)
    }
}
