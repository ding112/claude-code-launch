use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventModel {
    pub event_id: String,
    pub session_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionModel {
    pub session_id: String,
    pub project_name: String,
    pub agent_type: String,
    pub first_seen_at_ms: i64,
    pub last_active_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvaluationModel {
    pub evaluation_id: String,
    pub session_id: String,
    pub event_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub risk_level: String,
    pub efficiency_level: String,
    pub suggestion: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SettingsModel {
    pub provider: String,
    pub model: String,
    pub enabled: bool,
    pub sampling_rate: u32,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MinimalSchemaBundle {
    pub event: EventModel,
    pub session: SessionModel,
    pub evaluation: EvaluationModel,
    pub settings: SettingsModel,
}

pub fn minimal_models_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(MinimalSchemaBundle))
        .expect("failed to serialize minimal models schema")
}
