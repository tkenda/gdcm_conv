//! An easy-to-use Grassroots DICOM Library wrapper designed to convert DICOM files transfer syntaxes and photometric interpretation.
//!
//! # Usage
//!
//! You need CMake to build [GDCM Library](http://gdcm.sourceforge.net/wiki).
//!
//! ### Linux Ubuntu:
//!
//! ```cmd
//! sudo apt-get install cmake
//! ```
//!
//! ### Windows & MacOS:
//!
//! Download CMake directly from [www.cmake.org/download](https://cmake.org/download/) page.
//!
//! ## Quickstart
//!
//! Copy this code and make sure you have a DICOM file to test ([DICOM file samples](https://support.dcmtk.org/redmine/projects/dcmtk/wiki/DICOM_images)).
//!
//! ```rust
//! use std::io::prelude::*;
//! use std::fs::File;
//! use gdcm_conv::{TransferSyntax, PhotometricInterpretation};
//!
//! // Read input file
//! let mut ibuffer = Vec::new();
//! let mut ifile = File::open("test.dcm").unwrap();    
//! ifile.read_to_end(&mut ibuffer).unwrap();
//!
//! // Transcode DICOM file
//! let obuffer = match gdcm_conv::pipeline(
//!     // Input DICOM file buffer
//!     ibuffer,
//!     // Estimated Length
//!     None,
//!     // First Transfer Syntax conversion
//!     TransferSyntax::JPEG2000Lossless,
//!     // Photometric conversion
//!     PhotometricInterpretation::None,
//!     // Second Transfer Syntax conversion
//!     TransferSyntax::None,
//! ) {
//!     Ok(t) => t,
//!     Err(e) => {
//!         eprintln!("{}", e);
//!         return;
//!     }
//! };
//!
//! // Create output file and save
//! let mut ofile = File::create("output.dcm").unwrap();
//! ofile.write_all(&obuffer).unwrap();
//! ```
//!
//! ## How it works
//!
//! The gdcm_conv library takes as input the content of the DICOM file. It reuse the source vector allocating an estimated
//! size to avoid cloned memory. The default estimad length is 3 times the input file size, the worst case, changing from
//! a compressed image (like JPEG2000) to raw. Is recommended to use an estimated calculation, to minimize memory allocation.
//! 
//! If the allocated size is not enough, the library will re-allocate to the correct size and execute the FFI function again.
//!
//! To estimate the output length you could use this aproximation:
//!
//! - (0028,0100) bits_allocated
//! - (0028,0004) photometric_interpretation
//! - (0028,0008) number_of_frames
//! - (0028,0010) rows
//! - (0028,0011) columns
//!
//! ```
//! // MAX HEADER SIZE
//! const MAX_HEADER_SIZE: usize = 5000;
//!
//! let a = match bits_allocated {
//!     8 => 1,
//!     16 => 2,
//! };
//!
//! let b = match photometric_interpretation {
//!     "MONOCHROME1" => 1,
//!     "MONOCHROME2" => 1,
//!     _ => 3,
//! };
//!
//! let estimad_length = (a * b * rows * columns * number_of_frames) + MAX_HEADER_SIZE;
//! ```
//! 
//! The library works as a pipeline with a first transfer syntax conversion (PRE-TRANSFER), a photometric conversion 
//! and a final transfer syntax conversion (POST-TRANSFER). If you set to None it don't execute the step. 
//! Usually, you will use only the first and/or second step.
//!
//! I setup this way because in some cases is needed two transfer syntax transcoding like this example:
//! 
//! The conversion from JPEG Baseline (Process 1) 1.2.840.10008.1.2.4.50 with YBR_FULL or YBR_FULL_422 to JPEG2000
//! lossles, you need to change to Explicit Little Endian transfer syntax, then to an RGB photometric interpretation and
//! finally to JPG2000, to avoid GDCM color interpretation issue.
//!

