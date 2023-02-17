use crate::parser::filter_out_comments::CommentsRemover;
use crate::scanner::{Date, KeyWords, Time, Token};
use crate::token::{Area, County, Region};
use multipeek::{multipeek, MultiPeek};
use std::collections::HashMap;
use std::iter;
use std::vec::IntoIter;

use regex::{Regex, RegexBuilder};

use anyhow::{Context, Error};
use chrono::{NaiveDate, NaiveTime};
use lazy_static::lazy_static;

mod filter_out_comments;

pub struct Parser {
    tokens: MultiPeek<IntoIter<Token>>,
}

#[derive(Debug)]
pub struct UnexpectedToken {
    found: Token,
    expected: String,
}

impl Time {
    fn parse(&self) -> Result<(NaiveTime, NaiveTime), ParseError> {
        let parsed_start = self.parse_time(&self.start)?;
        let parsed_end = self.parse_time(&self.end)?;

        Ok((parsed_start, parsed_end))
    }

    fn format_am_or_pm(&self, value: &str) -> String {
        value.replace("P.M.", "PM").replace("A.M.", "AM")
    }

    fn parse_time(&self, value: &str) -> Result<NaiveTime, ParseError> {
        let value = self.format_am_or_pm(value);
        NaiveTime::parse_from_str(&value, "%I.%M %p")
            .with_context(|| format!("Failed to parse {}", value))
            .map_err(ParseError::ValidationError)
    }
}

#[derive(Debug)]
pub enum ParseError {
    ValidationError(Error),
    UnexpectedEndOfFile,
    UnexpectedToken(UnexpectedToken),
}

