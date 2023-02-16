use crate::token::{Area, County, Region};
use multipeek::{multipeek, MultiPeek};
use std::vec::IntoIter;

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

pub fn interpret(regions: Vec<Region>) -> Vec<Region> {
    regions
        .into_iter()
        .map(RegionInterpreter::interpret)
        .collect()
}

struct RegionInterpreter;

impl RegionInterpreter {
    fn interpret(region: Region) -> Region {
        let name = RegionInterpreter::clean_up_region_name(region.name);
        let counties = region
            .counties
            .into_iter()
            .map(CountyInterpreter::interpret)
            .collect::<Vec<_>>();
        Region { name, counties }
    }

    fn clean_up_region_name(region_name: String) -> String {
        region_name.replace("REGION", "").trim().to_string()
    }
}

struct CountyInterpreter;

impl CountyInterpreter {
    fn interpret(county: County) -> County {
        let name = CountyInterpreter::clean_up_county_name(county.name);
        let areas = county
            .areas
            .into_iter()
            .map(AreaInterpreter::interpret)
            .collect::<Vec<_>>();
        County { name, areas }
    }

    fn clean_up_county_name(name: String) -> String {
        name.replace("PARTS OF ", "")
            .replace("COUNTY", "")
            .trim()
            .to_string()
    }
}

struct AreaInterpreter;

impl AreaInterpreter {
    fn interpret(area: Area) -> Area {
        let pins = AreaPinsInterpreter::interpret(area.pins);
        Area {
            lines: area.lines,
            date: area.date,
            start: area.start,
            pins,
            end: area.end,
        }
    }
}

struct AreaPinsInterpreter;

impl AreaPinsInterpreter {
    fn interpret(lines: Vec<String>) -> Vec<String> {
        lines.into_iter().flat_map(interpret_line).collect()
    }
}

fn interpret_line(line: String) -> Vec<String> {
    lazy_static! {
        static ref PHASE: Regex =
            Regex::new(r"\d{1,}[\n\r\s]+[,&]+[\n\r\s]+\d{1,}").expect("PHASE regex to compile");
        static ref PHASE_NAME: Regex =
            Regex::new(r"([a-zA-Z]+[\n\r\s]+)").expect("PHASE_NAME regex to compile");
        static ref PHASE_NUMBERS: Regex =
            Regex::new(r"\d{1,}").expect("PHASE_NUMBERS regex to compile");
    }
    if PHASE.is_match(&line) {
        let phase_name = PHASE_NAME
            .captures_iter(&line)
            .into_iter()
            .map(|capture| format!("{}", &capture[0]))
            .collect::<String>();

        return PHASE_NUMBERS
            .captures_iter(&line)
            .into_iter()
            .map(|capture| format!("{} {}", &phase_name.trim(), &capture[0].trim()))
            .collect::<Vec<_>>();
    }
    vec![line]
}

#[cfg(test)]
mod tests {
    use crate::interpreter::interpret_line;

    #[test]
    fn test_regexx() {
        let result = interpret_line("Dandora Phase 3, 4 & 5".to_string());
        println!("{result:?}");

        let res = interpret_line("Dandora Phase 4 & 5".to_string());
        println!("{res:?}");
    }
}
