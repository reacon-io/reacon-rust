// Generated alternatives retained by the Reacon Rust union adapter.
use crate::models;
use serde::{Deserialize, Serialize};
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyProductToolInput {}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProductToolRequestInput {
    ProductDiscoverCompaniesInput(models::ProductDiscoverCompaniesInput),
    ProductDiscoverPeopleInput(models::ProductDiscoverPeopleInput),
    ProductDomainFinderInput(models::ProductDomainFinderInput),
    ProductEmailCountInput(models::ProductEmailCountInput),
    ProductPersonEnrichInput(models::ProductPersonEnrichInput),
    ProductLeadsListInput(models::ProductLeadsListInput),
    ProductLeadGetInput(models::ProductLeadGetInput),
    ProductLeadCreateInput(models::ProductLeadCreateInput),
    ProductLeadUpdateInput(models::ProductLeadUpdateInput),
    ProductLeadDeleteInput(models::ProductLeadDeleteInput),
    ProductLeadBulkDeleteInput(models::ProductLeadBulkDeleteInput),
    ProductLeadTagCreateInput(models::ProductLeadTagCreateInput),
    ProductLeadTagAssignInput(models::ProductLeadTagAssignInput),
    ProductCustomAttributeCreateInput(models::ProductCustomAttributeCreateInput),
    ProductLeadListUpdateInput(models::ProductLeadListUpdateInput),
    ProductLeadListDeleteInput(models::ProductLeadListDeleteInput),
    ProductLeadListAddLeadInput(models::ProductLeadListAddLeadInput),
    ProductCompaniesListInput(models::ProductCompaniesListInput),
    ProductCompanyTrackInput(models::ProductCompanyTrackInput),
    ProductCompanyUpdateInput(models::ProductCompanyUpdateInput),
    ProductCompanyDeleteInput(models::ProductCompanyDeleteInput),
    ProductCompanyListAddInput(models::ProductCompanyListAddInput),
    ProductSequenceRecipientsListInput(models::ProductSequenceRecipientsListInput),
    ProductSequenceRecipientsAddInput(models::ProductSequenceRecipientsAddInput),
    ProductSequenceRecipientAddInput(models::ProductSequenceRecipientAddInput),
    ProductSequenceRecipientCancelInput(models::ProductSequenceRecipientCancelInput),
    ProductSequenceStartInput(models::ProductSequenceStartInput),
    ProductConnectedAppPushInput(models::ProductConnectedAppPushInput),
    Empty(EmptyProductToolInput),
}
impl Default for ProductToolRequestInput {
    fn default() -> Self { Self::Empty(Default::default()) }
}
