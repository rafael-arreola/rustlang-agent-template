# 🤖 Rust AI Agent Template

Este proyecto es una plantilla robusta y escalable para construir sistemas multi-agente utilizando **Rust** y la librería **Rig**. Implementa una arquitectura de **Orquestador-Especialistas**.

---

## 🚀 Inicio Rápido

### 1. Prerrequisitos

- Rust (latest stable)
- Claves de API para los modelos que desees usar (OpenAI, Anthropic, Google Gemini).

### 2. Configuración

1. Clona el repositorio.
2. Copia el archivo de variables de entorno:
   ```bash
   cp .env.example .env
   ```
3. Edita `.env` y añade tus claves:
   ```env
   OPENAI_API_KEY=sk-...
   GEMINI_API_KEY=...
   ANTHROPIC_API_KEY=...
   ```

### 3. Ejecutar

```bash
cargo run
```

El servidor iniciará (por defecto en puerto 8080).

---

## 🧠 Arquitectura

El sistema funciona como una "colmena" jerárquica:

1.  **Orquestador (`agents/orchestrator`)**: Es el punto de entrada. Recibe el prompt del usuario, "piensa" y decide si puede responder directamente, usar una herramienta básica o invocar a un especialista.
2.  **Especialistas (`agents/specialized`)**: Son sub-agentes expertos en una tarea específica (ej. obtener dirección, evaluar daños). Se comportan como `Tools` complejas para el orquestador. A su vez, **ellos tienen sus propias herramientas**.
3.  **Herramientas (`agents/tools/`)**: Son "Las Manos" del sistema. Funciones deterministas (código puro) que ejecutan acciones concretas (consultar DB, invertir texto, calcular coordenadas).
4.  **Infraestructura (`infra/`)**: Maneja la conexión con los proveedores de LLM (OpenAI, Google, Anthropic).

### Concepto Clave: Agente vs Herramienta

*   **Agente (Specialist)**: Tiene cerebro (LLM). Puede razonar, tomar decisiones y usar múltiples herramientas para llegar a un resultado.
    *   *Ejemplo*: `AddressSpecialist` (Recibe una dirección mal escrita, la corrige, busca coordenadas y confirma si es válida).
*   **Herramienta (Tool)**: No piensa. Solo ejecuta una instrucción y devuelve un dato.
    *   *Ejemplo*: `GeoCoding` (Toma "Calle 123" y devuelve `lat: 10, lng: 20`).

---

## 🛠️ Cómo agregar un Nuevo Agente Especialista

Sigue estos 5 pasos para extender la funcionalidad del sistema. Usaremos el **`dummy`** como base.

### Paso 1: Copiar el Esqueleto

Ve a `src/agents/specialized/` y copia la carpeta `dummy` con un nuevo nombre (ej. `analyst`).

```bash
cd src/agents/specialized
cp -r dummy analyst
```

### Paso 2: Definir el "Cerebro" (System Prompt)

Edita `src/agents/specialized/analyst/system_prompt.md`.

- Escribe instrucciones claras sobre qué debe hacer este agente.
- Define su personalidad y limitaciones.

### Paso 3: Implementar la Lógica (Código)

Edita `src/agents/specialized/analyst/mod.rs`:

1.  **Renombra los Structs**: Cambia `DummyArgs`, `DummyOutput` y `DummySpecialist` por `AnalystArgs`, etc.
2.  **Define Inputs/Outputs**: Ajusta los structs `Args` para reflejar qué datos necesita este agente del orquestador.
3.  **Implementa `Tool`**:
    - Actualiza `const NAME`.
    - En `definition()`: Describe **cuándo** el orquestador debe usar esta herramienta. ¡La descripción es clave para que el LLM orquestador entienda su uso!
    - En `call()`: Define qué sucede al ejecutarlo (llamar al sub-agente LLM, consultar una base de datos, etc.).

### Paso 4: Exportar el Módulo

Edita `src/agents/specialized/mod.rs` y añade tu nuevo módulo:

```rust
pub mod address;
pub mod damage;
pub mod dummy;
pub mod analyst; // <--- Tu nuevo agente
```

### Paso 5: Conectar al Orquestador

Finalmente, "presenta" al nuevo especialista con el jefe.
Edita `src/agents/orchestrator/mod.rs`:

1.  Importa tu módulo:
    ```rust
    use super::specialized::{address::AddressSpecialist, analyst::AnalystSpecialist};
    ```
2.  Instáncialo en `Orchestrator::new()`. Aquí decides qué "cerebro" (modelo) usará.
    ```rust
    // Usa el modelo principal (gemini_flash) o crea uno específico
    let analyst_tool = AnalystSpecialist::new(model.clone());
    ```
3.  Añádelo al builder del agente principal:
    ```rust
    let agent = AgentBuilder::new(model)
        .preamble(include_str!("system_prompt.md"))
        .tool(address_tool)
        .tool(analyst_tool) // <--- Conectado
        .build();
    ```

### Paso 6 (Opcional): Actualizar al Jefe

Si la herramienta es compleja, actualiza `src/agents/orchestrator/system_prompt.md` mencionando que ahora tiene capacidad de análisis, para reforzar su decisión de usarla.

---

## 📂 Estructura del Proyecto

```
src/
├── agents/                # Núcleo de la IA
│   ├── orchestrator/      # Agente Principal (Router)
│   ├── specialized/       # Sub-Agentes (Tools cognitivas)
│   └── tools/             # Herramientas Ejecutables (Funciones)
│       ├── geocoding.rs
│       └── ...
├── infra/                 # Proveedores de LLM y Observabilidad
├── api/                   # Endpoints HTTP
├── state.rs               # Estado global de la app
└── main.rs                # Entrypoint
```
