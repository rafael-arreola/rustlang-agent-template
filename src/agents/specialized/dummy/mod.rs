use crate::agents::tools::text_reverser::TextReverser;
use rig::{
    agent::{Agent, AgentBuilder},
    completion::CompletionModel,
    tool::Tool,
};
use serde::{Deserialize, Serialize};

// ================================================================
// 1. Definición de Argumentos (Input/Output)
// ================================================================

/// Argumentos que el Orquestador enviará a este especialista.
/// Describe claramente los campos para que el LLM sepa qué enviar.
#[derive(Deserialize)]
pub struct DummyArgs {
    pub message: String,
}

/// La respuesta que devolverá este especialista.
#[derive(Serialize)]
pub struct DummyOutput {
    pub reply: String,
}

// ================================================================
// 2. Estructura del Especialista
// ================================================================

#[derive(Debug, thiserror::Error)]
#[error("Error en el Dummy Specialist")]
pub struct DummyError;

/// El struct principal que envuelve al Agente.
pub struct DummySpecialist<M: CompletionModel + Clone + Send + Sync + 'static> {
    agent: Agent<M>,
}

impl<M: CompletionModel + Clone + Send + Sync + 'static> DummySpecialist<M> {
    /// Crea una nueva instancia del especialista.
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(TextReverser)
            .build();

        Self { agent }
    }
}

// ================================================================
// 3. Implementación del Trait Tool (Para Rig)
// ================================================================

impl<M: CompletionModel + Clone + Send + Sync + 'static> Tool for DummySpecialist<M> {
    const NAME: &'static str = "dummy_specialist";

    type Error = DummyError;
    type Args = DummyArgs;
    type Output = DummyOutput;

    /// Define la "Firma" de la herramienta para que el Orquestador sepa cuándo usarla.
    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "dummy_specialist".to_string(),
            description:
                "Un agente de prueba. Úsalo cuando el usuario pida probar el sistema o diga 'ping'."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "El mensaje que se quiere probar o hacer eco."
                    }
                },
                "required": ["message"]
            }),
        }
    }

    /// La lógica de ejecución cuando el Orquestador llama a esta herramienta.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Aquí delegamos la tarea al sub-agente (LLM)
        let response = self
            .agent
            .prompt(&args.message)
            .await
            .map_err(|_| DummyError)?;

        Ok(DummyOutput { reply: response })
    }
}
