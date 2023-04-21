use multipeek::{multipeek, MultiPeek};
use std::collections::HashMap;
use std::iter;
use std::str::Chars;

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref MATCH_ADJUSCENT_CUSTOMERS: Regex =
        RegexBuilder::new(r"(&|and)[\n\r\s]+adjacent[\n\r\s]+customers?\.*")
            .case_insensitive(true)
            .build()
            .expect("Expected MATCH_ADJUSCENT_CUSTOMERS regex to compile ");
}
const END_OF_LOCATIONS: &str = "ENDOFLOCATIONS";

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Time {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    day_of_the_week: String,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyWords {
    EndOfAreaLocations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Comma,
    Identifier(String),
    Keyword(KeyWords),
    Region(String),
    County(String),
    Area(String),
    Date(Date),
    Time(Time),
}

pub struct Scanner<'a> {
    source: MultiPeek<Chars<'a>>,
    current_lexeme: String,
}

fn is_digit(c: char) -> bool {
    ('0'..='9').contains(&c)
}

fn is_alpha(c: char) -> bool {
    ('a'..='z').contains(&c)
        || ('A'..='Z').contains(&c)
        || ['.', '-', '_', '&', ':', ';', '(', ')', '’', '\''].contains(&c)
}

fn is_alphanumeric(c: char) -> bool {
    is_digit(c) || is_alpha(c)
}

fn is_nextline(c: char) -> bool {
    matches!(c, '\n')
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\r' | '\t' | '\n') || c.is_whitespace()
}

fn is_white_space_or_new_line(c: char) -> bool {
    is_whitespace(c) || is_nextline(c)
}

fn is_not_dash(c: char) -> bool {
    !['-', '–'].contains(&c)
}

fn is_not_new_line(c: char) -> bool {
    !is_nextline(c)
}

fn is_colon_or_semi_colon(c: char) -> bool {
    matches!(c, ';' | ':')
}

fn add_nairobi_county_to_text(raw_text: &str) -> String {
    let nairobi_region = "NAIROBI REGION";
    let nairobi_county = "PARTS OF NAIROBI COUNTY";
    let nairobi_region_and_country = format!(
        r"{nairobi_region}
           {nairobi_county}
        "
    );
    raw_text.replacen(nairobi_region, &nairobi_region_and_country, 1)
}

impl<'a> Scanner<'a> {
    fn new(raw_text: &'a str) -> Self {
        let source = multipeek(raw_text.chars());
        Self {
            source,
            current_lexeme: Default::default(),
        }
    }

    fn advance(&mut self) -> Option<char> {
        let next = self.source.next();
        if let Some(c) = next {
            self.current_lexeme.push(c);
        }
        next
    }

    fn peek_check(&mut self, check: &dyn Fn(char) -> bool) -> bool {
        match self.source.peek() {
            Some(&c) => check(c),
            None => false,
        }
    }

    fn advance_while(&mut self, condition: &dyn Fn(char) -> bool) {
        while self.peek_check(condition) {
            self.advance();
        }
    }

    fn advance_but_discard(&mut self, condition: &dyn Fn(char) -> bool) {
        while self.peek_check(condition) {
            self.source.next();
        }
    }

    fn identifier_or_keyword(&mut self) -> Option<Token> {
        self.advance_while(&is_alphanumeric);
        let acronym_map = HashMap::from([
            ("pri", "Primary"),
            ("rd", "Road"),
            ("est", "Estate"),
            ("sch", "School"),
            ("sec", "Secondary"),
            ("stn", "Station"),
            ("apts", "Apartments"),
            ("hqtrs", "Headquaters"),
            ("mkt", "Market"),
        ]);
        let token = match self.current_lexeme.as_ref() {
            "DATE:" | "DATE;" => Token::Date(self.date()),
            "TIME" | "TIME:" | "TIME;" => Token::Time(self.time()),
            "AREA:" | "AREA;" => Token::Area(self.area()),
            END_OF_LOCATIONS => Token::Keyword(KeyWords::EndOfAreaLocations),
            _ => self
                .peek_and_check_for_region_or_county()
                .unwrap_or_else(|| {
                    let current_lexeme = self.current_lexeme.clone();
                    let identifier = acronym_map
                        .get(current_lexeme.to_ascii_lowercase().as_str())
                        .cloned()
                        .unwrap_or(&current_lexeme)
                        .to_string();
                    Token::Identifier(identifier)
                }),
        };

        Some(token)
    }

