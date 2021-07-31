use anyhow::{anyhow, Result};
use graphql_client::*;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

type UInt32 = String;
type UInt64 = String;

pub const MINA_EXPLORER_ENDPOINT: &str = "https://graphql.minaexplorer.com";
pub const DEFAULT_LOCAL_ENDPOINT: &str = "http://localhost:3085/graphql";

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "contrib/regen_schema.graphql",
    query_path = "contrib/query.graphql",
    response_derives = "Debug,Serialize,PartialEq"
)]
pub struct StakingData;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "contrib/explorer_regen_schema.graphql",
    query_path = "contrib/explorer_query.graphql",
    response_derives = "Debug,Serialize,PartialEq"
)]
pub struct StakingDataExplorer;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchGenerateWitnessSingleRequest {
    pub global_slot: String,
    pub epoch_seed: String,
    pub delegator_index: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchPatchWitnessSingleVrfThresholdRequest {
    pub delegated_stake: String,
    pub total_stake: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchPatchWitnessSingleRequest {
    pub message: BatchGenerateWitnessSingleRequest,
    pub public_key: String,
    pub c: String,
    pub s: String,
    #[serde(rename = "ScaledMessageHash")]
    pub scaled_message_hash: Vec<String>,
    pub vrf_threshold: Option<BatchPatchWitnessSingleVrfThresholdRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchCheckWitnessSingleRequest {
    pub message: BatchGenerateWitnessSingleRequest,
    pub public_key: String,
    pub c: String,
    pub s: String,
    #[serde(rename = "ScaledMessageHash")]
    pub scaled_message_hash: Vec<String>,
    pub vrf_threshold: BatchPatchWitnessSingleVrfThresholdRequest,
    pub vrf_output: String,
    pub vrf_output_fractional: f64,
    pub threshold_met: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LedgerAccountJson {
    pub pk: String,
    pub balance: String,
    pub delegate: String,
}

pub struct LedgerAccount {
    #[allow(unused)]
    pub pk: String,
    pub balance: String,
    pub delegate: String,
    pub index: i64,
}

pub async fn graphql_query<U: IntoUrl, B: Serialize + ?Sized, R: DeserializeOwned>(
    endpoint: U,
    request_body: &B,
) -> Result<R> {
    let client = reqwest::Client::new();
    let res = client.post(endpoint).json(request_body).send().await?;
    let response_body: Response<R> = res.json().await?;
    if let Some(es) = response_body.errors {
        for e in es {
            log::error!("{}", e);
        }
        return Err(anyhow!("response_body contains errors"));
    }

    response_body.data.ok_or(anyhow!("response_body was none"))
}
