// Generated alternatives retained by the Reacon Rust union adapter.
use crate::models;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MailExperimentVariant {
    MailCadenceMessageExperimentVariant(models::MailCadenceMessageExperimentVariant),
    MailCadenceWorkflowExperimentVariant(models::MailCadenceWorkflowExperimentVariant),
}
impl Default for MailExperimentVariant {
    fn default() -> Self { Self::MailCadenceMessageExperimentVariant(Default::default()) }
}
