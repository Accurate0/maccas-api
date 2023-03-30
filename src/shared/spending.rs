use crate::{
    database::types::AuditActionType,
    types::{
        api::{GetDealsOffer, UserSpending},
        audit::AuditEntry,
    },
};
use itertools::Itertools;

pub fn generate_spending_information(audit_entries: &[AuditEntry]) -> UserSpending {
    let items = audit_entries
        .iter()
        .sorted_by_key(|entry| entry.offer.deal_uuid.clone())
        .group_by(|entry| entry.offer.deal_uuid.clone())
        .into_iter()
        .map(|e| e.1.collect::<Vec<&AuditEntry>>())
        .collect_vec();

    let mut final_list = Vec::new();
    for item in &items {
        let mut list = Vec::new();
        let all_add_count = item
            .iter()
            .filter(|e| e.action == AuditActionType::Add)
            .collect_vec()
            .len();

        let all_remove_count = item
            .iter()
            .filter(|e| e.action == AuditActionType::Remove)
            .collect_vec()
            .len();

        if all_add_count >= all_remove_count {
            let final_count = all_add_count - all_remove_count;

            if final_count > 0 {
                list.append(
                    &mut item
                        .iter()
                        .take(final_count)
                        .map(|e| GetDealsOffer::from(e.offer.clone()))
                        .collect_vec(),
                );
            }

            final_list.append(&mut list);
        } else {
            log::warn!("{:?}", item);
            log::warn!("add: {}, remove: {}", all_add_count, all_remove_count);
            log::warn!("this is a bug, should never occur");
            // lets ignore it instead of data fix
        }
    }

    let total_cost = audit_entries
        .iter()
        .map(|entry| {
            let price = entry.offer.price.unwrap_or_default();
            match entry.action {
                AuditActionType::Add => price,
                AuditActionType::Remove => -price,
            }
        })
        .sum();

    UserSpending {
        total: total_cost,
        items: final_list,
    }
}
