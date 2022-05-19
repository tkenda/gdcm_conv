#include "gdcmFileDerivation.h"
#include "gdcmImageReader.h"
#include "gdcmImage.h"
#include "gdcmWriter.h"
#include "gdcmAttribute.h"
#include "gdcmImageWriter.h"
#include "gdcmImageChangeTransferSyntax.h"
#include "gdcmImageChangePhotometricInterpretation.h"
#include "gdcmImageApplyLookupTable.h"
#include "gdcmFileExplicitFilter.h"
#include "gdcmFileMetaInformation.h"
#include "gdcmJPEG2000Codec.h"
#include "gdcmJPEGLSCodec.h"
#include "gdcmJPEGCodec.h"

#include <iostream>
#include <istream>
#include <fstream>
#include <streambuf>
#include <string>

#include "wrapper.h"

using namespace std;

namespace gdcm
{
    static bool derives(File &file, const Pixmap &compressed_image)
    {
        DataSet &ds = file.GetDataSet();

        if (!ds.FindDataElement(Tag(0x0008, 0x0016)) || ds.GetDataElement(Tag(0x0008, 0x0016)).IsEmpty())
        {
            return false;
        }

        if (!ds.FindDataElement(Tag(0x0008, 0x0018)) || ds.GetDataElement(Tag(0x0008, 0x0018)).IsEmpty())
        {
            return false;
        }

        const DataElement &sopclassuid = ds.GetDataElement(Tag(0x0008, 0x0016));
        const DataElement &sopinstanceuid = ds.GetDataElement(Tag(0x0008, 0x0018));

        // Make sure that const char* pointer will be properly padded with \0 char:
        std::string sopclassuid_str(sopclassuid.GetByteValue()->GetPointer(), sopclassuid.GetByteValue()->GetLength());
        std::string sopinstanceuid_str(sopinstanceuid.GetByteValue()->GetPointer(), sopinstanceuid.GetByteValue()->GetLength());
        ds.Remove(Tag(0x0008, 0x0018));

        gdcm::FileDerivation fd;
        fd.SetFile(file);
        fd.AddReference(sopclassuid_str.c_str(), sopinstanceuid_str.c_str());

        // CID 7202 Source Image Purposes of Reference
        // {"DCM",121320,"Uncompressed predecessor"},
        fd.SetPurposeOfReferenceCodeSequenceCodeValue(121320);

        // CID 7203 Image Derivation
        // { "DCM",113040,"Lossy Compression" },
        fd.SetDerivationCodeSequenceCodeValue(113040);
        fd.SetDerivationDescription("lossy conversion");
        if (!fd.Derive())
        {
            std::cerr << "Sorry could not derive using input info" << std::endl;
            return false;
        }

        /*
        (0028,2110) CS [01]                                     #   2, 1 LossyImageCompression
        (0028,2112) DS [15.95]                                  #   6, 1 LossyImageCompressionRatio
        (0028,2114) CS [ISO_10918_1]                            #  12, 1 LossyImageCompressionMethod
        */
        const DataElement &pixeldata = compressed_image.GetDataElement();

        size_t len = pixeldata.GetSequenceOfFragments()->ComputeByteLength();
        size_t reflen = compressed_image.GetBufferLength();
        double ratio = (double)reflen / (double)len;

        Attribute<0x0028, 0x2110> at1;
        at1.SetValue("01");
        ds.Replace(at1.GetAsDataElement());

        Attribute<0x0028, 0x2112> at2;
        at2.SetValues(&ratio, 1);
        ds.Replace(at2.GetAsDataElement());

        Attribute<0x0028, 0x2114> at3;

        return true;
    }
} // namespace gdcm

struct ProcResp
{
    unsigned int status;
    std::string image;
};

