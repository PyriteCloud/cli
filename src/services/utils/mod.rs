use std::{error::Error, future::Future};

use cliclack::spinner;

#[derive(Debug, Clone)]
pub(crate) struct UtilsService;

impl UtilsService {
    pub async fn with_progress<T, R>(
        fun: impl FnOnce() -> T,
        msg: &str,
        success: &str,
        failed: &str,
    ) -> Result<R, Box<dyn std::error::Error>>
    where
        T: Future<Output = Result<R, Box<dyn Error>>>,
    {
        let progress = spinner();
        progress.start(msg);

        let result = fun().await;

        match result {
            Ok(x) => {
                progress.stop(success);
                Ok(x)
            }
            Err(err) => {
                progress.error(failed);
                Err(err)
            }
        }
    }
}
