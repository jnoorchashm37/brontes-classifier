use alloy_primitives::Address;

pub trait DataContext<T> {
    fn get_protocol(&self, target_address: Address) -> eyre::Result<T>;

    fn get_protocol_tokens_sorted(&self, target_address: Address) -> eyre::Result<Vec<Address>>;
}
