use rjw_uktides::StationId;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum TidesError {
    Library(rjw_uktides::Error),
    Fetch(ureq::Error),
    NoSuchStation(StationId),
}

impl core::error::Error for TidesError {}

impl std::fmt::Display for TidesError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
