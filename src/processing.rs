use crate::graphql::*;
use anyhow::{anyhow, bail, Result};
use graphql_client::GraphQLQuery;

pub async fn get_staking_data(
    endpoint: &str,
    epoch: i64,
) -> Result<(String, String, String, Vec<LedgerAccount>)> {
    let request_body = StakingData::build_query(staking_data::Variables {});
    let data: staking_data::ResponseData = graphql_query(endpoint, &request_body).await?;

    let best_chain = match &data.best_chain {
        None => bail!("best_chain is None"),
        Some(best_chain) => match best_chain.len() == 1 {
            false => bail!("should only have 1 best_chain"),
            true => &best_chain[0],
        },
    };
    let best_epoch = &best_chain.protocol_state.consensus_state.epoch;

    let (seed, total_currency, ledger_hash) = if best_epoch != &epoch.to_string() {
        let request_body =
            StakingDataExplorer::build_query(staking_data_explorer::Variables { epoch });
        let data: staking_data_explorer::ResponseData =
            graphql_query(MINA_EXPLORER_ENDPOINT, &request_body).await?;

        let explorer_staking_epoch_data = data.blocks[0]
            .as_ref()
            .ok_or(anyhow!("no block"))?
            .protocol_state
            .as_ref()
            .ok_or(anyhow!("no protocol state"))?
            .consensus_state
            .as_ref()
            .ok_or(anyhow!("no consensus state"))?
            .staking_epoch_data
            .as_ref()
            .ok_or(anyhow!("no staking epoch data"))?;

        let ledger = &explorer_staking_epoch_data
            .ledger
            .as_ref()
            .ok_or(anyhow!("no ledger"))?;
        let seed = explorer_staking_epoch_data
            .seed
            .as_ref()
            .ok_or(anyhow!("no seed"))?;
        let total_currency = &ledger
            .total_currency
            .as_ref()
            .ok_or(anyhow!("no total currency"))?;
        let ledger_hash = explorer_staking_epoch_data
            .ledger
            .as_ref()
            .ok_or(anyhow!("no ledger"))?
            .hash
            .as_ref()
            .ok_or(anyhow!("no hash"))?;

        (
            seed.clone(),
            total_currency.to_string(),
            ledger_hash.clone(),
        )
    } else {
        let staking_epoch_data = &best_chain.protocol_state.consensus_state.staking_epoch_data;
        let seed = &staking_epoch_data.seed;
        let total_currency = &staking_epoch_data.ledger.total_currency;
        let ledger_hash = &staking_epoch_data.ledger.hash;

        (seed.clone(), total_currency.clone(), ledger_hash.clone())
    };

    let url = format!(
        "https://raw.githubusercontent.com/zkvalidator/mina-vrf-rs/main/data/epochs/{}.json",
        ledger_hash,
    );

    let ledger: Vec<LedgerAccountJson> =
        serde_json::from_slice(&reqwest::get(url).await?.bytes().await?.to_vec())?;

    let delegators = extract_delegators(&ledger);

    Ok((seed, total_currency, ledger_hash, delegators))
}

fn extract_delegators(ledger: &[LedgerAccountJson]) -> Vec<LedgerAccount> {
    let delegators = ledger
        .into_iter()
        .enumerate()
        .map(|(i, a)| LedgerAccount {
            pk: a.pk.clone(),
            balance: a.balance.clone(),
            delegate: a.delegate.clone(),
            index: i as i64,
        })
        .collect();

    delegators
}
