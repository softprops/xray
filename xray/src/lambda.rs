use crate::Header;
use std::{
    env::var,
    fs::{create_dir_all, File},
    io::Result as IoResult,
};

pub(crate) fn init() -> IoResult<()> {
    if task_root_present() {
        create_dir_all("/tmp/.aws-xray")?;
        File::create("/tmp/.aws-xray/initialized")?;
    }
    Ok(())
}

pub(crate) fn task_root_present() -> bool {
    var("LAMBDA_TASK_ROOT").is_ok()
}

pub(crate) fn header() -> Option<Header> {
    var("_X_AMZN_TRACE_ID")
        .ok()
        .and_then(|value| value.parse::<Header>().ok())
}
