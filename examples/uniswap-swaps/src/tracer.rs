use alloy_provider::{Provider, RootProvider};
use revm::{
    MainBuilder, MainContext,
    context::BlockEnv,
    context_interface::block::{BlobExcessGasAndPrice, calc_blob_gasprice},
    database::{AlloyDB, CacheDB, EmptyDB, WrapDatabaseAsync},
};
use revm_inspectors::tracing::TracingInspector;
use revm_inspectors::tracing::TracingInspectorConfig;

async fn build_tracer(provider: RootProvider, block_number: u64) -> eyre::Result<()> {
    let db = WrapDatabaseAsync::with_handle(
        AlloyDB::new(provider.clone(), block_number.into()),
        tokio::runtime::Handle::current(),
    );

    let block = provider
        .get_block_by_number(block_number.into())
        .hashes()
        .await?
        .unwrap();

    let mut evm = revm::Context::mainnet()
        .with_db(CacheDB::new(db))
        .with_block(BlockEnv {
            number: block.header.number,
            beneficiary: block.header.beneficiary,
            timestamp: block.header.timestamp,
            gas_limit: block.header.gas_limit,
            basefee: block.header.base_fee_per_gas.unwrap_or_default(),
            difficulty: block.header.difficulty,
            prevrandao: Some(block.header.mix_hash),
            blob_excess_gas_and_price: block
                .header
                .excess_blob_gas
                .map(|b| BlobExcessGasAndPrice::new(b, false)),
        })
        .build_mainnet();

    let mut insp =
        TracingInspector::new(TracingInspectorConfig::default_geth().set_record_logs(true));
    evm.with_inspector(&mut insp);

    Ok(())
}

async fn get_b(provider: &RootProvider, block_number: u64) -> eyre::Result<()> {
    // let header = self
    //     .cache()
    //     .get_header(block_hash)
    //     .await
    //     .map_err(Self::Error::from_eth_err)?;
    // let evm_env = self.evm_config().evm_env(&header);

    // Ok((evm_env, block_hash.into()))

    Ok(())
}