// Modify Photometric Interpretation from incoming stream.
ProcResp change_photometric(
    int photometric_interpretation,
    std::string &src)
{
    struct ProcResp proc_resp;

    std::istringstream dicomInput(src);
    std::ostringstream dicomOutput;

    gdcm::PixmapReader reader;
    reader.SetStream(dicomInput);

    if (!reader.Read())
    {
        proc_resp.status = 0x01;
        return proc_resp;
    }

    gdcm::Pixmap &image = reader.GetPixmap();

    gdcm::PixmapWriter writer;
    writer.SetStream(dicomOutput);
    writer.SetFile(reader.GetFile());

    if(image.GetPhotometricInterpretation() != gdcm::PhotometricInterpretation::PALETTE_COLOR) {
        gdcm::ImageChangePhotometricInterpretation change;
        change.SetInput(image);

        switch (photometric_interpretation)
        {
        case 1:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::MONOCHROME1);
            break;
        case 2:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::MONOCHROME2);
            break;
        case 3:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::PALETTE_COLOR);
            break;
        case 4:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::RGB);
            break;
        case 5:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::HSV);
            break;
        case 6:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::ARGB);
            break;
        case 7:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::CMYK);
            break;
        case 8:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::YBR_FULL);
            break;
        case 9:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::YBR_FULL_422);
            break;
        case 10:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::YBR_PARTIAL_422);
            break;
        case 11:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::YBR_PARTIAL_420);
            break;
        case 12:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::YBR_ICT);
            break;
        case 13:
            change.SetPhotometricInterpretation(gdcm::PhotometricInterpretation::YBR_RCT);
            break;
        default:
            proc_resp.status = 0x02;
            return proc_resp;
        }

        if (!change.Change())
        {
            proc_resp.status = 0x03;
            return proc_resp;
        }

        writer.SetPixmap(change.PixmapToPixmapFilter::GetOutput());
    } else {
        gdcm::ImageApplyLookupTable change;
        change.SetInput(image);

        if (!change.Apply())
        {
            proc_resp.status = 0x04;
            return proc_resp;
        }

        writer.SetPixmap(change.PixmapToPixmapFilter::GetOutput());
    }

    if (!writer.Write())
    {
        proc_resp.status = 0x05;
        return proc_resp;
    }

    proc_resp.status = 0x00;
    proc_resp.image = dicomOutput.str();
    return proc_resp;
}

ProcResp change_transfer(
    int transfer_syntax,
    char is_lossy,
    int quality1,
    int quality2,
    int quality3,
    char irreversible,
    int allow_error,
    std::string &src)
{
    struct ProcResp proc_resp;

    std::istringstream dicomInput(src);
    std::ostringstream dicomOutput;

    gdcm::FileMetaInformation::SetImplementationVersionName("Idria Software");
    gdcm::FileMetaInformation::SetSourceApplicationEntityTitle("PROTEUS");

    bool is_jpeg = false;
    bool is_jpegls = false;
    bool is_j2k = false;
    bool derive = false;

    gdcm::PixmapReader reader;
    reader.SetStream(dicomInput);
    if (!reader.Read())
    {
        proc_resp.status = 0x01;
        return proc_resp;
    }
    gdcm::Pixmap &image = reader.GetPixmap();

    // Make sure the DICOM attributes follows PS 3.6 rules,
    // when converting to an explicit little transfer syntax.
    if (transfer_syntax != 1)
    {
        gdcm::FileExplicitFilter toExplicit;
        toExplicit.SetChangePrivateTags(false);
        toExplicit.SetFile(reader.GetFile());
        if (!toExplicit.Change())
        {
            proc_resp.status = 0x02;
            return proc_resp;
        }
    }

    gdcm::JPEGCodec jpegcodec;
    gdcm::JPEGLSCodec jpeglscodec;
    gdcm::JPEG2000Codec j2kcodec;
    gdcm::ImageChangeTransferSyntax change;

    switch (transfer_syntax)
    {
    case 1:
        change.SetTransferSyntax(gdcm::TransferSyntax::ImplicitVRLittleEndian);
        break;
    case 2:
        change.SetTransferSyntax(gdcm::TransferSyntax::ExplicitVRLittleEndian);
        break;
    case 3:
        change.SetTransferSyntax(gdcm::TransferSyntax::ExplicitVRBigEndian);
        break;
    case 4:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEGBaselineProcess1);
        is_jpeg = true;
        break;
    case 5:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEGExtendedProcess2_4);
        is_jpeg = true;
        break;
    case 6:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEGLosslessProcess14);
        is_jpeg = true;
        break;
    case 7:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEGLosslessProcess14_1);
        is_jpeg = true;
        break;
    case 8:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEGLSLossless);
        is_jpegls = true;
        break;
    case 9:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEGLSNearLossless);
        is_jpegls = true;
        break;
    case 10:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEG2000Lossless);
        is_j2k = true;
        break;
    case 11:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEG2000);
        is_j2k = true;
        break;
    case 12:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEG2000Part2Lossless);
        is_j2k = true;
        break;
    case 13:
        change.SetTransferSyntax(gdcm::TransferSyntax::JPEG2000Part2);
        is_j2k = true;
        break;
    case 14:
        change.SetTransferSyntax(gdcm::TransferSyntax::RLELossless);
        break;
    case 15:
        change.SetTransferSyntax(gdcm::TransferSyntax::MPEG2MainProfile);
        break;
    default:
        proc_resp.status = 0x03;
        return proc_resp;
    }

    // jpeg lossy
    if (is_lossy && is_jpeg)
    {
        jpegcodec.SetLossless(false);
        if (quality1)
            jpegcodec.SetQuality(static_cast<double>(quality1));
        change.SetUserCodec(&jpegcodec);
        derive = true;
    }

    // jpegls lossy
    if (is_lossy && is_jpegls)
    {
        jpeglscodec.SetLossless(false);
        if (allow_error)
            jpeglscodec.SetLossyError(allow_error);
        change.SetUserCodec(&jpeglscodec);
        derive = true;
    }

    // jk2 lossy
    if (is_lossy && is_j2k)
    {
        j2kcodec.SetLossyFlag(true);
        if (quality1)
            j2kcodec.SetQuality(0, static_cast<double>(quality1));
        if (quality2)
            j2kcodec.SetQuality(1, static_cast<double>(quality2));
        if (quality3)
            j2kcodec.SetQuality(2, static_cast<double>(quality3));
        if (irreversible)
        {
            j2kcodec.SetReversible(false);
        }
        else
        {
            j2kcodec.SetReversible(true);
        }
        change.SetUserCodec(&j2kcodec);
        derive = true;
    }

    change.SetInput(image);
    if (!change.Change())
    {
        proc_resp.status = 0x04;
        return proc_resp;
    }

    // Derive image only for lossy
    if (derive)
    {
        if (!gdcm::derives(reader.GetFile(), change.PixmapToPixmapFilter::GetOutput()))
        {
            proc_resp.status = 0x05;
            return proc_resp;
        }
    }

    gdcm::PixmapWriter writer;
    writer.SetStream(dicomOutput);
    writer.SetFile(reader.GetFile());

    gdcm::File &file = writer.GetFile();
    gdcm::FileMetaInformation &fmi = file.GetHeader();
    fmi.Remove(gdcm::Tag(0x0002, 0x0100)); //  '   '    ' // PrivateInformationCreatorUID
    fmi.Remove(gdcm::Tag(0x0002, 0x0102)); //  '   '    ' // PrivateInformation

    const gdcm::Pixmap &pixout = change.PixmapToPixmapFilter::GetOutput();
    writer.SetPixmap(pixout);
    if (!writer.Write())
    {
        proc_resp.status = 0x06;
        return proc_resp;
    }

    proc_resp.status = 0x00;
    proc_resp.image = dicomOutput.str();
    return proc_resp;
}