    fn advance_n_times_and_clear_lexeme(&mut self, position: usize) {
        for _ in iter::repeat(()).take(position) {
            self.source.next();
        }
        self.current_lexeme.clear();
    }
    fn peek_and_check_for_region_or_county(&mut self) -> Option<Token> {
        let mut position = 0;
        let mut buffer = String::new();
        while self.source.peek_nth(position) != Some(&'\n') {
            buffer.push(*self.source.peek_nth(position)?);
            position += 1;
        }

        let buffer = buffer.trim();

        if buffer.ends_with("REGION") {
            let whole_match = format!("{} {}", &self.current_lexeme, buffer);
            self.advance_n_times_and_clear_lexeme(position);
            return Some(Token::Region(whole_match));
        }

        if buffer.ends_with("COUNTY") {
            let whole_match = format!("{} {}", &self.current_lexeme, buffer);
            self.advance_n_times_and_clear_lexeme(position);
            return Some(Token::County(whole_match));
        }

        None
    }

    fn does_next_keyword_match(&mut self, keyword: &str) -> bool {
        let mut idx = 0;
        let keyword_len = keyword.len();
        let mut buffer = String::new();
        while idx < keyword_len {
            if let Some(c) = self.source.peek_nth(idx) {
                buffer.push(*c);
                idx += 1;
            }
        }
        keyword.eq_ignore_ascii_case(&buffer)
    }

    fn date(&mut self) -> Date {
        self.current_lexeme.clear();
        self.advance_but_discard(&is_whitespace);
        self.advance_while(&is_alpha);
        let day_of_the_week = self.current_lexeme.clone();
        self.current_lexeme.clear();
        self.advance_but_discard(&is_white_space_or_new_line);
        loop {
            self.advance_while(&is_alphanumeric);
            self.advance_but_discard(&is_white_space_or_new_line);
            if self.does_next_keyword_match("TIME") {
                break;
            }
        }

        let date = self.current_lexeme.clone();

        Date {
            day_of_the_week,
            date,
        }
    }

    fn time(&mut self) -> Time {
        self.current_lexeme.clear();
        self.advance_but_discard(&is_colon_or_semi_colon);
        self.advance_while(&is_not_dash);
        // skip the dash
        self.source.next();
        let from = self.current_lexeme.trim().to_owned();
        self.current_lexeme.clear();
        self.advance_while(&is_not_new_line);
        let to = self.current_lexeme.trim().to_owned();

        Time {
            start: from,
            end: to,
        }
    }

    fn area(&mut self) -> String {
        self.current_lexeme.clear();

        loop {
            self.advance_while(&is_not_new_line);
            self.advance_but_discard(&is_whitespace);
            if self.does_next_keyword_match("DATE") {
                break;
            }
        }

        let area = self.current_lexeme.to_owned();
        area.replace("PART OF", "").replace("PARTS OF", "")
    }

    fn scan_next(&mut self) -> Option<Token> {
        self.current_lexeme.clear();

        self.advance_but_discard(&is_white_space_or_new_line);

        let next_char = match self.advance() {
            Some(c) => c,
            None => return None,
        };

        let token = match next_char {
            ',' => Token::Comma,
            _ => self.identifier_or_keyword()?,
        };

        Some(token)
    }
}

pub fn scan(text: &str) -> Vec<Token> {
    let raw_text = add_nairobi_county_to_text(text);
    let text = MATCH_ADJUSCENT_CUSTOMERS.replace_all(&raw_text, END_OF_LOCATIONS);
    let scanner = ScannerIter {
        scanner: Scanner::new(&text),
    };
    scanner.into_iter().collect()
}

struct ScannerIter<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Iterator for ScannerIter<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.scanner.scan_next()
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::{is_alphanumeric, is_whitespace, scan};

    #[test]
    fn test_alphanumeric() {
        println!("{}", is_alphanumeric('&'))
    }

    #[test]
    fn test_scanned_text() {
        let text = r"Interruption";

        let result = scan(text);

        println!("{:?}", result)
    }

    #[test]
    fn test_white_space() {
        println!("{}", is_whitespace('\n'))
    }

    #[test]
    fn test_tokenizing() {
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
 
AREA: PART OF MTWAPA,
COMSOR
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

        let result = scan(r);

        println!("{result:?}")
    }
}
