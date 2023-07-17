//! Models of what gets written to `/home/dicom/log`.
#![allow(non_snake_case)]

use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
struct OnePatient<'a> {
    PatientID: &'a str,
    PatientName: &'a str,
    PatientAge: &'a str,
    PatientSex: &'a str,
    PatientBirthDate: &'a str,
    StudyList: Vec<&'a str>
}
