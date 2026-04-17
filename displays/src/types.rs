//! Convenience re-exports for users of the `displays` crate.
//!
//! Internal `displays` crate code should import these types from their owning
//! crates instead of using this module.

pub use displays_logical_types::{
    LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState, LogicalDisplayUpdate,
    LogicalDisplayUpdateContent,
};
pub use displays_physical_types::{
    PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState, PhysicalDisplayUpdate,
    PhysicalDisplayUpdateContent,
};
pub use displays_types::{
    Brightness, DisplayIdentifier, DisplayIdentifierInner, Orientation, PixelFormat, Point,
};
