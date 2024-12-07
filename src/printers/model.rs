use actix_multipart::form::{json::Json as MPJson, tempfile::TempFile, MultipartForm};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Printer {
    pub name: String,
    pub uri: String,
    pub state: String,
    pub capabilites: JobOptionsCapabilities,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JobOptionsCapabilities {
    pub two_sided: bool,
}

#[derive(Debug, Deserialize)]
pub struct JobOptions {
    pub two_sided: bool,
}

#[derive(Debug, Deserialize)]
pub struct PrintJobInfo {
    pub printer: String,
    pub options: Option<JobOptions>,
}

#[derive(Debug, MultipartForm)]
pub struct PrinterJobForm {
    #[multipart(limit = "100MB")]
    pub file: TempFile,
    pub job_info: MPJson<PrintJobInfo>,
}