use libc::{c_char, c_int, c_uchar, c_uint, size_t};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GDCMError {
    #[error("Unknown error.")]
    Unknown,
    #[error("Input buffer pointer is NULL.")]
    PointerNULL,
    #[error("Empty input buffer.")]
    EmptyBuffer,
    #[error("[GDCM PRE] {0}")]
    Pre(Error),
    #[error("[GDCM PHOTO] {0}")]
    Photo(Error),
    #[error("[GDCM POST] {0}")]
    Post(Error),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not read stream.")]
    ReadStream,
    #[error("Invalid photometric interpretation.")]
    InvalidPhotometricInterpretation,
    #[error("Could not execute change.")]
    ExecuteChange,
    #[error("Could not execute LUT change.")]
    ExecuteLUTChange,
    #[error("Could not write stream.")]
    WriteStream,
    #[error("Could not execute FileExplicitFilter.")]
    FileExplicitFilter,
    #[error("Invalid transfer syntax.")]
    InvalidTransferSyntax,
    #[error("Could not derive file.")]
    DeriveFile,
}

#[derive(Copy, Clone, Debug)]
pub enum TransferSyntax {
    None,
    /// [1.2.840.10008.1.2] Implicit VR Endian: Default Transfer Syntax for DICOM.
    ImplicitVRLittleEndian,
    /// [1.2.840.10008.1.2.1] Explicit VR Little Endian.
    ExplicitVRLittleEndian,
    /// [1.2.840.10008.1.2.2] Explicit VR Big Endian.
    ExplicitVRBigEndian,
    /// [1.2.840.10008.1.2.5] RLE Lossless.
    RLELossless,
    /// [1.2.840.10008.1.2.4.50] JPEG Baseline (Process 1): Default Transfer Syntax for Lossy JPEG 8-bit Image Compression
    /// (Process 4 only). Input parameter: (quality).
    JPEGBaselineProcess1(u32),
    /// [1.2.840.10008.1.2.4.51] JPEG Baseline (Processes 2 & 4): Default Transfer Syntax for Lossy JPEG 12-bit Image Compression.
    /// Input parameter: (quality).
    JPEGExtendedProcess2_4(u32),
    /// [1.2.840.10008.1.2.4.57] JPEG Lossless, Nonhierarchical (Processes 14).
    JPEGLosslessProcess14,
    /// [1.2.840.10008.1.2.4.70] JPEG Lossless, Nonhierarchical, First- Order Prediction (Processes 14 [Selection Value 1]):
    /// Default Transfer Syntax for Lossless JPEG Image Compression.
    JPEGLosslessProcess14_1,
    /// [1.2.840.10008.1.2.4.80] JPEG-LS Lossless Image Compression.
    JPEGLSLossless,
    /// [1.2.840.10008.1.2.4.81] JPEG-LS Lossy (Near- Lossless) Image Compression.
    /// Input parameter: (allow_error).
    JPEGLSNearLossless(u32),
    /// [1.2.840.10008.1.2.4.90] JPEG 2000 Image Compression (Lossless Only).
    JPEG2000Lossless,
    /// [1.2.840.10008.1.2.4.91] JPEG 2000 Image Compression.
    /// Input parameters: (quality1, quality2, quality3, irreversible)
    JPEG2000(u32, u32, u32, bool),
    /// [1.2.840.10008.1.2.4.92] JPEG 2000 Part 2 Multicomponent Image Compression (Lossless Only).
    JPEG2000Part2Lossless,
    /// [1.2.840.10008.1.2.4.93] JPEG 2000 Part 2 Multicomponent Image Compression.
    /// Input parameters: (quality1, quality2, quality3, irreversible)
    JPEG2000Part2(u32, u32, u32, bool),
    /// [1.2.840.10008.1.2.4.94] JPIP Referenced
    MPEG2MainProfile,
}

