use crate::agents::tools::geocoding::GeoCoding;
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

// 1. Definimos la estructura de argumentos que el Orquestador enviará a este sub-agente.
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct AddressChangeArgs {
    pub customer_id: String,
    pub new_address: String,
    pub reason: String,
}

// 2. Definimos la estructura del Tool/Agente
#[derive(Clone)]
pub struct AddressSpecialist<M: CompletionModel + Clone + Send + Sync + 'static> {
    // Usamos Arc para permitir Clone y moverlo a tareas.
    agent: Arc<Agent<M>>,
}

impl<M: CompletionModel + Clone + Send + Sync + 'static> AddressSpecialist<M> {
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(GeoCoding)
            .build();

        Self {
            agent: Arc::new(agent),
        }
    }
}

// 3. Implementamos el trait Tool para que el Orquestador pueda usarlo.
impl<M: CompletionModel + Clone + Send + Sync + 'static> Tool for AddressSpecialist<M> {
    const NAME: &'static str = "address_specialist";

    type Args = AddressChangeArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Usa este agente cuando el usuario quiera cambiar su dirección de entrega o modificar datos de envío.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(AddressChangeArgs))
                .expect("Failed to serialize schema"),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let prompt = format!(
            "Procesa un cambio de dirección para el cliente {}. Nueva dirección: {}. Motivo: {}",
            args.customer_id, args.new_address, args.reason
        );

        let agent = self.agent.clone();

        // Spawn a task to ensure the future is Send + Sync compatible if needed,
        // and to handle the async call cleanly.
        let response = tokio::spawn(async move { agent.prompt(&prompt).await })
            .await
            .map_err(|e| ToolError(format!("Task join error: {}", e)))?
            .map_err(|e| ToolError(format!("Agent execution error: {}", e)))?;

        Ok(response)
    }
}
