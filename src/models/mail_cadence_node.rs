// Generated alternatives retained by the Reacon Rust union adapter.
use crate::models;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MailCadenceNode {
    MailCadenceNodeAnyOf(models::MailCadenceNodeAnyOf),
    MailCadenceNodeAnyOf1(models::MailCadenceNodeAnyOf1),
    MailCadenceNodeAnyOf2(models::MailCadenceNodeAnyOf2),
    MailCadenceNodeAnyOf3(models::MailCadenceNodeAnyOf3),
    MailCadenceNodeAnyOf4(models::MailCadenceNodeAnyOf4),
    MailCadenceNodeAnyOf5(models::MailCadenceNodeAnyOf5),
    MailCadenceNodeAnyOf6(models::MailCadenceNodeAnyOf6),
}
impl Default for MailCadenceNode {
    fn default() -> Self { Self::MailCadenceNodeAnyOf(Default::default()) }
}
