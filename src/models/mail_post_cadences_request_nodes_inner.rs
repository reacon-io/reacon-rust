// Generated alternatives retained by the Reacon Rust union adapter.
use crate::models;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MailPostCadencesRequestNodesInner {
    MailPostCadencesRequestNodesInnerAnyOf(models::MailPostCadencesRequestNodesInnerAnyOf),
    MailPostCadencesRequestNodesInnerAnyOf1(models::MailPostCadencesRequestNodesInnerAnyOf1),
    MailPostCadencesRequestNodesInnerAnyOf2(models::MailPostCadencesRequestNodesInnerAnyOf2),
    MailPostCadencesRequestNodesInnerAnyOf3(models::MailPostCadencesRequestNodesInnerAnyOf3),
    MailPostCadencesRequestNodesInnerAnyOf4(models::MailPostCadencesRequestNodesInnerAnyOf4),
    MailPostCadencesRequestNodesInnerAnyOf5(models::MailPostCadencesRequestNodesInnerAnyOf5),
    MailPostCadencesRequestNodesInnerAnyOf6(models::MailPostCadencesRequestNodesInnerAnyOf6),
}
impl Default for MailPostCadencesRequestNodesInner {
    fn default() -> Self { Self::MailPostCadencesRequestNodesInnerAnyOf(Default::default()) }
}
