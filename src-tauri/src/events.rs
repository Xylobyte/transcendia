pub enum Events {
    OnOffConfigTrayItem,
}

impl Events {
    pub fn as_str(&self) -> &'static str {
        match self {
            Events::OnOffConfigTrayItem => "OnOffConfigTrayItem",
        }
    }
}
