use crate::parser::filter_out_comments::CommentsRemover;
use crate::scanner::{Date, KeyWords, Time, Token};
use crate::token::{Area, County, Region};
use multipeek::{multipeek, MultiPeek};

use std::vec::IntoIter;

use regex::{Regex, RegexBuilder};

use anyhow::{anyhow, Context, Error};
use chrono::{NaiveDate, NaiveTime};
use lazy_static::lazy_static;

mod filter_out_comments;

pub struct Parser {
    tokens: MultiPeek<IntoIter<Token>>,
}

#[derive(Debug)]
pub struct UnexpectedToken {
    pub found: Token,
    pub expected: String,
}

impl Time {
    fn parse(&self) -> Result<(NaiveTime, NaiveTime), ParseError> {
        let parsed_start = self.parse_time(&self.start)?;
        let parsed_end = self.parse_time(&self.end)?;

        Ok((parsed_start, parsed_end))
    }

    fn format_am_or_pm(&self, value: &str) -> String {
        value
            .replace("P.M.", "PM")
            .replace("P.M", "PM")
            .replace("A.M.", "AM")
            .replace("A.M", "AM")
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
                        let locations = area.locations.into_iter().map(sanitize).collect();
                        Area {
                            name: sanitize(area.name),
                            locations,
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
        let name = consume_expected_token!(
            self.tokens,
            Token::Area(literal),
            literal.trim().to_string(),
            "Area".to_string()
        )?;

        let date = consume_expected_token!(
            self.tokens,
            Token::Date(Date { date, .. }),
            NaiveDate::parse_from_str(&date, "%d.%m.%Y")
                .with_context(|| format!("Failed to parse the Date. {date:?}"))
                .map_err(ParseError::ValidationError),
            "Date".to_owned()
        )??;

        let (start, end) = consume_expected_token!(
            self.tokens,
            Token::Time(time),
            time.parse(),
            "Time".to_owned()
        )??;

        let from = date
            .and_time(start)
            .try_into()
            .map_err(|err| ParseError::ValidationError(anyhow!("{err}")))?;
        let to = date
            .and_time(end)
            .try_into()
            .map_err(|err| ParseError::ValidationError(anyhow!("{err}")))?;
        let pins = self.locations()?;

        Ok(Area {
            name,
            to,
            from,
            locations: pins,
        })
    }

    fn locations(&mut self) -> Result<Vec<String>, ParseError> {
        let mut results = vec![];
        fn end_of_locations(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::Keyword(KeyWords::EndOfAreaLocations)))
        }

        let mut location_buffer = String::new();

        while !end_of_locations(self.tokens.peek()) {
            let token = self.tokens.next().ok_or(ParseError::UnexpectedEndOfFile)?;

            match token {
                Token::Comma if self.digit_after_comma() => location_buffer.push(','),
                Token::Comma => {
                    results.push(location_buffer.clone());
                    location_buffer.clear();
                }
                Token::Identifier(identifier) => {
                    location_buffer.push(' ');
                    location_buffer.push_str(&identifier);
                }
                token => {
                    return Err(ParseError::UnexpectedToken(UnexpectedToken {
                        found: token,
                        expected: "Identifier".to_string(),
                    }))
                }
            }
        }

        results.push(location_buffer);

        // consume the end of pins keyword
        self.tokens.next();

