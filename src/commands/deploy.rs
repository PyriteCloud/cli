use crate::{services::DeployService, PyriteJson};

#[derive(Debug, Clone)]
pub(crate) struct DeployCommands;

impl DeployCommands {
    pub async fn run(file: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = file.unwrap();
        let file_data = std::fs::read_to_string(file_path)?;
        let pyrite_json: PyriteJson = serde_json::from_str(&file_data)?;

        let service = DeployService::deploy(pyrite_json).await?;

        println!("Service deployed: {}", service.name);

        Ok(())
    }
}