macro_rules! consume_expected_token {
    ($tokens:expr, $expected:pat, $transform_token:expr, $required_element:expr) => {
        match $tokens.next() {
            Some($expected) => Ok($transform_token),
            Some(token) => {
                let unexpected_token = UnexpectedToken {
                    found: token.clone(),
                    expected: $required_element,
                };
                Err(ParseError::UnexpectedToken(unexpected_token))
            }
            None => Err(ParseError::UnexpectedEndOfFile),
        }
    };
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let comments_remover = CommentsRemover::new();
        let tokens = comments_remover.remove_comments(tokens);
        Self {
            tokens: multipeek(tokens.into_iter()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Region>, ParseError> {
        let mut regions = Vec::new();
        let mut error = None;
        loop {
            let result = self.parse_region();
            match result {
                Ok(region) => regions.push(region),
                Err(ParseError::UnexpectedEndOfFile) => {
                    break;
                }
                Err(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        Ok(regions
            .into_iter()
            .map(|region| self.sanitize_region(region))
            .collect())
    }

    fn sanitize_region(&self, region: Region) -> Region {
        lazy_static! {
            static ref WORDS_TO_REMOVE: Regex =
                RegexBuilder::new(r"(Parts? of)|(Whole of)|(Region)|(County)")
                    .case_insensitive(true)
                    .build()
                    .expect("WORDS_TO_REMOVE to have been built successfully");
        }

        fn sanitize(value: String) -> String {
            WORDS_TO_REMOVE.replace_all(&value, "").trim().to_string()
        }

        let name = sanitize(region.name);
        let counties = region
            .counties
            .into_iter()
            .map(|county| {
                let name = sanitize(county.name);
                let areas = county
                    .areas
                    .into_iter()
                    .map(|area| {
                        let lines = area.lines.into_iter().map(sanitize).collect();
                        let pins = area.pins.into_iter().map(sanitize).collect();
                        Area {
                            lines,
                            pins,
                            ..area
                        }
                    })
                    .collect();
                County { name, areas }
            })
            .collect();
        Region { name, counties }
    }

    fn parse_region(&mut self) -> Result<Region, ParseError> {
        let region = consume_expected_token!(
            self.tokens,
            Token::Region(literal),
            literal,
            "Region".to_string()
        )?;
        let counties = self.parse_counties()?;
        Ok(Region {
            name: region,
            counties,
        })
    }

    fn parse_counties(&mut self) -> Result<Vec<County>, ParseError> {
        let mut counties = vec![];
        // loop up until we get to another region, returning the list of counties
        fn does_region_match(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::Region(_)))
        }
        while !does_region_match(self.tokens.peek()) {
            counties.push(self.parse_county()?);
        }
        Ok(counties)
    }

    fn parse_county(&mut self) -> Result<County, ParseError> {
        let county = consume_expected_token!(
            self.tokens,
            Token::County(literal),
            literal,
            "County".to_string()
        )?;

        let areas = self.parse_areas()?;

        Ok(County {
            name: county,
            areas,
        })
    }

    fn parse_areas(&mut self) -> Result<Vec<Area>, ParseError> {
        let mut areas = vec![];
        fn matches_county_or_region(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::County(_)) | Some(&Token::Region(_)))
        }

        while !matches_county_or_region(self.tokens.peek()) {
            areas.push(self.area()?);
        }

        Ok(areas)
    }

    fn area(&mut self) -> Result<Area, ParseError> {
        let area_lines = consume_expected_token!(
            self.tokens,
            Token::Area(literal),
            literal
                .split(",")
                .map(|line| line.trim().to_string())
                .collect(),
            "Area".to_string()
        )?;

        let date = consume_expected_token!(
            self.tokens,
            Token::Date(Date { date, .. }),
            NaiveDate::parse_from_str(&date, "%d.%m.%Y")
                .context("Failed to parse the Date.")
                .map_err(ParseError::ValidationError),
            "Date".to_owned()
        )??;
        let (start, end) = consume_expected_token!(
            self.tokens,
            Token::Time(time),
            time.parse(),
            "Time".to_owned()
        )??;
        let pins = self.pins()?;

        Ok(Area {
            lines: area_lines,
            date,
            start,
            end,
            pins,
        })
    }

    fn pins(&mut self) -> Result<Vec<String>, ParseError> {
        let mut results = vec![];
        fn end_of_pins(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::Keyword(KeyWords::EndOfAreaPins)))
        }

        let mut pin_buffer = String::new();

        while !end_of_pins(self.tokens.peek()) {
            let token = self.tokens.next().ok_or(ParseError::UnexpectedEndOfFile)?;

            match token {
                Token::Comma if self.digit_after_comma() => pin_buffer.push_str(","),
                Token::Comma => {
                    results.push(pin_buffer.clone());
                    pin_buffer.clear();
                }
                Token::Identifier(identifier) => {
                    pin_buffer.push_str(" ");
                    pin_buffer.push_str(&identifier);
                }
                token => {
                    return Err(ParseError::UnexpectedToken(UnexpectedToken {
                        found: token,
                        expected: "Identifier".to_string(),
                    }))
                }
            }
        }

        results.push(pin_buffer);

        // consume the end of pins keyword
        self.tokens.next();

        Ok(results
            .into_iter()
            .map(|pin| self.extract_and_construct_pin_phases(pin))
            .flatten()
            .collect())
    }

    fn digit_after_comma(&mut self) -> bool {
        let peeked = self.tokens.peek();
        matches!(peeked, Some(&Token::Identifier(ref ident)) if ident.clone().trim().parse::<usize>().is_ok())
    }

    /// Eg: "Dandora Phase 1 & 2" becomes -> ["Dandora Phase 1", "Dandora Phase 2"]
    fn extract_and_construct_pin_phases(&self, pin: String) -> Vec<String> {
        lazy_static! {
            static ref PHASE: Regex =
                Regex::new(r"\d{1,}[\n\r\s]+[,&]+[\n\r\s]+\d{1,}").expect("PHASE regex to compile");
            static ref PHASE_NAME: Regex =
                Regex::new(r"([a-zA-Z]+[\n\r\s]+)").expect("PHASE_NAME regex to compile");
            static ref PHASE_NUMBERS: Regex =
                Regex::new(r"\d{1,}").expect("PHASE_NUMBERS regex to compile");
        }
        if PHASE.is_match(&pin) {
            let phase_name = PHASE_NAME
                .captures_iter(&pin)
                .into_iter()
                .map(|capture| format!("{}", &capture[0]))
                .collect::<String>();

            return PHASE_NUMBERS
                .captures_iter(&pin)
                .into_iter()
                .map(|capture| format!("{} {}", &phase_name.trim(), &capture[0].trim()))
                .collect::<Vec<_>>();
        }
        vec![pin]
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use crate::scanner::scan;

    #[test]
    fn test_parser() {
        let r = r"
        Interruption of 
Electricity Supply 
Notice is hereby given under Rule 27 of the Electric Power Rules 
That the electricity supply will be interrupted as here under: 
(It  is  necessary  to  interrupt  supply  periodically  in  order  to 
facilitate maintenance and upgrade of power lines to the network; 
to connect new customers or to replace power lines during road 
construction, etc.) 
 
NAIROBI REGION 

AREA: DANDORA 
DATE: Sunday 12.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
Dandora Phase 3, 4 & 5, HZ & Co. Ltd & adjacent customers. 
 
AREA: PART OF DANDORA 
DATE: Sunday 12.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
Dandora  Phase  1  &  2,  Dandora  Mkt,  Korogocho,  Kamunde  Rd  &  adjacent 
customers. 
 
AREA: PART OF UMOJA 3 
DATE: Sunday 12.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
KCC, Part of Umoja 3 & adjacent customers. 
 
AREA: KOMAROCK 
DATE: Sunday 12.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
Komarock Est, Part of Mowlem, Part of Mungetho & adjacent customers. 
 
AREA: PART OF ROAD C 
DATE: Sunday 12.02.2023                                         TIME: 9.00 A.M. - 3.00 P.M.   
Candy Kenya, Spectore, Eagle Plain Est, St. Bakhita Sch, Odds & Ends C & P, 
Vitafoam & adjacent customers.         
 
AREA: EASTLEGIH 1ST, 3RD AVENUE 
DATE: Tuesday 14.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Sewage Works, 1st Ave, California Est, Highrise Est, Madina Mall, Zawadi Pri Sch, 
Section 3 Eastleigh, 1st  Avenue Eastleigh & adjacent customers. 
 
AREA: BAHATI, SHAURI MOYO 
DATE: Tuesday 14.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Bahati S/Center, Ambira Rd, KBC Est Shauri Moyo, True Food Ltd, Kaloleni Est, 
Undugu Society & adjacent customers. 
 
AREA: JERUSALEM, JERICHO 
DATE: Tuesday 14.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Jerusalem Health Centre, Uhuru Est, Ofafa Jericho Pri Est, Kiambiu & adjacent 
customers. 
 
AREA: PART OF KAREN  
DATE: Tuesday 14.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Langata South Rd, Comboni Rd, Tangaza College, Bogani Rd, Part of Silanga 
Rd, JKUAT Karen, Kipevu Rd, Kifaru Rd, Ndalat Rd, Kuro Rd, Kenya School of 
Law & adjacent customers.  
 
AREA: PARTS OF UMOJA INNERCORE 
DATE: Tuesday 14.02.2023                                       TIME: 9.00 A.M. - 3.00 P.M.   
Oloibon  Hotel,  Unity  Pri,  Cathsam  Sch,  Inner  Core  Sango,  Kwa  Chief  Umoja 
Office, Umoja 2 Mkt & adjacent customers. 
 
AREA: SOUTH B  
DATE: Wednesday 15.03.2023                                  TIME: 9.00 A.M. - 3.00 P.M. 
South B S/Centre, Balozi Est, Mater Hosp, Mariakani Est, Daidai Rd, Kapiti Rd, 
Part of  Golden  Gate  Est,  Highway  Sec  Sch,  Elimu  Sacco,  South  B Police  Stn, 
KIMC,  Our  Lady  of  Mercy  Catholic  Church,  Kenol,  Kobil  Petrol  Stn  &  adjacent 
customers.  
 
AREA: DONHOLM, SAVANNAH 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Whole  of  Donholm  Phase  VIII,  Donholm  Phase  V,  New  Donholm  Est,  Old 
Donholm  Est,  Kioi  Plaza,  Donholm  Total  Petrol  Stn,  Donholm  Equity  Bank, 
Donholm Co-op Bank, Greenspan Est, Jacaranda Est, Harambee Sacco, Sunrise 
Est, Greenfield Est, Savannah Est, Parts of Soweto, Carmelvale Pri Sch, Edelvale 
Sch & adjacent customers. 
 
AREA: METAMETA 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Mathare 4A, Mathare Area 1, 2 & 3 & adjacent customers. 
 
AREA: MIREMA 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Mirema  Rd,  Farmers  Choice,  St.  Mary’s  Pri,  Mother  Teresa  Church,  Medas 
Academy, Mirema Service Apts Hotel & adjacent customers. 
 
AREA: PART OF PARKLANDS 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Stima  Lane,  Bidwood  Suites,  Blue  Ridge,  Westgate,  3rd  Parklands  Avenue, 
Mpaka Rd., Citam Church, Ngao Rd. and  adjacent customers 
 
AREA: PART OF WAIYAKI WAY 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
White Field Place, Safaricom House, School Lane & adjacent customers. 
 
AREA: KYUNA 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Brookside Drive, Lower Kabete Rd, Shanzu Rd, Kyuna Rd, Hillview Est, Loresho 
Ridge Rd, Kibagare Rd & adjacent customers. 
   
AREA: PEPONI 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Gulam's Mohammed, Mwanzi Rd, Gen Mathenge Drive, Peponi Gardens, Peponi 
Rd, Howard Humphrey, Pinewood Est, Pushpark Development Ltd, Portfolio Ltd 
& adjacent customers. 
 
AREA: PART OF WESTLANDS 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Sarit,  Sky  Park  Plaza,  Kipro  Centre,  Garden  Properties,  Bishop  Properties, 
Carbon  Ltd,  New  Rehema  House,  Canon  Aluminum  Fabricators  Ltd,  Telkom 
Kenya, Ravine Development Ltd, Madina Homes,  
Raphta Rd, St. Michael Rd & adjacent customers. 
 
AREA: THOME, GITHURAI, CLAYWORKS 
DATE: Sunday 19.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
KBL  HQ,  Baptist  Mission,  Indigo  Garments,  Ngumba,  GSU  HQ,  Drive-Inn  Est, 
ICIPE HQtrs, KENHA, Whole of Kasarani, Clay City Est, Warren, Lumumba Drive, 
Roysambu,  Safari  Park  Hotel,  Mountain  View  Roysambu,  USIU,  Ruaraka, 
Kasarani Sports Complex,  Printing Press, Ruaraka Housing Est, NIM Ltd, Andela 
Kenya  Ltd,  Thome,  Mirema  Rd,  Top  Quality  Garage,  Wankan  Zimmerman, 
Zimmerman,  Kahawa  Barracks,  Farmers  Choice,  Congo,  Kahawa  West  & 
adjacent customers. 
 
PARTS OF MACHAKOS COUNTY 
AREA: TALA MARKET 
DATE: Wednesday 15.02.2023                                 TIME: 9.00 A.M. – 5.00 P.M. 
Tala Mkt, Kangundo Mkt, Kathui, Koivaani, Kangundo Level 4 Hosp, Katwanyaa, 
Kambusu, Kyekoyo, Kinyui Girls & adjacent customers. 
  
AREA: PART OF WOTE ROAD 
DATE: Wednesday 15.02.2023                                 TIME: 9.00 A.M. – 5.00 P.M. 
Kimutwa  Sisters,  Kwa  Kavoo,  Yaitha,  Mbondoni,  Mbembani,  Kyawalia, 
Kyamuthinza,  Kitonyini,  Muumandu,  Ngiini,  Kamuuani,  Kavyuni,  Katuaa,  Iiyuni, 
Mutulani, Kola & adjacent customers. 
 
AREA: KATHIANI 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Kathiani,  Ngoleni,  Kaviani,  Mbee,  Kauti,  Kaiani,  Lumbwa,  Isyukoni, 
Kusyomuomo, Tendelyani & adjacent customers. 
 
AREA: KOMA MARKET, JOSKA MARKET 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Koma Mkt, Kantafu Mkt, Malaa Mkt, Kware, Joska Mkt & adjacent customers. 
 
NORTH RIFT REGION 
 
PARTS OF UASIN GISHU COUNTY 
AREA: KAPTAGAT, WONIFOUR, LENGWAI 
DATE: Wednesday 15.02.2023                                  TIME: 9.00 A.M. – 4.00 P.M. 
Kaptagat, Strawberg, Flax, Plateau, Kileges, Kipkabus, Chuiyat, Uhuru, Elgeiyo 
Border,  Tendwo,  Chirchir,  Bindura,  Sirwo,  Tilol,  Cheroreget,  Kapsuneiywo, 
Naiberi & adjacent customers. 
 
AREA: KAPKOI, KONDOO 
DATE: Friday 17.02.2023                                           TIME: 9.00 A.M. – 4.00 P.M. 
Cheptiret Pri, Kapko, Bayete, Lorian, Ndunguru, Burnt Forest, Kondoo, Ngarua, 
Soliat, Kitingia, Chereber, Cherus, Ainabkoi & adjacent customers. 
 
WESTERN REGION 
 
PARTS OF KISUMU COUNTY 
AREA: KBC, RED CROSS 
DATE: Tuesday 14.02.2023                                     TIME: 10.00 A.M. – 3.00 P.M. 
Red  Cross,  KBC,  LPASO  Restaurant,  Tumaini,  G4s  Headquarters,  JOUST  & 
adjacent customers. 
 
AREA: KISUMU LAW COURT 
DATE: Wednesday 15.02.2023                                TIME: 10.00 A.M. – 3.00 P.M. 
Nightngale, Whirlspring Hotel, Jaralam, Jumbo Apts, Kisumu Law Courts, K-City, 
DIRI, Ayoti Distributors, Lands Offices & adjacent customers. 
 
PARTS OF VIHIGA COUNTY 
AREA: GRAND ROYAL SWIZ HOTEL 
DATE: Wednesday 15.02.2023                                  TIME: 8.30 A.M. – 5.00 P.M. 
Grand  Royal  Swiz  Hotel,  Farm  Engineering,  Kiboswa  Mkt,  Kibowsa  Safaricom 
Boosters, Boyani, Jebkoyai, Givole & adjacent customers. 
 
PARTS OF BUNGOMA COUNTY 
AREA: KIMILILI CBD, KAPSOKWONY 
DATE: Saturday 11.02.2023                                    TIME: 9.00 A.M. – 12.00 P.M. 
Bahai Mkt, Matili Tech, Kimilili CBD, Bituyu, Lutonyi, Chebukwabi, Kuywa, Kitayi, 
Maeni Girls, Mukulima, Kumusinga Schs, Kamutiongo Water, Dream Land Hosp, 
Kapsokwony CBD, Kamuneru, Sambocho, Kimobo & adjacent customers. 
 
 

                                                                                                     

For further information, contact 
the nearest Kenya Power office 
  Interruption notices may be viewed at 
www.kplc.co.ke 
                                                                                                          

Interruption of 
Electricity Supply 
Notice is hereby given under Rule 27 of the Electric Power Rules 
That the electricity supply will be interrupted as here under: 
(It  is  necessary  to  interrupt  supply  periodically  in  order  to  facilitate 
maintenance and upgrade of power lines to the network; to connect new 
customers or to replace power lines during road construction, etc.) 
 
PARTS OF KAKAMEGA COUNTY 
AREA: SHIMANYIRO, LURAMBI 
DATE: Tuesday 14.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Mutsuma,  Bukhanga,  Imbiakalo,  Sawwa,  Lukume,  Shihome,  Shikutse,  Power 
Spot,  Bushiri,  Ingotse,  Ewamakhumbi,  Shikoti,  Esumeiya,  Eshiongo,  Lwatingu, 
Maraba,  Lwanda  Shop,  Lurambi,  Hirumbi,  Bukhulunya,  Joyland,  Eshisiru, 
Rosterman,  Shimanyiro,  Jamdas,  Elwesero,  Elwasambi,  Ekonyero  &  adjacent 
customers. 
 
SOUTH NYANZA REGION 
 
PARTS OF HOMABAY COUNTY 
AREA: ARAMO, RIAT MARKET 
DATE: Sunday 12.02.2023                                        TIME: 9.00 A.M. – 3.00 P.M. 
Riat  Mkt,  God  Nyango  Pri  Sch,  Kosira  Pri  Sch,  Nyafare Mkt,  Yadhelo  Pri  Sch, 
Otunga Water Point, Aramo Sch & adjacent customers. 
 
AREA: OWALO MARKET, ATEMO 
DATE: Monday 13.02.2023                                        TIME: 9.00 A.M. – 3.00 P.M. 
Owalo Mkt, Nyamokenye DEB Pri Sch and Village, Kakelo Stage, Atemo Water 
Point, Atemo Polytechnic and Sec Sch & adjacent customers. 
 
AREA: RAMBUSI, KODIERA 
DATE: Wednesday 15.02.2023                                 TIME: 9.00 A.M. – 3.00 P.M. 
Kodiera  Mkt,  Rambusi  Sec  Sch,  Olodo  &  Nyakahia  Pri  Schs  &  adjacent 
customers. 
 
PARTS OF MIGORI COUNTY 
AREA: KEHANCHA, NTIMARU 
DATE: Saturday 11.02.2023                                       TIME: 9.00 A.M. - 3.00 P.M.  
KEFRI, Migori Airstrip, Nyanchabo, Masaba, Nyamagagana, Kurutyange, Maeta, 
Kegonga, Matare, Senta, Kebaroti, Taranganya, Kobinto, Masangora, Komotobo, 
Remanyanki, Ntimaru, Kwiho, Ikerege, Nyametamburo & adjacent customers. 
 
AREA: NYAMOME, MASARA 
DATE: Wednesday 15.02.2023                                  TIME: 9.00 A.M. - 3.00 P.M.  
Nyamome,  Kababu,  Nyabisawa,  Namba  ka  Hezron,  St.  Peter’s  Abwao  Sec, 
Mukuro,  Chungni,  Nyarongi,  God  Kwer,  Mikei,  Kona  Kalangi  &  adjacent 
customers. 
 
PARTS OF NYAMIRA COUNTY 
AREA: SANGANYI TEA FACTORY 
DATE: Wednesday 15.02.2023                                  TIME: 9.00 A.M. - 3.00 P.M.  
Sanganyi  T/Fact,  Nyaramba  Mkt,  Makairo,  Kebirigo  High  Sch,  Viongozi,  Ibara 
Hosp, Kiabonyoru Girls, Kerema, Nyangoge, Nyagokiani, Itibo, Iteresi & adjacent 
customers. 
 
AREA: MOSOMBETI, GESIMA 
DATE: Friday 17.02.2023                                           TIME: 9.00 A.M. - 3.00 P.M.  
Gesima,  Mochenwa,  Nyamakoroto,  Mosombeti,  Enchoro,  Royal  Media, 
Riamanoti & adjacent customers. 
 
MT. KENYA REGION 
 
PARTS OF NYERI COUNTY 
AREA: NGAINI, HIRIGA 
DATE: Wednesday 15.02.2023                         TIME: 9.00 A.M. – 3.00 P.M. 
Rititi,  Kianjogu,  Thaithi,  Kiangima,  Gatiko,  Ngaini,  Kahiraini,  Gatung’ang’a, 
Mukanye    Ritho,  Maganjo,  Chieni,  Mikundi,  Kaiganaine,  Hiriga  &  adjacent 
customers. 
 
AREA: TETU, KIGOGOINI, IHWA 
DATE: Wednesday 15.02.2023                   TIME: 9.00 A.M. – 4.00 P.M. 
Kamuyu Mkt, Kagunduini Mkt, Kinunga Ihwa, Chania, Githakwa, Kamuyu C/Fact, 
Gachiro, Tetu Mkt, Githurini, Kihatha Mkt, Chania Mkt, Githumi Village & adjacent 
customers. 
 
PARTS OF KIRINYAGA COUNTY 
AREA; MAKUTANO, KIKUMINI, VI MKT  
DATE: Thursday 16.02.2023                         TIME: 8.30 A.M. – 1.30 P.M. 
Makutano, Kamweli, Chai Wu Yi, Kirwara, Atlantis Kamweli, Gitaraka, Kikumini, 
White Rose, VI Mkt, Makawani & adjacent customers. 
 
PARTS OF EMBU COUNTY 
AREA: GATONDO, MUCHONOKE 
DATE: Monday 13.02.2023                                  TIME: 9.00 A.M. – 5.00 P.M. 
Gatondo  C/Fact,  Kiamuringa  Mkt,  Kavingori  Dispensary,  Muchonoke  Mkt, 
Muinganania  &  Mukunguru  Mkt,  Mukunguru  Sch,  Siakago  Hosp,  Siakago  Mkt, 
Siakago Boys, Siakago Girls, Siakago DO, Riandu Mkt, Minuri Mkt, Kanyaga Mkt 
& adjacent customers. 
  
AREA: MAJIMBO EASTATE, KAMIU PRIMARY 
DATE: Tuesday 14.02.2023                                TIME: 9.00 A.M. – 5.00 P.M. 
Kamiu Macadamia Fact, St. Anne Catholic, By Grace Shopping Centre Kamiu, 
Lower Iveche & adjacent customers. 
  
AREA: ISHIARA, KAMWIMBI, INTEX CRASHER 
DATE: Thursday16.02.2023                                  TIME: 9.00 A.M. – 5.00 P.M. 
Ishiara  Mkt,  Ishiara  Hosp,  Intex  Quarry,  KETRACO  Ishiara  Switching  Station, 
Kieniri Mkt, Kamarandi Sec Sch, Ciangera Mkt, Karerema Mkt, Ngoce Sec Sch, 
Monge Mkt & adjacent customers. 
 
NORTH EASTERN REGION 
 
PARTS OF KIAMBU COUNTY 
AREA: KWAMAIKO CENTRE, CRF, NEMBU  
DATE: Saturday 11.02.2023                          TIME: 9.00 A.M. - 5.00 P.M. 
Gathage, Kahuguini, Githima, Viva Gardens, Jacaranda Water, Oaklands, Osho 
Grain  Millers,  Mega  Pipe  Ltd,  Kambui,  Raiyani,  Gathirimu  Girls  &  adjacent 
customers. 
 
AREA: FARMERS CHOICE 
DATE: Monday 13.02.2023                                         TIME: 9.00 A.M. - 5.00 P.M. 
Farmers  Choice,  Uplands  Police,  Canaan  Dairies,  Nyambari,  Olive  Farm, 
Roromo  Water,  Murengeti,  Mutosi,  Nyamweru  Water,  Juvenalis  Gitau  Sch, 
Githirioni Forest Offices & adjacent customers. 
 
AREA: PART OF BYPASS 
DATE: Tuesday 14.02.2023                                        TIME: 9.00 A.M. - 5.00 P.M. 
KU  Riverside,  Thome,  Exodus,  St.  Linda,  Kwihota,  Kihunguro,  Sunset,  Part  of 
Eastern  Bypass,  Mashinani,  Mitikenda,  Karunguru,  Mutonya,  Kiratina, 
Gatongora, Green Valley, Varsity Ville, Gicheha Farms & adjacent customers. 
 
AREA: CARBACID, KINALE, MATHORE 
DATE: Wednesday 15.02.2023                                  TIME: 9.00 A.M. - 5.00 P.M. 
Carbacid,  Kimende,  Lari,  Magina,  Rivelco,  Mirangi,  Maingi,  Afrodine,  Kinale, 
Gathuariga, Sulmac, Kiracha, Mukeo, Kwa Edward, Mathore, Muringa Holding, 
Marira Clinic, Kirenga, Kereita Farm & adjacent customers. 
 
AREA: GACHARAGE, ACME PLASTICS 
DATE: Wednesday 15.02.2023                                  TIME: 9.00 A.M. - 3.00 P.M. 
Karuri Hosp, Karuri Police, Karuri Exchange, Banana Mkt, Part of Ruaka Between 
Gacharage  &  Ndenderu  Junction,  National  Oil  Ndenderu,  Shell  Ndenderu, 
Banana & adjacent customers. 
 
AREA: GITHUNGURI TOWN, THAKWA  
DATE: Thursday 16.02.2023                                      TIME: 9.00 A.M. - 5.00 P.M. 
Wanjo, Thakwa, Eco Bricks, County Pride Hotel, Maichomo, Thuthuriki, Mukua, 
Ciiko Pri, Police Stn, Beta Care Hosp, Kiriko & adjacent customers. 
 
AREA: TOLA, BOB HARRIS, TIBS 
DATE: Thursday 16.02.2023                                      TIME: 9.00 A.M. - 5.00 P.M. 
Whole of Tola, Kiahuria, Metro, Parts of Ngoingwa, Ndarugo Motel, Compuera, 
Mpesa Academy, Muiri & adjacent customers. 
 
AREA: FARMERS CHOICE 
DATE: Friday 17.02.2023                                            TIME: 9.00 A.M. - 5.00 P.M. 
ACME Containers, Gatuikira, Tumaini Sch, Red Hill North, Raphalites, Valentine 
Growers, Laini, Norbrook, Ombi Rubber Rollers, Kentmare Club, Echuka Farm, 
Farley Dam, High Wood Farm, Marambaa Factory Water Pump, Njiku, Uplands, 
Canaan, Githirioni & adjacent customers. 
 
AREA: KIGWARU, MUCHATHA, GATHANGA 
DATE: Friday 17.02.2023                                           TIME: 9.00 A.M. - 5.00 P.M. 
Little  Sisters,  Guango,  Clifftop,  Kigwaru  Inn,  Royal  Brains,  Ruaka  Shopping 
Center & adjacent customer. 
 
COAST REGION 
 
PARTS OF MOMBASA COUNTY 
AREA: PART OF SHANZU 
DATE: Thursday 16.02.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Papillon,  Neptune,  Seven  Sea  Lodge,  Kenya  Bay,  Indiana  Beach,  Yuls  Hotel, 
Kenya Bay, Severine Hotel, Kahama Hotel & adjacent customers. 
 
PARTS OF KILIFI COUNTY 
AREA: PART OF KANAMAI 
DATE: Tuesday 14.02.2023               TIME: 9.00 A.M. – 5.00 P.M. 
Northcoast  Beach  Hotel,  Gurdum  Villah,  Xanadu  Villahs,  Royal  Reserve, 
Kanamai  Conference,  Salama  Beach  Hotel,  St.  Mathews  Sch,  Barani  Timboni, 
Barani Sch, Kanamai Four Farm, Mtwapa Gardens, Cocacola Bottlers, Bayusuf 
Farm,  Ashut  Plastics,  Brilliant  EPZ,  Mombasa  Apparel,  Rembo  Apts,  Kariuki 
Farm,  Kwa  Mwavitswa,  Toto  Dogo,  Biladi,  Mtwapa  Pri,  Pwani  Fish  Farm  & 
adjacent customers. 
 
AREA: PART OF MTWAPA 
DATE: Thursday 16.02.2023               TIME: 9.00 A.M. – 5.00 P.M. 
Mtwapa  Rhino,  Mtwapa  Plaza,  Jambo  Jipya  Medical,  Mtwapa  Gardens, 
Moorings,  Kwa  Wapokomo,  Funyula,  Mtomondoni  Chief’s  Camp,  Mikanjuni, 
Airport, Aloo Drive, Creek, Zanak, Mtomondoni Pri, Mtomondoni Sec, AlBastad, 
Mulika  Mwizi,  Mama  Mtaa,  Golden  Key  Kubwa,  Kizingitini,  Mama  Mabata  & 
adjacent customers. 
 
 

                                                                                                    

For further information, contact 
the nearest Kenya Power office 
  Interruption notices may be viewed at 
www.kplc.co.ke 
        ";

        let results = scan(r);
        let mut parser = Parser::new(results);

        let parsed_results = parser.parse();

        println!("{:?}", parsed_results);
    }
}
