use std::sync::Arc;
use alloy::primitives::{U256};
use alloy::providers::{Provider, ProviderBuilder};
use tracing::{info};
use config::Config;
use crate::uniswapv3pool::{UniswapV3Pool};
use super::*;

#[tokio::test]
async fn test_calculate_amount_out() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let cfg = Config::default();
    let net = config::Network::Ethereum;
    let provider = cfg.providers.get(&net).unwrap().clone();
    let addresses = cfg.addresses.get(&net).unwrap().clone();
    let provider = Arc::new(ProviderBuilder::new().on_http(provider.api.parse().unwrap()));

    let block = provider
        .get_block_number()
        .await
        .unwrap();

    let mut pool = UniswapV3Pool::new(
        addresses.uniswap_v3.pools.get("USDC_WETH").unwrap().clone(),
        addresses.uniswap_v3.periphery.clone());

    pool.data = UniswapV3Pool::fetch_data(
        &pool.metadata,
        provider.clone(),
        block.into(),
    ).await.unwrap();

    pool.state = UniswapV3Pool::fetch_state(
        &pool.metadata,
        &pool.data,
        provider.clone(),
        block.into(),
    ).await.unwrap();

    let amount_in = U256::from(U256::from(10).pow(U256::from(18)));
    let tok_in = pool.data.tok0;
    let tok_out = pool.get_other_token(tok_in);
    let amount_out_local = local::calc_amount_out(
        amount_in,
        tok_in,
        pool.data.clone().into(),
        pool.state.clone().into(),
    ).unwrap();
    info!(?amount_out_local);

    let amount_out_quoter = quoter::calc_amount_out(
        pool.metadata.periphery.quoter.clone(),
        amount_in,
        tok_in,
        tok_out,
        pool.data.fee,
        provider.clone(),
        block.into(),
    )
        .await
        .unwrap();
    info!(?amount_out_quoter);

    let amount_out_quoter2 = quoter2::calc_amount_out(
        pool.metadata.periphery.quoter_v2.clone(),
        amount_in,
        tok_in,
        tok_out,
        pool.data.fee,
        provider.clone(),
        block.into(),
    )
        .await
        .unwrap();
    info!(?amount_out_quoter2);

    assert_eq!(amount_out_quoter, amount_out_quoter2);
    assert_eq!(amount_out_quoter, amount_out_local); // FAILS
}
