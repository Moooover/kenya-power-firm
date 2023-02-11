pub struct Area {
    // not so sure about this name, but it makes the most sense right now.
    lines: Vec<String>,
    date: String,
    time: String,
    pins: Vec<String>,
}

pub struct County {
    name: String,
    areas: Vec<Area>,
}

pub struct Region {
    pub name: String,
    pub counties: Vec<County>,
}
