//! `.nmp` (taktakk Module Package) binary format.
//!
//! ## Wire format
//!
//! ```text
//! [0..4]   Magic bytes : 0x54 0x41 0x4B 0x54  ("TAKT")
//! [4]      Format version : u8  (must equal NMP_FORMAT_VERSION)
//! [5..9]   Manifest length : u32 big-endian
//! [9..9+M] Manifest JSON
//! [9+M..9+M+4]  Signature length : u32 big-endian (always 64 for Ed25519)
//! [9+M+4..9+M+68] Ed25519 signature over manifest bytes
//! [9+M+68..9+M+72] Object count : u32 big-endian
//! For each object:
//!   [0..4]  Object data length : u32 big-endian
//!   [4..4+N] Raw object bytes
//! ```

pub mod error;
pub mod manifest;
pub mod reader;
pub mod stream_reader;
pub mod writer;

pub use error::{ContentError, ContentResult};
pub use reader::NmpReader;
pub use writer::NmpWriter;
