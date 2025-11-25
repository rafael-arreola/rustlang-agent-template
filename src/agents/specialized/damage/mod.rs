use crate::agents::tools::cost_database::CostDatabase;
use rig::{
    agent::{Agent, AgentBuilder},
    completion::{CompletionModel, Prompt},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
#[error("Tool execution error: {0}")]
pub struct ToolError(String);

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct DamageReportArgs {
    pub item_name: String,
    pub description_of_damage: String,
}

#[derive(Clone)]
pub struct DamageSpecialist<M: CompletionModel + Clone + Send + Sync + 'static> {
    agent: Arc<Agent<M>>,
}

impl<M: CompletionModel + Clone + Send + Sync + 'static> DamageSpecialist<M> {
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(CostDatabase)
            .build();

        Self { agent: Arc::new(agent) }
    }
}

impl<M: CompletionModel + Clone + Send + Sync + 'static> Tool for DamageSpecialist<M> {
    const NAME: &'static str = "damage_specialist";

    type Args = DamageReportArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Usa este agente cuando el usuario reporte un artículo dañado, roto o defectuoso.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(DamageReportArgs))
                .expect("Failed to serialize schema"),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let prompt = format!(
            "Reporte de daño para el articulo '{}'. Descripción: {}",
            args.item_name, args.description_of_damage
        );

        let agent = self.agent.clone();

        let response = tokio::spawn(async move {
            agent.prompt(&prompt).await
        })
        .await
        .map_err(|e| ToolError(format!("Task join error: {}", e)))?
        .map_err(|e| ToolError(format!("Agent execution error: {}", e)))?;
        
        Ok(response)
    }
}
