An easy-to-use Grassroots DICOM Library wrapper designed to convert DICOM files transfer syntaxes and photometric interpretation.

# Usage

You need CMake to build [GDCM Library](http://gdcm.sourceforge.net/wiki).

### Linux Ubuntu:

```cmd
sudo apt-get install cmake
```

### Windows & MacOS:

Download CMake directly from [www.cmake.org/download](https://cmake.org/download/) page.

## Quickstart

Copy this code and make sure you have a DICOM file to test ([DICOM file samples](https://support.dcmtk.org/redmine/projects/dcmtk/wiki/DICOM_images)).

```rust
use std::io::prelude::*;
use std::fs::File;
use gdcm_conv::{TransferSyntax, PhotometricInterpretation};

// Read input file
let mut ibuffer = Vec::new();
let mut ifile = File::open("test.dcm").unwrap();    
ifile.read_to_end(&mut ibuffer).unwrap();

// Transcode DICOM file
let obuffer = match gdcm_conv::pipeline(
    // Input DICOM file buffer
    ibuffer,
    // Estimated Length
    None,
    // First Transfer Syntax conversion
    TransferSyntax::JPEG2000Lossless,
    // Photometric conversion
    PhotometricInterpretation::None,
    // Second Transfer Syntax conversion
    TransferSyntax::None,
) {
    Ok(t) => t,
    Err(e) => {
        eprintln!("{}", e);
        return;
    }
};

// Create output file and save
let mut ofile = File::create("output.dcm").unwrap();
ofile.write_all(&obuffer).unwrap();
```

## How it works

The gdcm_conv library takes as input the content of the DICOM file. It reuse the source vector allocating an estimated
size to avoid cloned memory. The default estimad length is 3 times the input file size, the worst case, changing from
a compressed image (like JPEG2000) to raw. Is recommended to use an estimated calculation, to minimize memory allocation.

If the allocated size is not enough, the library will re-allocate to the correct size and execute the FFI function again.

To estimate the output length you could use this aproximation:

- (0028,0100) bits_allocated
- (0028,0004) photometric_interpretation
- (0028,0008) number_of_frames
- (0028,0010) rows
- (0028,0011) columns

```
// MAX HEADER SIZE
const MAX_HEADER_SIZE: usize = 5000;

let a = match bits_allocated {
    8 => 1,
    16 => 2,
};

let b = match photometric_interpretation {
    "MONOCHROME1" => 1,
    "MONOCHROME2" => 1,
    _ => 3,
};

let estimad_length = (a * b * rows * columns * number_of_frames) + MAX_HEADER_SIZE;
```

The library works as a pipeline with a first transfer syntax conversion (PRE-TRANSFER), a photometric conversion 
and a final transfer syntax conversion (POST-TRANSFER). If you set to None it don't execute the step. 
Usually, you will use only the first and/or second step.

I setup this way because in some cases is needed two transfer syntax transcoding like this example:

The conversion from JPEG Baseline (Process 1) 1.2.840.10008.1.2.4.50 with YBR_FULL or YBR_FULL_422 to JPEG2000
lossles, you need to change to Explicit Little Endian transfer syntax, then to an RGB photometric interpretation and
finally to JPG2000, to avoid GDCM color interpretation issue.
