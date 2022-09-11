use crate::constants::mc_donalds;
use libmaccas::ApiClient;

#[async_trait]
pub trait ApiClientExtensions {
    async fn remove_all_from_deal_stack_for(&self, account_name: &str)
        -> Result<(), anyhow::Error>;
}

#[async_trait]
impl ApiClientExtensions for ApiClient<'_> {
    async fn remove_all_from_deal_stack_for(
        &self,
        account_name: &str,
    ) -> Result<(), anyhow::Error> {
        // honestly, we don't want failures here, so we'll probably just suppress them...
        log::info!("{}: trying to clean deal stack", account_name);
        let deal_stack = self
            .get_offers_dealstack(mc_donalds::default::OFFSET, &mc_donalds::default::STORE_ID)
            .await;
        if let Ok(deal_stack) = deal_stack {
            if let Some(deal_stack) = deal_stack.body.response {
                if let Some(deal_stack) = deal_stack.deal_stack {
                    for deal in deal_stack {
                        log::info!("{}: removing offer -> {}", account_name, deal.offer_id);
                        self.remove_from_offers_dealstack(
                            &deal.offer_id,
                            &deal.offer_proposition_id,
                            mc_donalds::default::OFFSET,
                            &mc_donalds::default::STORE_ID,
                        )
                        .await
                        .ok();
                    }
                }
            }
        };
        Ok(())
    }
}
