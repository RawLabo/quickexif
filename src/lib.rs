//! A rust library to parse EXIF data from images.
//!
//! ### Why another EXIF parsing library?
//! Most EXIF libraries parse all the tags, whether they are needed or not, which consumes unnecessary memory and cpu resources.
//! But **quickexif** parses the only needed tags to save computational resources during large-scale images EXIF extraction.
//! 
//! ### Example
//! ```no_run
//! let sample = std::fs::read("sample.JPG").unwrap();
//! let rule = quickexif::describe_rule!(tiff {
//!     0x010f {
//!         str + 0 / make
//!     }
//!     0x8769 {
//!         0xa002 / width
//!         0xa003 / height
//!     }
//! });
//! 
//! let parsed_info = quickexif::parse(&sample, &rule).unwrap();
//! 
//! let make = parsed_info.str("make").unwrap();
//! let width = parsed_info.u32("width").unwrap();
//! let height = parsed_info.u32("height").unwrap();
//! ```
//! 
pub mod rule;
pub mod parsed_info;
pub mod parser;
pub mod value;
mod utility;

pub use parser::parse as parse;