struct OutputStruct c_convert(
    char *i_buffer_ptr,
    size_t i_buffer_len,
    size_t max_size,
    int transfer_syntax_pre,
    int transfer_syntax_post,
    int photometric_interpretation,
    char is_lossy,
    int quality1,
    int quality2,
    int quality3,
    char irreversible,
    int allow_error)
{
    struct OutputStruct resp;
    struct ProcResp proc_resp;

    // Use memory map as input & output
    if (i_buffer_ptr == NULL) {
        resp.status = 0x0F;
        return resp;
    }
    if (i_buffer_len == 0) {
        resp.status = 0x1F;
        return resp;
    }
    
    std::string inputString(i_buffer_ptr, i_buffer_len);

    // Copy for process pipeline
    proc_resp.image = inputString;

    // Change transfer syntax pre
    if (transfer_syntax_pre > 0)
    {
        proc_resp = change_transfer(
            transfer_syntax_pre,
            is_lossy,
            quality1,
            quality2,
            quality3,
            irreversible,
            allow_error,
            proc_resp.image);
        if (proc_resp.status > 0)
        {
            resp.status = proc_resp.status + 0x10;
            return resp;
        }
    }

    // Change photometric interpretation
    if (photometric_interpretation > 0)
    {
        proc_resp = change_photometric(
            photometric_interpretation,
            proc_resp.image);
        if (proc_resp.status > 0)
        {
            resp.status = proc_resp.status + 0x20;
            return resp;
        }
    }

    // Change transfer syntax post
    if (transfer_syntax_post > 0)
    {
        proc_resp = change_transfer(
            transfer_syntax_post,
            is_lossy,
            quality1,
            quality2,
            quality3,
            irreversible,
            allow_error,
            proc_resp.image);
        if (proc_resp.status > 0)
        {
            resp.status = proc_resp.status + 0x30;
            return resp;
        }
    }

    if (max_size >= proc_resp.image.size())
    {
        memcpy(i_buffer_ptr, proc_resp.image.c_str(), proc_resp.image.size());
        resp.status = 0x00;
    }
    else
    {
        resp.status = 0xFF;
    }

    resp.size = proc_resp.image.size();
    return resp;
}
