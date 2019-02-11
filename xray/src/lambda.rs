use std::{
    env::var,
    fs::{create_dir_all, File}
};

pub(crate) fn init() -> std::io::Result<()> {
    if var("LAMBDA_TASK_ROOT").is_ok() {
        create_dir_all("/tmp/.aws-xray")?;
        File::create("/tmp/.aws-xray/initialized")?;
    }
    Ok(())
}