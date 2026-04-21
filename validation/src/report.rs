//! Report writer stubs — replaced in Wave-2-6 with the real HTML matrix +
//! JSONL record writers per RESEARCH §"report.html schema" / §"report.jsonl schema".

use anyhow::Result;

use crate::driver::Report;

pub fn write_html(_report: &Report, _path: &str) -> Result<()> {
    Ok(())
}

pub fn write_jsonl(_report: &Report, _path: &str) -> Result<()> {
    Ok(())
}
