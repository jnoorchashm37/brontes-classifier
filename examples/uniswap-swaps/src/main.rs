use std::path::Path;

use alloy_primitives::{TxHash, address, b256};

use brontes_classifier::{TraceClassifier, types::ClassifiedTx};
use brontes_tracer::{TracingClient, types::executor::BrontesTaskManager};
use uniswap_swaps::types::{
    Actions, DataCache, Protocol, UniswapProtocolTokens, UniswapSwapClassifer,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let uni_v2_block_number = 22077264;
    let uni_v2_tx_hash =
        b256!("0xa45cbc6a6caf1d71f8af4d1e8aec42d1ff4ae7d17e92a32b7ab6dfd74117d63a");
    let uni_v2_pool_addr = address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");
    let uni_v2_pool = UniswapProtocolTokens {
        protocol: Protocol::UniswapV2,
        sorted_tokens: [
            address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            address!("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        ],
    };

    let uni_v3_block_number = 22077285;
    let uni_v3_tx_hash =
        b256!("0xb230102d52a19ede8e48413819399b89253847fe4a1eba40d2eca73b08148a1e");
    let uni_v3_pool_addr = address!("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    let uni_v3_pool = UniswapProtocolTokens {
        protocol: Protocol::UniswapV3,
        sorted_tokens: [
            address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            address!("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        ],
    };

    let data_cache = DataCache::new(vec![
        (uni_v2_pool_addr, uni_v2_pool),
        (uni_v3_pool_addr, uni_v3_pool),
    ]);

    let manager = BrontesTaskManager::new(tokio::runtime::Handle::current(), true);

    let db_path = "/home/data/reth/db";
    let classifier = UniswapSwapTracer {
        data_cache,
        tracer: TracingClient::new(Path::new(db_path), 1000, manager.executor()),
    };

    let v2_result = classifier
        .get_actions_for_tx_hash(uni_v2_block_number, uni_v2_tx_hash)
        .await?;

    println!("V2:\n{v2_result:?}\n\n");

    let v3_result = classifier
        .get_actions_for_tx_hash(uni_v3_block_number, uni_v3_tx_hash)
        .await?;
    println!("V3:\n{v3_result:?}\n\n");
    Ok(())
}

struct UniswapSwapTracer {
    data_cache: DataCache,
    tracer: TracingClient,
}

impl UniswapSwapTracer {
    async fn get_actions_for_tx_hash(
        &self,
        block_number: u64,
        tx_hash_to_get: TxHash,
    ) -> eyre::Result<ClassifiedTx<Actions>> {
        let classifed_block = self.classify_block(block_number).await?;
        let classified_txs = classifed_block
            .transactions
            .into_iter()
            .filter(|tx| tx.tx_hash == tx_hash_to_get && !tx.traces.is_empty())
            .collect::<Vec<_>>();

        assert_ne!(classified_txs.len(), 0);

        Ok(classified_txs.first().unwrap().clone())
    }
}

impl TraceClassifier<UniswapSwapClassifer> for UniswapSwapTracer {
    type DataProvider = DataCache;

    fn data_provider(&self) -> &Self::DataProvider {
        &self.data_cache
    }

    fn eth_provider(&self) -> &TracingClient {
        &self.tracer
    }
}
