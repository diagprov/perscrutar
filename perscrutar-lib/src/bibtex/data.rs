
use std::collections::HashMap;

pub enum BibType {
    Article,
    Book,
    InCollection,
    InProceedings,
    Misc,
    Report,
    Thesis,
    PhdThesis,
    MastersThesis,
}

pub struct Entry<'a> {
    itemtype : BibType,
    entries : HashMap<&'a str, &'a str>,
}
