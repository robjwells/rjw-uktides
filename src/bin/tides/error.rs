use rjw_uktides::StationId;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum TidesError<'a> {
    Library(rjw_uktides::Error),
    Fetch(ureq::Error),
    NoSuchStation(&'a StationId),
}

impl<'a> core::error::Error for TidesError<'a> {}

impl<'a> std::fmt::Display for TidesError<'a> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
