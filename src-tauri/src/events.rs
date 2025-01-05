pub enum Events {
    OnOffConfigTrayItem,
    RefreshOverlay,
    NewTranslatedText,
}

impl Events {
    pub fn as_str(&self) -> &'static str {
        match self {
            Events::OnOffConfigTrayItem => "OnOffConfigTrayItem",
            Events::RefreshOverlay => "RefreshOverlay",
            Events::NewTranslatedText => "NewTranslatedText",
        }
    }
}
