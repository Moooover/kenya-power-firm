use crate::pdf_extractor::TextExtractor;
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::error::Error;

use use_cases::import_planned_blackouts::Area;

lazy_static! {
    static ref INTERUPTION_TEXT: Regex = RegexBuilder::new(r"(Interruption)[\s\S]*(etc\.\))")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .expect("Expected regex to compile interruptions text pattern");
    static ref LINE_BREAK_REMOVING_REGEX: Regex = Regex::new(r"[\r\n]+").unwrap();
    static ref REGION: Regex = RegexBuilder::new(r"(?P<region>\w+ region)+")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .expect("Expected regex to build and compile the region pattern");
}

pub struct TextExtractorImpl {}

#[async_trait]
impl TextExtractor for TextExtractorImpl {
    async fn extract(&self, text: String) -> anyhow::Result<Vec<Area>> {
        let regions = self.split_by_region(&text);
        println!("{regions:?}");
        let result = regions
            .into_iter()
            .map(Self::extract_many_areas)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(result.into_iter().flatten().collect())
    }
}

impl TextExtractorImpl {
    fn split_by_region(&self, text: &str) -> Vec<&str> {
        for caps in REGION.captures_iter(text) {
            println!("{caps:?}");
        }
        vec![]
    }

    fn split_by_area(region: &str) -> Vec<&str> {
        todo!()
    }

    fn extract_many_areas(region: &str) -> anyhow::Result<Vec<Area>> {
        for area in Self::split_by_area(region) {}
        todo!()
    }

    fn extract_single_area(area: &str) -> anyhow::Result<Area> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf_extractor::TextExtractor;
    use crate::text_extractor::TextExtractorImpl;

