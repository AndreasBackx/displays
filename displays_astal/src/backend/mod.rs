#[cfg(feature = "faked")]
pub mod fake;
#[cfg(not(feature = "faked"))]
pub mod real;
pub mod types;

use glib::Error;

/// Shared backend contract for the GI-facing manager.
///
/// The public GObject API talks only in terms of the normalized `types::*`
/// structs below, which allows the crate to swap between the real platform
/// backend and the deterministic fake backend without changing the exported GI
/// surface.
pub trait Backend {
    fn query(&self) -> Result<Vec<types::DisplayData>, Error>;
    fn get(
        &self,
        ids: Vec<types::DisplayIdentifierData>,
    ) -> Result<Vec<types::DisplayMatchData>, Error>;
    fn apply(
        &self,
        updates: Vec<types::DisplayUpdateData>,
        validate: bool,
    ) -> Result<Vec<types::DisplayUpdateData>, Error>;
}

/// Returns the backend that should satisfy manager requests for this build.
///
/// The `faked` Cargo feature is intended for smoke testing, typelib iteration,
/// and development without touching real displays. Without that feature, the
/// library delegates to the real `displays` crate.
pub fn active() -> &'static dyn Backend {
    #[cfg(feature = "faked")]
    static BACKEND: fake::FakeBackend = fake::FakeBackend;

    #[cfg(not(feature = "faked"))]
    static BACKEND: real::RealBackend = real::RealBackend;

    &BACKEND
}