impl TransferSyntax {
    pub fn to_id(self) -> i32 {
        match self {
            TransferSyntax::None => 0,
            TransferSyntax::ImplicitVRLittleEndian => 1,
            TransferSyntax::ExplicitVRLittleEndian => 2,
            TransferSyntax::ExplicitVRBigEndian => 3,
            TransferSyntax::JPEGBaselineProcess1(_) => 4,
            TransferSyntax::JPEGExtendedProcess2_4(_) => 5,
            TransferSyntax::JPEGLosslessProcess14 => 6,
            TransferSyntax::JPEGLosslessProcess14_1 => 7,
            TransferSyntax::JPEGLSLossless => 8,
            TransferSyntax::JPEGLSNearLossless(_) => 9,
            TransferSyntax::JPEG2000Lossless => 10,
            TransferSyntax::JPEG2000(_, _, _, _) => 11,
            TransferSyntax::JPEG2000Part2Lossless => 12,
            TransferSyntax::JPEG2000Part2(_, _, _, _) => 13,
            TransferSyntax::RLELossless => 14,
            TransferSyntax::MPEG2MainProfile => 15,
        }
    }
}

#[derive(Copy, Clone)]
pub enum PhotometricInterpretation {
    None,
    Monochrome1,
    Monochrome2,
    PaletteColor,
    RGB,
    HSV,
    ARGB,
    CMYK,
    YbrFull,
    YbrFull422,
    YbrPartial422,
    YbrPartial420,
    YbrIct,
    YbrRct,
}

impl PhotometricInterpretation {
    pub fn to_id(self) -> i32 {
        match self {
            PhotometricInterpretation::None => 0,
            PhotometricInterpretation::Monochrome1 => 1,
            PhotometricInterpretation::Monochrome2 => 2,
            PhotometricInterpretation::PaletteColor => 3,
            PhotometricInterpretation::RGB => 4,
            PhotometricInterpretation::HSV => 5,
            PhotometricInterpretation::ARGB => 6,
            PhotometricInterpretation::CMYK => 7,
            PhotometricInterpretation::YbrFull => 8,
            PhotometricInterpretation::YbrFull422 => 9,
            PhotometricInterpretation::YbrPartial422 => 10,
            PhotometricInterpretation::YbrPartial420 => 11,
            PhotometricInterpretation::YbrIct => 12,
            PhotometricInterpretation::YbrRct => 13,
        }
    }
}

#[repr(C)]
struct output_t {
    status: c_uint,
    size: size_t,
}

extern "C" {
    fn c_convert(
        source_ptr: *const c_uchar,
        source_len: size_t,
        max_size: size_t,
        transfer_syntax_pre: c_int,
        transfer_syntax_post: c_int,
        photometric_interpretation: c_int,
        is_lossy: c_char,     // jpeg + j2k + jpegls
        quality1: c_int,      // jpeg + j2k
        quality2: c_int,      // j2k
        quality3: c_int,      // j2k
        irreversible: c_char, // j2k
        allow_error: c_int,   // jpegls
    ) -> output_t;
}

