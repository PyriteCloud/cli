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

    pub fn get_service_status_label(service_status: i32) -> String {
        match service_status {
            // Generic
            3001 => "Ready".to_owned(),

            // Sync
            2011 => "Syncing Network".to_owned(),
            4011 => "Failed To Sync Network".to_owned(),

            // Delete
            2021 => "Deleting".to_owned(),
            4021 => "Failed To Delete".to_owned(),

            _ => "Unknown".to_owned(),
        }
    }

    pub fn get_service_status_color(service_status: i32) -> comfy_table::Color {
        match service_status {
            // Generic
            3001 => comfy_table::Color::Green,

            // Sync
            2011 => comfy_table::Color::Yellow,
            4011 => comfy_table::Color::Red,

            // Delete
            2021 => comfy_table::Color::Yellow,
            4021 => comfy_table::Color::Red,

            _ => comfy_table::Color::Yellow,
        }
    }

    pub fn get_deployment_status_label(deployment_status: i32) -> String {
        match deployment_status {
            // Generic
            2001 => "Pending".to_owned(),
            4001 => "Failed".to_owned(),

            // Build
            2011 => "Building".to_owned(),
            3011 => "Build Succeeded".to_owned(),
            4011 => "Build Failed".to_owned(),

            // Deploy
            2021 => "Deploying".to_owned(),
            3021 => "Deployed".to_owned(),
            4021 => "Failed To Deploy".to_owned(),

            // Pause
            2031 => "Pausing".to_owned(),
            3031 => "Paused".to_owned(),
            4031 => "Failed To Pause".to_owned(),

            // Stop
            2041 => "Stopping".to_owned(),
            3041 => "Stopped".to_owned(),
            4041 => "Failed To Stop".to_owned(),

            // Cancel
            2051 => "Canceling".to_owned(),
            3051 => "Canceled".to_owned(),
            4051 => "Failed To Cancel".to_owned(),

            _ => "Unknown".to_owned(),
        }
    }

    pub fn get_deployment_status_color(deployment_status: i32) -> comfy_table::Color {
        match deployment_status {
            // Generic
            2001 => comfy_table::Color::Yellow,
            4001 => comfy_table::Color::Red,

            // Build
            2011 => comfy_table::Color::Blue,
            3011 => comfy_table::Color::Green,
            4011 => comfy_table::Color::Red,

            // Deploy
            2021 => comfy_table::Color::Blue,
            3021 => comfy_table::Color::Green,
            4021 => comfy_table::Color::Red,

            // Pause
            2031 => comfy_table::Color::Yellow,
            3031 => comfy_table::Color::Yellow,
            4031 => comfy_table::Color::Red,

            // Stop
            2041 => comfy_table::Color::Yellow,
            3041 => comfy_table::Color::Grey,
            4041 => comfy_table::Color::Red,

            // Cancel
            2051 => comfy_table::Color::Yellow,
            3051 => comfy_table::Color::Grey,
            4051 => comfy_table::Color::Red,

            _ => comfy_table::Color::Yellow,
        }
    }
}
