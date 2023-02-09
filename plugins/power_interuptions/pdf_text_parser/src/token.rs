pub struct Area {
    part: Option<String>,
    areas: Vec<String>,
    date: String,
    time: String,
    pins: Vec<String>,
}

pub enum Ast {
    Region(Vec<Area>),
}
