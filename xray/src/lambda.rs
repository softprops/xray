use crate::Header;
use std::{
    env::var,
    fs::{create_dir_all, File},
};

pub(crate) fn init() -> std::io::Result<()> {
    if taskRoot().is_some() {
        create_dir_all("/tmp/.aws-xray")?;
        File::create("/tmp/.aws-xray/initialized")?;
    }
    Ok(())
}

pub(crate) fn taskRoot() -> Option<String> {
    var("LAMBDA_TASK_ROOT").ok()
}

pub(crate) fn header() -> Option<Header> {
    var("_X_AMZN_TRACE_ID")
        .ok()
        .and_then(|value| value.parse::<Header>().ok())
}
