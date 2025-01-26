pub enum Events {
    OnOffConfigTrayItem,
    RefreshOverlay,
    NewTranslatedText,
    DownloadProgress,
    StopDownload,
}

impl Events {
    pub fn as_str(&self) -> &'static str {
        match self {
            Events::OnOffConfigTrayItem => "OnOffConfigTrayItem",
            Events::RefreshOverlay => "RefreshOverlay",
            Events::NewTranslatedText => "NewTranslatedText",
            Events::DownloadProgress => "DownloadProgress",
            Events::StopDownload => "StopDownload",
        }
    }
}