pub fn pipeline(
    mut source: Vec<u8>,
    estimated_length: Option<usize>,
    transfer_syntax_pre: TransferSyntax,
    photometric_interpretation: PhotometricInterpretation,
    transfer_syntax_post: TransferSyntax,
) -> Result<Vec<u8>, GDCMError> {
    let mut ret;
    let max_size;

    // Set lossy compression parameters
    let (is_lossy, quality1, quality2, quality3, irreversible, allow_error) =
        match transfer_syntax_post {
            TransferSyntax::JPEGBaselineProcess1(t) => {
                // Lossy & Quality1
                if t > 0 {
                    (true, t, 0, 0, false, 0)
                } else {
                    (false, 0, 0, 0, false, 0)
                }
            }
            TransferSyntax::JPEGExtendedProcess2_4(t) => {
                // Lossy & Quality1
                if t > 0 {
                    (true, t, 0, 0, false, 0)
                } else {
                    (false, 0, 0, 0, false, 0)
                }
            }
            TransferSyntax::JPEGLSNearLossless(t) => {
                // Lossy & Allow_error
                if t > 0 {
                    (true, 0, 0, 0, false, t)
                } else {
                    (false, 0, 0, 0, false, t)
                }
            }
            TransferSyntax::JPEG2000(t1, t2, t3, t4) => {
                // Lossy, Quality1, Quality2, Quality3 & Irreversible
                if t1 != 0 || t2 != 0 || t3 != 0 || t4 {
                    (true, t1, t2, t3, t4, 0)
                } else {
                    (false, 0, 0, 0, false, 0)
                }
            }
            TransferSyntax::JPEG2000Part2(t1, t2, t3, t4) => {
                // Lossy, Quality1, Quality2, Quality3 & Irreversible
                if t1 != 0 || t2 != 0 || t3 != 0 || t4 {
                    (true, t1, t2, t3, t4, 0)
                } else {
                    (false, 0, 0, 0, false, 0)
                }
            }
            _ => (false, 0, 0, 0, false, 0),
        };

    // Add more capacity
    if let Some(t) = estimated_length {
        source.reserve(t);
    } else {
        source.reserve(source.len() * 3);
    }

    max_size = source.capacity();

    // Call C function
    ret = unsafe {
        c_convert(
            source.as_ptr(),
            source.len() as size_t,
            max_size as size_t,
            transfer_syntax_pre.to_id(),
            transfer_syntax_post.to_id(),
            photometric_interpretation.to_id(),
            is_lossy as c_char,
            quality1 as i32,
            quality2 as i32,
            quality3 as i32,
            irreversible as c_char,
            allow_error as i32,
        )
    };

    // If need more size, reserve more and re-process
    if ret.status == 0xFF {
        println!(
            "OVERSIZED [{:?}] input: {} estimated: {:?} needed: {}",
            transfer_syntax_pre,
            source.len(),
            estimated_length,
            ret.size,
        );
        source.reserve(ret.size);
        ret = unsafe {
            c_convert(
                source.as_ptr(),
                source.len() as size_t,
                ret.size as size_t,
                transfer_syntax_pre.to_id(),
                transfer_syntax_post.to_id(),
                photometric_interpretation.to_id(),
                is_lossy as c_char,
                quality1 as i32,
                quality2 as i32,
                quality3 as i32,
                irreversible as c_char,
                allow_error as i32,
            )
        };
    }

    // Translate errors
    match ret.status {
        // Success
        0x00 => {
            unsafe {
                source.set_len(ret.size);
            }
            Ok(source)
        }
        // PRE Transfer Syntax conversion error
        0x11 => Err(GDCMError::Pre(Error::ReadStream)),
        0x12 => Err(GDCMError::Pre(Error::FileExplicitFilter)),
        0x13 => Err(GDCMError::Pre(Error::InvalidTransferSyntax)),
        0x14 => Err(GDCMError::Pre(Error::ExecuteChange)),
        0x15 => Err(GDCMError::Pre(Error::DeriveFile)),
        0x16 => Err(GDCMError::Pre(Error::WriteStream)),
        // Photometric conversion error
        0x21 => Err(GDCMError::Photo(Error::ReadStream)),
        0x22 => Err(GDCMError::Photo(Error::InvalidPhotometricInterpretation)),
        0x23 => Err(GDCMError::Photo(Error::ExecuteChange)),
        0x24 => Err(GDCMError::Photo(Error::ExecuteLUTChange)),
        0x25 => Err(GDCMError::Photo(Error::WriteStream)),
        // POST Transfer Syntax conversion error
        0x31 => Err(GDCMError::Post(Error::ReadStream)),
        0x32 => Err(GDCMError::Post(Error::FileExplicitFilter)),
        0x33 => Err(GDCMError::Post(Error::InvalidTransferSyntax)),
        0x34 => Err(GDCMError::Post(Error::ExecuteChange)),
        0x35 => Err(GDCMError::Post(Error::DeriveFile)),
        0x36 => Err(GDCMError::Post(Error::WriteStream)),
        // Other errors
        0x0F => Err(GDCMError::PointerNULL),
        0x1F => Err(GDCMError::EmptyBuffer),
        _ => Err(GDCMError::Unknown),
    }
}