    const TEXT: &str = r"




Interruption of
Electricity Supply
Notice is hereby given under Rule 27 of the Electric Power Rules
That the electricity supply will be interrupted as here under:
(It  is  necessary  to  interrupt  supply  periodically  in  order  to
facilitate maintenance and upgrade of power lines to the network;
to connect new customers or to replace power lines during road
construction, etc.)

NAIROBI REGION

AREA: GRAIN BULK
DATE: Sunday 05.02.2023                                  TIME: 9.00 A.M. – 5.00 P.M.
Heavy Engineering, Grain Bulk Handlers, Posh Auto body, SGR Head office
& adjacent customers.

AREA: REDHILL ROAD
DATE: Tuesday 07.02.2023                         TIME: 9.00 A.M. – 5.00 P.M.
Redhill Rd, Rosslyn Green, Part of Nyari, Embassy of Switzerland, Gachie,
Karura  SDA  Church,  Hospital  Hill  Sec  Sch,  Commission   for  University
Education, Trio Est & adjacent customers.

AREA: KIHARA, KINANDA
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 5.00 P.M.
Kihara  Village,  Old  Karura  Rd,  Kihara  Mkt,  Kitsuru  Ridge  Villas  Est, White
Cottage Sch, Weaverbird Kenya, Part of Kirawa Rd, Kitsuru Country Homes,
Hotani Close, Parazoro Institute & adjacent customers.

AREA: GACHIE, NGECHA ROAD
DATE: Tuesday 07.02.2023                              TIME: 9.00 A.M. – 12.00 P.M.
Thigiri  Groove,  Thigiri  Ridge,  NSIS  Training,  Brookside  Drive  &  adjacent
customers.

AREA: ROSSLYN
DATE: Tuesday 07.02.2023                              TIME: 9.00 A.M. – 12.00 P.M.
Rosslyn Close, Thigiri Farm, Nyari Est, Rosslyn Heights Est, Redhill Drive &
adjacent customers.

AREA: PARTS OF UMOJA INNERCORE
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 5.00 P.M.
Oloibon Hotel, Unity Pri, Cathsam Sch, Inner Core Sango, Kwa Chief Umoja
Office, Umoja 2 Mkt & adjacent customers.

AREA: PART OF RING ROAD KILIMANI
DATE: Tuesday 07.02.2023                      TIME: 9.00 A.M. – 5.00 P.M.
Chania  Ave,  Part  of  Ring  Rd  Kilimani,  Mugo  Kabiru  Rd,  Woodley,  Joseph
Kangethe, Prestige, Jamhuri Est, Toi Pri Sch, Moi Girls High Sch & adjacent
customers.

AREA: MUKURU
DATE: Wednesday 08.02.2023                           TIME: 9.00 A.M. – 5.00 P.M.
Maziwa Stage, Ebeneza, AA of Kenya, Wapewape, Mukuru Kwa Njenga, Our
Lady  of  Nazareth,  Kware  Police,  Kwa  Chairman,  Plot  10,  State  House  &
adjacent customers.

AREA: PART OF WESTLANDS
DATE: Thursday 09.02.2023                               TIME: 9.00 A.M. – 5.00 P.M.
Part  of  Kolobot  Rd,  Taarifa  Suites,  Part  of  Ojijo  Rd,  Part  of  Muthithi  Rd,
Chiromo Lane, Part of Westlands Rd, Part of Mpaka Rd, Woodvale Groove
& adjacent customers.

AREA: ARBORETUM DRIVE, PART OF RIVERSIDE DRIVE
DATE: Thursday 09.02.2023                               TIME: 9.00 A.M. – 5.00 P.M.
Statehouse  Girls,  Part  of  Statehouse  Crescent,  Arboretum  Drive,  Kolobot
Drive,  Confucius  Institute,  Arboretum  Court,  Part  of  Riverside
Drive, Riverside Gardens, SBM Bank, Prime Bank Riverside Drive, Dusitd2,
UoN Chiromo Campus, Chiromo Mortuary, ICEA Lion & adjacent customers.

AREA: PART OF KILIMANI
DATE: Thursday 09.02.2023                               TIME: 9.00 A.M. – 5.00 P.M.
Nairobi  Baptist  Church  Ngong  Rd,  Part  of  Ngong  Rd,  Kindaruma  Lane,
Muchai Drive, Maki Apts, Casamia Apats, Kingston Apts, Jameson Court, Ola
Energy,  Rose  of  Sharon  Sch,  St.  Nicholas  High  Sch,  Nursing  Council  of
Kenya, Coptic Hosp, Kabarnet Rd & adjacent customers.







PARTS OF MAKUENI COUNTY
AREA: SULTAN TO MUANGINI, SULTAN
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 5.00 P.M.
Sultan  Mosque,  Sub-County  Hosp,  Part  of  Sultan  Town,  Enguli,  Kasikeu,
Nduluni, Mutyambua, Barazani, Kitivo, Muangini & adjacent customers.

PARTS OF KAJIADO COUNTY
AREA: RONGAI
DATE: Thursday 09.02.2023                              TIME: 9.00 A.M. – 5.00 P.M.
Maxwell  Univ,  Acacia  Est,  Olesakunda,  Magenche,  Exciting,  Kimandiro,
Total, Laiser Hill, Nairobi Women Hosp, Fatima Hosp, Fatima South & North,
Part of Kware, Tuskys, Kingdom Hall, Wama Hosp, Rongai Town, Sinai Hosp
& adjacent customers.

NORTH RIFT REGION

PARTS OF UASIN GISHU COUNTY
AREA: KIPKORGOT, HILLSIDE
DATE: Wednesday 08.02.2023                         TIME: 10.00 A.M. – 2.00 P.M.
Hillside,  Kipkorgot,  Naiberi  Center,  Plateau,  Duka  Moja  &  adjacent
customers.

PARTS OF NANDI COUNTY
AREA: CHEPKUNYUK, KAPCHURIA
DATE: Monday 06.02.2023                              TIME: 10.00 A.M. – 12.00 P.M.
Chepkunyuk  Center,  St.  John’s  Chepkunyuk  Sec,  Kapchuria  Center  &
adjacent customers.

AREA: BARATON UNIVERSITY
DATE: Monday 06.02.2023                                TIME: 10.00 A.M. – 3.00 P.M.
Baraton  Univ,  Kapdildil,  Samoo,  Kipchabo  Fact,  Kipsomoiite,  Chemuswa,
Sironoi, Talai, Kimondi & adjacent customers.

AREA: LESSOS, KOILOT
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 4.00 P.M.
Lessos Town, Lolduga, Sochoi, Cheboror, Kesses, Koisagat Pri Sch, Koilot,
Mogobich, Kimogoch & adjacent customers.

AREA: NDALAT, CENTER KWANZA
DATE: Thursday 09.02.2023                             TIME: 10.00 A.M. – 3.00 P.M.
Sigot,  Cheptil,  Kapsato,  Ndalat,  Kabiemit,  Center  Kwanza,  Malando,
Chepkemel, Tuktuk & adjacent customers.

PARTS OF TRANS NZOIA COUNTY
AREA: KAPSARA, SIBANGA
DATE: Friday 10.02.2023                                    TIME: 9.00 A.M. – 2.00 P.M.
Kapsara,  Chebarus,  Kabolet,  Makoi,  Chisare,  Ngonyek,  Minex,  Sibanga,
Sirwo Resort, Namba Nne, Kirita, Sitatunga, Cooperative, Marura, Mukuyu,
Legacy,  Chisare,  Sirwo  Resort,  Noigam,  West  Pokot  County  &  adjacent
customers.

PARTS OF TURKANA COUNTY
AREA: KALEMNGOROK, KATILU
DATE: Saturday 04.02.2023                                TIME: 9.00 A.M. – 4.00 P.M.
Kalemngorok  Center,  Kalemngorok  Safaricom  Boosters,  Katilu  Hosp,
Lokichar Center, Hospital & Safaricom Booster & adjacent customers.



















For further information, contact
the nearest Kenya Power office
  Interruption notices may be viewed at
www.kplc.co.ke




Interruption of
Electricity Supply
Notice is hereby given under Rule 27 of the Electric Power Rules
That the electricity supply will be interrupted as here under:
(It  is  necessary  to  interrupt  supply  periodically  in  order  to
facilitate maintenance and upgrade of power lines to the network;
to connect new customers or to replace power lines during road
construction, etc.)

SOUTH NYANZA REGION

PARTS OF KISII COUNTY
AREA: OGEMBO, TABAKA
DATE: Wednesday 08.02.2023                           TIME: 9.00 A.M. - 3.00 P.M.
Ogembo Law Courts, Nyabisiongororo, Kodero Bara, Riosiri, Riosiri Sec Sch,
Riosiri  W/Pump,  Nyabigena  Sec  Sch,  Nyabigena  Mkt,  Nyakiembene,
Nyandiwa Pri Sch, Kiabigoria, Gotichaki, Part of Tabaka Mkt, Tabaka Orwaki
Pri Sch, Sae Pri Sch & adjacent customers.

AREA: RAIBAMANYI, ITUMBE
DATE: Sunday 12.02.2023                                 TIME: 9.00 A.M. - 3.00 P.M.
Kegati,  Riabamanyi,  Mativo,  Kiogoro,  Camel  Park,  Itumbe  T/Fact,  Itumbe
Mkt, Rianyamwamu T/Fact & adjacent customers.

PARTS OF NYAMIRA COUNTY
AREA: CHEPILAT, KIJAURI
DATE: Wednesday 08.02.2023                           TIME: 9.00 A.M. - 3.00 P.M.
Chebilat,  Kijauri,  Nyaronde,  Highland  Creamers,  Nyasiongo  Tea,  Matutu,
Gesima, Manga Laitigo, Nyasiongo Mission & adjacent customers.

AREA: KEBUKO, NYAMBOGO
DATE: Friday 10.02.2023                                     TIME: 9.00 A.M. - 3.00 P.M.
Kianungu, Marani Girls, Sirate, Nyambogo Pri, Ekoro, Kenyamware, Kebuko,
Kianginda Sec & adjacent customers

WESTERN REGION

PARTS OF SIAYA COUNTY
AREA: HONO
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 2.00 P.M.
Hono Mkt, Nyakongo, Gombe & adjacent customers.

AREA: MALUMBE
DATE: Thursday 09.02.2023                               TIME: 9.00 A.M. – 2.00 P.M.
Pisoko  Water  Plant,  Rabar  Mkt,  Malumbe  Sec  Sch,  Kalkada,  Nyawita,
Kodiere, Wang’ Chieng’ & adjacent customers.

PARTS OF BUSIA COUNTY
AREA: BUTULA
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 5.00 P.M.
Burinda,  Igula,  Murumba,  Kuhunyango  Sub  County  Hosp,  Mabunge,
Bumutiru,  Kingandole,  Sikoma,  Nela,  Umer,  Barober,  Butula  Sub  County
Offices, Timbolo, Bulwani, Bualiro, Kanjala & adjacent customers.

AREA: MUNDIKA
DATE: Thursday 09.02.2023                               TIME: 9.00 A.M. – 5.00 P.M.
Sherman,  Sirwa  Millers,  Mundika,  Nambuko,  Kabuodo,  Murende,  Nasewa,
Matayos, Bumala, Namboboto & adjacent customers.

PARTS OF VIHIGA COUNTY
AREA: GAMBOGI
DATE: Wednesday 08.02.2023                           TIME: 8.30 A.M. – 5.00 P.M.
Part of Gambogi Mkt, Jebrok, Kinu Mkt, Mutave, Ivudemesi, Simbi, Karandini,
Kimarani, Gamukuywa, Musiri, Jepsis & adjacent customers.

NORTH EASTERN REGION

PARTS OF KIAMBU COUNTY
AREA: KINGEERO, WANGIGE
DATE: Sunday 05.02.2023                                   TIME: 7.00 A.M. - 5.00 P.M.
Njonjo Farm, Gathiga Village, Kessel Homes, Epic, Mwimuto, Wangige Mkt,
Ndumbuini  Mkt,  Uthiru  Gichagi,  Genesis,  Kanyariri,  Fort  Smith,  Gachio,
Kamutiini, Mararo, Kwa Ndume & adjacent customers.












AREA: MATANGI, NDARASHA
DATE: Tuesday 07.02.2023                      TIME: 9.00 A.M. – 5.00 P.M.
Matangi  S/Centre,  Spirit  of  Faith,  Kumura,  Kwa  Mundia,  Kwa  Tom,
Kariaini, Ndarasha,  Matangi  Junction,  Judah,  Magomano,  Juja  Farm,  Juja
South, Space & Style, Mastore, Juja Athi & adjacent customers.

PARTS OF KITUI COUNTY
AREA: KITUI TOWN, MUTUNE
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 5.00 P.M.
Kunda Kindu, Wikililye, Mulango, Nzambani, Nzangathi, Miambani, Zombe,
Kisasi, Mbitini, Mosa, Ikanga, Voo, Kinakoni, Kitui Teachers Sacco, Oil Libya
Kitui, Kobil Kitui, KIE, Slaughter House Kitui, Kiembeni, Kalundu Mkt, Ngiini,
Mutune Mkt, St. Angelus Sec Sch, Kwa Gindu Mkt, Kwa Nzao Pri, Museve
Mkt,  Kwa  Mutheke,  Kwa  Ukungu,  Kyalilini,  Kitui  CBD,  Kitui  General  Hosp,
Site,  JICA,  KEFRI,  Kwa  Ngindu,  Kitui  Prison,  Kitui  Police  Stn,  Kitui  Total
Petrol Stn, Law Courts, County Assembly, Governor's Office, Nzeu, County
Commissioner’s Office, Muslim Sec, Kitui High Sch, Majengo, Showground,
Ithokwe, Kalawa & adjacent customers.

COAST REGION

PARTS OF MOMBASA COUNTY
AREA: PART OF SHANZU
DATE: Thursday 09.02.2023                        TIME: 8.00 A.M. – 5.00 P.M.
Flamingo Beach Hotel, Kilua Hotel, Royal Shaza, Ngomongo, Shanzu Navy,
Kwa Rodney, Shimo la Tewa Sch, Shanzu Teachers, Shimo la Tewa Prison
Staff Quarters, Shimo la Tewa Women’s Prison & adjacent customers.

PARTS OF KILIFI COUNTY
AREA: PART OF KANAMAI
DATE: Tuesday 07.02.2023                         TIME: 8.00 A.M. – 5.00 P.M.
Mega  Apparel,  Afriwear,  Pride  Industries,  Safepack,  Jallaram  Plastics,  My
Pasta  Ltd,  Revital  EPZ,  Bodoi  Stage,  Afriware,  Total  Amkeni  &  adjacent
customers.

AREA: PART OF MTWAPA
DATE: Thursday 09.02.2023                               TIME: 8.00 A.M. – 5.00 P.M.
La  Marina,  Ndodo,  Matundura,  Maweni,  Bandari,  Coba  Cabana,  Bahari
Parents, Faga Faga, Mtwapa Luxury, Quickmart, Petrocity, Pizza Inn, Furaha
Academy, Part of Mtwapa Town & adjacent customers.






























For further information, contact
the nearest Kenya Power office
  Interruption notices may be viewed at
www.kplc.co.ke";
    #[tokio::test]
    async fn test_extraction() {
        let extractor = TextExtractorImpl {};
        extractor.extract(TEXT.to_owned()).await;
    }
}