        Ok(results
            .into_iter()
            .flat_map(|pin| self.extract_and_construct_location_phases(pin))
            .collect())
    }

    fn digit_after_comma(&mut self) -> bool {
        let peeked = self.tokens.peek();
        matches!(peeked, Some(Token::Identifier(ident)) if ident.clone().trim().parse::<usize>().is_ok())
    }

    /// Eg: "Dandora Phase 1 & 2" becomes -> ["Dandora Phase 1", "Dandora Phase 2"]
    fn extract_and_construct_location_phases(&self, location: String) -> Vec<String> {
        lazy_static! {
            static ref PHASE: Regex =
                Regex::new(r"\d{1,}[\n\r\s]+[,&]+[\n\r\s]+\d{1,}").expect("PHASE regex to compile");
            static ref PHASE_NAME: Regex =
                Regex::new(r"([a-zA-Z]+[\n\r\s]+)").expect("PHASE_NAME regex to compile");
            static ref PHASE_NUMBERS: Regex =
                Regex::new(r"\d{1,}").expect("PHASE_NUMBERS regex to compile");
        }
        if PHASE.is_match(&location) {
            let phase_name = PHASE_NAME
                .captures_iter(&location)
                .into_iter()
                .map(|capture| (capture[0]).to_string())
                .collect::<String>();

            return PHASE_NUMBERS
                .captures_iter(&location)
                .into_iter()
                .map(|capture| format!("{} {}", &phase_name.trim(), &capture[0].trim()))
                .collect::<Vec<_>>();
        }
        vec![location]
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
 
AREA: NORTH AIRPORT ROAD 
DATE: Sunday 26.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
G4S,  Starbright,  Amiran  Kenya,  Co-op  Bank,  Equity  Bank,  North  Airport  Rd, 
Kitchen  Professionals,  Masai  Cables,  Alleluia  Ministries,  Axel  Engineering,  DK 
Engineering, Tri Clover Industries, Embakasi Village Crafts, Cabanas Stage, Feil, 
Green Forest, Barabara Plaza, Horticultural Produce Dev & adjacent customers.  
 
AREA: PARTS OF JKIA 
DATE: Sunday 26.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
Mitchell Cotts, Freight Complex, Kenya Airways Cargo, Signon, Acceller, Global 
Freight,  NAS Airport, Freight Terminals, Kenya Ports Authority, Crowne Plaza, 
Four  Points,  Shalimar  Flowers,  Kuehne  Nagel,  Airflo,  DHL  Global &  adjacent 
customers. 
 
AREA: KAYOLE MATOPENI 
DATE: Monday 27.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
Part of Kayole Matopeni, Kayole North Pri Sch & adjacent customers. 
 
AREA: PARTS OF UMOJA INNERCORE 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Oloibon  Hotel,  Unity  Pri,  Cathsam  Sch,  Inner  Core  Sango,  Kwa  Chief  Umoja 
Office, Umoja 2 Mkt & adjacent customers. 

AREA: KIWANJA 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Part of Kiwanja, Njurinjeke Hostel, Peponi Hostel, The Grace Hostels, Booster & 
adjacent customers. 
 
AREA: EASTLEIGH 1ST AVENUE 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Part of General Waruinge, Part of 1st Ave & adjacent customers. 
 
AREA: LORESHO 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Kaptagat Rd, KARI, Coopers, Syinyalu, Kaoumoni, Part of Loresho Ridge, Thego 
Rd, Qaribu Inn, Loresho Green Apts, Kabete Vetlab, Part of Waiyaki Way, UoN 
Kabete Campus, New Loresho Est & adjacent customers. 
 
AREA: MAMLAKA ROAD 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
UoN Hostels, State House Girls, Arboretum, Radisson Blue Hotel, Mamlaka Rd, 
Mamlaka  Hall,  Mamlaka  Court,  Mamlaka  Hostels,  Ufungamano  House, 
Permanent Presidential Music Commission & adjacent customers. 
 
AREA: MUKOMA ROAD 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Mukoma Rd, Multimedia Univ, Ndorobo Rd, Giraffe Centre, KWS, Parliamentary 
Studies, Kikeni Rd & adjacent customers. 
 
AREA: PART OF LANGATA, NAIROBI WEST 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Airport View, Part of Dam Est, Uhuru Monument, Total Petrol Stn Mbagathi Way, 
Blue Sky Est, Jonathan Glog, Deliverance Church, Dam 2 Est, Langata S/Centre, 
Nairobi  Sailing  Club,  Jehovah  Witness  Langata,  Funguo  Est,  Akila  1  Est,  AP 
Police  Camp,  Langata  Paradise,  Royal  Apt,  Choice Apt,  Chelsea  Marina,  Nula 
Apt, Texas Cancer Centre & adjacent customers. 
 
AREA: MUKURU KAYABA, HAZINA 
DATE: Wednesday 01.03.2023                                  TIME: 9.00 A.M. - 3.00 P.M. 
Mukuru Kayaba,  Hazina  Est,  Part of  Railway  Training  Institute,  Nerkwo,  Balozi 
Est,  Our  Lady  of  Mercy  Sec,  Naivas  South  B,  Aoko  Rd,  Railway  Training, 
Riverbank Phase 1 & 2, Kariba Est, Part of Golden Gate, Part of Kapiti Rd, South 
B Mosque & adjacent customers.  
 
AREA: PART OF KAMAE 
DATE: Thursday 02.03.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Part of Kamae, Shell and Total Petrol Stns & adjacent customers. 
 
AREA: MAASAI WEST ROAD 
DATE: Thursday 02.03.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Maasai West Rd, Maasai Rd, Ushirika Rd, Cooperative College, Twiga Rd, Hardy 
S/Centre, Koitobos Rd, Lamwia Rd & adjacent customers. 
 
AREA: PART OF LANGATA ROAD 
DATE: Thursday 02.03.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Part of Wilson Airport, Sunshine Sec, Maasai Est, Jonathan Ngeno, Breeze Est, 
Part of Uhuru Monument, Part of Dam Est & adjacent customers. 
 
AREA: PARTS OF UMOJA 1, INNERCORE 
DATE: Thursday 02.03.2023                                     TIME: 9.00 A.M. – 5.00 P.M. 
Umoja 1 H, J, K, L, M, N, P & Q, Co-op Bank Umoja, Umoja 1 Mkt, Egesa, Buru 
Buru Phase 4, EAST & adjacent customers. 
  
PARTS OF MACHAKOS COUNTY 
AREA: PART OF MUA ROAD  
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Catholic  Training  Institute,  Kitanga  Booster,  Kitanga  Pri,  Kivani  Pri,  Kwa  Muli 
Ranch & adjacent customers. 
 
AREA: PART OF KANGUNDO ROAD 
DATE: Wednesday 01.03.2023                                 TIME: 9.00 A.M. – 5.00 P.M. 
Mua  Ikumbini,  Mua  Girls,  KOL,  AIC  Wathia,  Kyasila,  Muthwani  &  adjacent 
customers. 
 
PARTS OF KAJIADO COUNTY 
AREA: KITENGELA, ISINYA 
DATE: Saturday 25.02.2023                                      TIME: 9.00 A.M. – 5.00 P.M.  
Milimani  Oloitikosh,  Kwa  Mohammed,  Golden  Plains  Academy  &  adjacent 
customers.  
 
AREA: ILPOLOSAT, KONZA 
DATE: Wednesday 01.03.2023                                  TIME: 9.00 A.M. – 5.00 P.M.  
Konza  Borehole,  New  Data  Center,  Konza  Cereal  Board,  Konza  ABC  Sec, 
Ilpolosat  Pri  &  Sec,  Konza  Town,  Kwa  Mumo  Matemo,  Naserian,  Ilmamen  & 
adjacent customers.  
 
PARTS OF MAKUENI COUNTY 
AREA: KAUMONI 
DATE: Wednesday 01.03.2023                                 TIME: 9.00 A.M. – 5.00 P.M. 
Kaumoni Mkt, Mbumbuni, No Nyanya, Ndumbi, Kisau, Mba, Itangini, Tawa Mkts, 
Uvaani,  Kikima,  Tuvilani,  Ngele,  Mavindu,  Kyuu,  Emale  Mbooni  Boys  &  Girls, 
Mbooni DC’s Office & Hospital & adjacent customers. 
 
CENTRAL RIFT REGION 

PARTS OF NAKURU COUNTY 
AREA: WHOLE OF BARNABAS  
DATE: Thursday 02.03.2023                 TIME: 8.30 A.M. - 2.00 P.M. 
Whole of Barnabas, Machine, Proto Energy, Ecorific Ltd, Royal Est, Buraha 
Zenoni, Mbaruk, Kwa Senior, Green Steads, Shiners Boys, Part of Pipeline & 
adjacent customers. 
 
AREA: WHOLE OF KIAMUNYI 
DATE: Thursday 02.03.2023                              TIME: 10.00 A.M. - 3.00 P.M. 
Whole of Zaburi S/Centre,  Olive Inn, Edma Est, Muiru Center,  Baraka Est, 
London Est, Part of Ngata, Mercy Njeri, Makiki Bore Hole, Madrugada Farm 
& adjacent customers. 
 
NORTH RIFT REGION 
 
PARTS OF UASIN GISHU COUNTY 
AREA: KAPKOI, BAYETE 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 4.00 P.M. 
Cheptiret Pri, Kapkoi, Bayete, Lorian, Ndunguru, Burnt Forest, Kondoo, Ngarua, 
Soliat, Kitingia, Chereber, Cherus, Ainabkoi & adjacent customers. 
 
AREA: TUGEN ESTATE, TOROCHMOI 
DATE: Tuesday 28.02.2023                                     TIME: 10.00 A.M. – 4.00 P.M. 
Tugen  Estate  Pri,  Kaburgei  Pri,  Chebarus  Pri,  Mosop,  Karo  Farm  &  adjacent 
customers. 
 
AREA: KAPTINGA, LEMOOK 
DATE: Wednesday 01.03.2023                                 TIME: 9.00 A.M. – 4.00 P.M. 
Eldoret  Sewage,  Kipkenyo,  Kaptinga,  Simat  Lemmok,  Tuiyo  Kamotong  & 
adjacent customers. 
 
AREA: OASIS BIBLE COLLEGE 
DATE: Friday 03.03.2023                                         TIME: 10.00 A.M. – 4.00 P.M. 
Oasis  Bible  College,  Annex  Buzeki,  Mti  Moja,  Kambi  Nguruwe  &  adjacent 
customers.  
 
AREA: KAPTAGAT, WONIFOUR, LENGWAI 
DATE: Friday 03.03.2023                        TIME: 9.00 A.M. – 4.00 P.M. 
Kaptagat, Strawberg, Flax, Plateau, Kileges, Kipkabus, Chuiyat, Uhuru, Elgeiyo 
Border,  Tendwo,  Chirchir,  Bindura,  Sirwo,  Tilol,  Cheroreget,  Kapsuneiywo, 
Naiberi & adjacent customers. 
 
AREA: PART OF ELDORET TOWN 
DATE: Sunday 05.03.2023                                        TIME: 9.00 A.M. – 4.00 P.M. 
UG  Sec,  Asis  Hotel,  Eagles  Hardware,  Sugarland  Plaza,  Naivas  Sokoni,  Siro 
Properties & adjacent customers.  
 
PARTS OF WEST POKOT COUNTY 
AREA: ORTUM 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 5.00 P.M. 
Ortum, Cemtech, Kerelwa, Sina, Murpus, Chepkorniswo, Sebit, Wakor, Marich, 
Sekerr, Sigor, Lomut, Sarmach, Kainuk & adjacent customers. 
 
 

                                                                                                     

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
 
WESTERN REGION 
 
PARTS OF KISUMU COUNTY 
AREA: PART OF KISUMU TOWN 
DATE: Sunday 26.02.2023                                        TIME: 9.00 A.M. – 3.00 P.M. 
Breweries Factory, United Millers Dedicated Airport Rd, Old Airport Gate, Equator 
Bottlers,  Kenya  Pipeline,  Usoma,  NITA,  NTSA,  Public  Works  Office,  Bandani, 
KENHA, Mombasa Maize Millers, Kombedu, Golf Club, Sea Foods, Subuni Rd, 
Toyota Kenya & adjacent customers. 
 
AREA: OMBEYI, KASONGO 
DATE: Tuesday 07.03.2023                                     TIME: 10.00 A.M. – 3.00 P.M. 
Miwani  Fact,  Kasongo  Mkt,  Ombeyi  Mkt,  Onyalo  Obiro,  Kasongo,  Nyakoko  & 
adjacent customers. 
 
PARTS OF SIAYA COUNTY 
AREA: BAR KADHIAMBO 
DATE: Tuesday 28.02.2023                                      TIME: 9.00 A.M. – 2.00 P.M. 
Bar  Kodhiambo,  Nyalgunga,  Nyamila,  Nina,  Nyamila  Pri,  Nyamila  Community 
Centre & adjacent customers. 
 
PARTS OF BUSIA COUNTY 
AREA: BUDALANGI, PORT VICTORIA 
DATE: Wednesday 01.03.2023                                 TIME: 9.00 A.M. – 5.00 P.M. 
Budalangi Mkt, Budalangi Vocational Training Inst, Port Victoria Mkt, Port Victoria 
Sub County Hosp, DCC’s Offices, Sifugwe Pri & adjacent customers. 
 
AREA: SIOPORT, BUMBE 
DATE: Friday 03.03.2023                                           TIME: 9.00 A.M. – 5.00 P.M. 
Sioport Mkt, Bumbe Technical, Bumbe Mkt, Sigalame & adjacent customers. 
 
PARTS OF VIHIGA COUNTY 
AREA: LIKINDU, GIVUDEMESI 
DATE: Thursday 02.03.2023                                     TIME: 8.30 A.M. – 5.00 P.M. 
Likindi Health Center, Kapkerer, Kinu, Givudemesi, Simbi, Karandini, Gimarani, 
Gamukuywa, Musiri, Jepsis, Dr. Maurice Dagana Sec & adjacent customers. 
 
AREA: KAIMOSI COMPLEX 
DATE: Friday 02.03.2023                                           TIME: 8.30 A.M. – 5.00 P.M. 
Kaimosi  Complex,  Jamlongo,  Muhudu,  Ivumbu,  Mahanga,  Part  of  Cheptulu, 
Vumavi & adjacent customers. 
 
PARTS OF KAKAMEGA COUNTY 
AREA: BUKURA 
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 4.00 P.M. 
Mutaho,  Musoli  Water,  Akatsa,  Bukura  College,  Bukura  Mkt,  Mwiyenga, 
Eshikomere, Kilimo Girls & adjacent customers. 
 
SOUTH NYANZA REGION 
 
PARTS OF MIGORI COUNTY 
AREA: OPAPO, NYANGAU 
DATE: Sunday 26.02.2023                                         TIME: 9.00 A.M. - 3.00 P.M. 
Opapo, Nyangau, Ndagoriedo, Sango, Miare, Komito, Nyaburu, Ofwanga, Opapo 
Sugar Research, Opapo Fisheries, Winyo, Kasere, Nyamuga, Ngodhe, Nyarach, 
Umbwa Kali, Rongo Town, Ogengo, Magena, Lwala, Ringa Kandongo, Ogango, 
Lwanda, Ngiya Parish & adjacent customers. 
 
PARTS OF NYAMIRA COUNTY 
AREA: TEA, MOGUSII, FARM 
DATE: Monday 27.02.2023                                       TIME: 10.00 A.M. - 4.00 P.M. 
Kipkebe  Tea,  Mogusii  Farm,  Chebilat  Mkt,  Mokomoni,  Stmathias,  Arroket  Tea, 
Monieri, Kitaru, Odieki Farm & adjacent customers. 
 
AREA: ISOGE MARKET, GONZA 
DATE: Friday 03.03.2023                                         TIME: 10.00 A.M. - 4.00 P.M. 
Isoge Mkt, Gonza, Itumbe Sec, Pator Mairura, Riontonyi Police, Nyaronde 
Mlimani, Nyaronde Pri & adjacent customers. 
 
PARTS OF HOMABAY COUNTY 
AREA: KIGWA MARKET 
DATE: Monday 27.02.2023                                       TIME: 10.00 A.M. - 4.00 P.M. 
Kigwa Mkt, Ruma National Park, Wiga Mkt & Schools, Nyadenda Mkt & adjacent 
customers. 
 
AREA: MIROGI MARKET 
DATE: Wednesday 01.03.2023                                TIME: 10.00 A.M. - 4.00 P.M.  
Mirogi Mkt, Mirogi Complex, Kodumba Mkt, Ruma National Park, Okok Sec Sch, 
Kobodo Mkt, Andiwo Mkt & adjacent customers. 
 
 
 
 
 PARTS OF KISII COUNTY 
AREA: KIAMOKAMA, KEUMBU 
DATE: Thursday 02.03.2023                                      TIME: 9.00 A.M. - 4.00 P.M.  
Keumbu Mkt, Kiamokama Fact, Nyamache Fact, Nyanturago Mkt, Kabosi, Ibeno, 
Borangi,  Maji  Mazuri,  Nyosia,  Kebuko,  Giosenseri,  Kiobegi,  Nyabisabo,  Kwa 
Monda, Riangabi, Igare, Emenwa, Boitang’are, Enchoro, Tukiamwana, Kionduso, 
Nyamache Mkt & adjacent customers. 
 
AREA: KAMEJI, TABAKA, MARYLAND 
DATE: Friday 03.03.2023                                            TIME: 9.00 A.M. - 4.00 P.M.  
Kameji, Misesi, Tabaka Mkt, Maryland Hosp, Kobado, Nyabigege, Kamagambo, 
Chico Quarry, Nyachenge, Kerina, Mesisita, Nyatike & adjacent customers. 
 
MT. KENYA REGION 
 
PARTS OF NYERI COUNTY 
AREA: KIMAHURI, NDATHI 
DATE: Wednesday 01.03.2023                                 TIME: 9.00 A.M. – 4.00 P.M. 
Kimahuri,  Mapema  Junction  Villa,  Kabaru,  Ndathi,  Mountain  Lodge  &  adjacent 
customers. 
 
AREA: KIRIAINI, KAMUNE 
DATE: Thursday 02.03.2023                                      TIME: 8.00 A.M. - 5.00 P.M. 
Gatugi  Mkt,  Waitima,  Mbari  ya  Ndiga,  Kiriaini  Town,  Munaini  DC’s  Office, 
Kamacharia, Kagumoini Mkt, Diara C/Fact, Kanjama Mkt, Kora Mkt, Kamune Mkt, 
Iruri  Mkt,  Thuita  Mkt,  Kiaga  C/Fact,  Karuthi  C/Fact  &  Sec,  Geitwa  Mkt,  Kariki, 
Giathugu Iruri, Mairo Mkt & adjacent customers. 
 
NORTH EASTERN REGION 
 
PARTS OF KIAMBU COUNTY 
AREA: KWA MAIKO 
DATE: Saturday 25.02.2023                                       TIME: 9.00 A.M. - 5.00 P.M. 
Kwa  Maiko  Shops,  Raini  Village,  Rayani  Village,  Kambui  Girls  Sec,  Mitahato 
Shops,  Gathirimu  Girls,  Riagithu  Village,  Mchana  Est,  Kofinaf  HQtrs  Offices  & 
adjacent customers. 
 
AREA: KIHARA, GACHIE 
DATE: Tuesday 28.02.2023                                        TIME: 9.00 A.M. - 5.00 P.M. 
Karura  Kanyungu,  Kihara  Hosp,  Honey  Bee,  Mahindi,  Wangunyu,  Karura 
Kagongo, Kihara High & adjacent customers. 
 
AREA: MEMBLEY, KIWANJA 
DATE: Tuesday 28.02.2023                                        TIME: 9.00 A.M. - 5.00 P.M. 
Referral Hospital IMIC Block, Wataalam, Githunguri Ranch, Whole of Membley 
Est, Gitambaya, Nyayo Hostel KU Kiwanja, Low Rates, Githunguri Pri, Membley 
PCEA, Membley High Sch, Membley Sweat Water, Rubies Petrol Stn & adjacent 
customers. 
 
AREA: RUAKA MKT, BORDER 
DATE: Thursday 02.03.2023                                      TIME: 9.00 A.M. - 5.00 P.M. 
Whole of Ruaka Mkt, Gertrude’s Ruaka, Quickmart Ruaka, Boarder, Slaughter, 
Kanungu, California Rd, Delta Ruaka, Royal Brain & adjacent customers. 
 
AREA: MUNYU, KOMO, FARA INYA 
DATE: Tuesday 02.03.2023                                        TIME: 9.00 A.M. - 5.00 P.M. 
Munyu Center, Fara Inya, Komo, St. Paul Secondary kwa Simon, Gichigi Village 
Spur,  Maganjo  S/Centre,  Ngiriki  S/Centre,  AIC  Church  Munyu  &  adjacent 
customers. 
 
AREA: GITHUNGURI SHOPS 
DATE: Friday 03.03.2023                                           TIME: 9.00 A.M. - 5.00 P.M. 
Thakwa Village, Mukua Vlllage, Ciiako Pri, Thuthuriki Village, Githunguri Police 
Stn, Lazinos Hotel, County Pride Hotel & adjacent customers. 
 
COAST REGION 
 
PARTS OF KWALE COUNTY 
AREA: PARTS OF KWALE 
DATE: Tuesday 28.02.2023                                   TIME: 10.00 A.M. – 12.00 P.M. 
Shamu,  Mbuwani,  Mabokoni,  Technical  University  of  Mombasa,  Dr.  Babla  Sec 
Sch, Buga, Mwajamba & adjacent customers. 
 
PARTS OF KILIFI COUNTY 
AREA: PARTS OF MTWAPA 
DATE: Sunday 26.02.2023                                        TIME: 9.00 A.M. – 5.00 P.M. 
La Marina, Ndodo, Maweni, Parts of Mtwapa Town & adjacent customers. 
 
AREA: PARTS OF MTWAPA,
COMSOR
DATE: Tuesday 28.02.2023                                       TIME: 9.00 A.M. – 3.00 P.M. 
Ogali’s,  Gassaro,  Aljazeera  Est,  Mwalimu  Omar,  Mzuri  Sweets,  Parts  of 
Mzambarauni,  Greenwood,  KMA,  Mwatundo,  Radar,  Tropical  Sea  Life,  Dada 
Millers, Parts of Kanamai & adjacent customers. 
           

                                                                                                    

For further information, contact 
the nearest Kenya Power office 
  Interruption notices may be viewed at 
www.kplc.co.ke 
        ";

        let results = scan(r);
        println!("{results:?}");
        let mut parser = Parser::new(results);

        let parsed_results = parser.parse();

        println!("{:?}", parsed_results);
        assert!(parsed_results.is_ok())
    }
}
