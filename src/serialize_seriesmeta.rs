//! Helpers for serializing the stuff which goes into e.g.
//! `/home/dicom/log/studyData/1.2.840.113845.11.1000000001785349915.20130308061609.6346698-series/1.3.12.2.1107.5.2.19.45152.2013030808061520200285270.0.0.0-meta.json`
#![allow(non_snake_case)]
use crate::dicom_data::name_of;
use crate::errors::ElementSerializationError;
use dicom::core::header::Header;
use dicom::core::value::{DataSetSequence, Value};
use dicom::core::{PrimitiveValue, VR};
use dicom::dictionary_std::tags;
use dicom::object::mem::InMemElement;
use dicom::object::{DefaultDicomObject, InMemDicomObject};
use hashbrown::HashMap;
use itertools::Itertools;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize)]
pub(crate) struct StudyDataSeriesMeta<'a> {
    SeriesInstanceUID: &'a str,
    SeriesBaseDir: &'a str,
    DICOM: HashMap<String, ValueAndLabel<'a>>,
}

impl<'a> StudyDataSeriesMeta<'a> {
    pub fn new(
        SeriesInstanceUID: &'a str,
        SeriesBaseDir: &'a str,
        dcm: &'a DefaultDicomObject,
    ) -> Self {
        let DICOM = dcm
            .iter()
            .filter(|ele| ele.tag() != tags::PIXEL_DATA)
            .map(ValueAndLabel::try_from)
            .filter_map(|r| r.ok())
            .map(|v| (v.label.to_string(), v))
            .collect::<HashMap<String, ValueAndLabel>>();
        Self {
            SeriesInstanceUID,
            SeriesBaseDir,
            DICOM,
        }
    }
}

#[derive(Debug, Serialize)]
struct ValueAndLabel<'a> {
    value: Cow<'a, str>,
    label: &'a str,
}

impl<'a> TryFrom<&'a InMemElement> for ValueAndLabel<'a> {
    type Error = ElementSerializationError;
    fn try_from(ele: &'a InMemElement) -> Result<Self, Self::Error> {
        let tag = ele.tag();
        let label = name_of(tag).ok_or_else(|| ElementSerializationError::UnknownTagError(tag))?;
        let value = match ele.value() {
            Value::Primitive(value) => Ok(serialize_primitive(value, ele.vr())),
            Value::Sequence(seq) => {
                // e.g. tags with complex data such as ReferencedImageSequence and RequestAttributesSequence
                let s = serialize_sequence(seq);
                Ok(Cow::Owned(s))
            }
            Value::PixelSequence(_) => Err(ElementSerializationError::Excluded(tag)),
        }?;
        // println!("{} {} {:?} {}", label, ele.vr(), ele.value(), value);
        Ok(Self { label, value })
    }
}

/// Serialize a [PrimitiveValue].
///
/// You would think that [PrimitiveValue::to_str] is sufficient, and sometimes it is,
/// but lists of numbers such as `ImagePositionPatient` are represented as a list of
/// strings instead of floats. Thus we need to do some custom logic of our own.
///
/// See discussion on Github: https://github.com/Enet4/dicom-rs/discussions/401
fn serialize_primitive(value: &PrimitiveValue, vr: VR) -> Cow<str> {
    match value {
        PrimitiveValue::Strs(strs) => {
            if matches!(vr, VR::IS | VR::SS | VR::DS) {
                serialize_nums(strs, value)
            } else if strs.len() > 1 {
                serde_json::to_string(strs.as_slice())
                    .map(Cow::Owned)
                    .unwrap_or_else(|_| value.to_str())
            } else {
                value.to_str()
            }
        }
        PrimitiveValue::U8(nums) => serialize_nums(nums, value),
        PrimitiveValue::I16(nums) => serialize_nums(nums, value),
        PrimitiveValue::U16(nums) => serialize_nums(nums, value),
        PrimitiveValue::I32(nums) => serialize_nums(nums, value),
        PrimitiveValue::U32(nums) => serialize_nums(nums, value),
        PrimitiveValue::I64(nums) => serialize_nums(nums, value),
        PrimitiveValue::U64(nums) => serialize_nums(nums, value),
        PrimitiveValue::F32(nums) => serialize_nums(nums, value),
        PrimitiveValue::F64(nums) => serialize_nums(nums, value),
        _ => value.to_str(),
    }
}

/// Serialize a list of numbers to a JSON string.
fn serialize_nums<'a, D: std::fmt::Display>(
    strs: &'a dicom::core::value::C<D>,
    value: &'a PrimitiveValue,
) -> Cow<'a, str> {
    if strs.len() == 1 {
        return value.to_str();
    }
    let s = format!("[{}]", strs.iter().join(", "));
    Cow::Owned(s)
}

/// Serializer for [Value::Sequence].
fn serialize_sequence(seq: &DataSetSequence<InMemDicomObject>) -> String {
    format!(
        "[{}]",
        seq.items().iter().flat_map(serialize_subdicom).join("\n")
    )
}

/// Serializes a complicated element, e.g. `RequestAttributesSequence` and `ReferencedImageSequence`
///
/// Example for `RequestAttributesSequence`
///
/// `"[(0040, 0007) Scheduled Procedure Step Descriptio LO: 'MR Brain'\n(0040, 0009) Scheduled Procedure Step ID         SH: '4101374'\n(0040, 1001) Requested Procedure ID              SH: '4101374']"`
fn serialize_subdicom(dcm: &InMemDicomObject) -> impl Iterator<Item = String> + '_ {
    dcm.iter().map(serialize_subdicom_element)
}

fn serialize_subdicom_element(ele: &InMemElement) -> String {
    let tag = ele.tag();

    // This doesn't perfectly match what px-repack would spit out, because px-repack would give a fully
    // human-readable name from the "Attribute Name" column of this table:
    // https://dicom.nema.org/medical/dicom/2020b/output/chtml/part03/sect_C.4.10.html
    // e.g. "Scheduled Procedure Step Descriptio"
    let label = name_of(tag).unwrap_or("RX_REPACK_ERROR");

    let vr = ele.vr();
    let value = ele
        .to_str()
        .map(|c| c.to_string())
        .unwrap_or("RX_REPACK_ERROR".to_string());
    format!("{tag} {label} {vr}: {value}")
}
