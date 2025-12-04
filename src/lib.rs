#[cfg(feature = "futures")]
pub mod futuries;
pub mod macros;
#[cfg(feature = "smol")]
pub mod smol_streamies;
#[cfg(feature = "futures")]
pub mod streamies;
#[cfg(feature = "futures")]
pub mod try_streamies;

#[cfg(feature = "futures")]
pub use crate::futuries::*;
#[cfg(feature = "futures")]
pub use crate::streamies::*;
#[cfg(feature = "futures")]
pub use crate::try_streamies::*;
#[cfg(feature = "futures")]
pub use futures::{Stream, StreamExt, TryStream, TryStreamExt};
