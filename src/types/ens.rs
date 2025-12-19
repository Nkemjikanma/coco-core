use alloy::primitives::{Address, address};

#[derive(Debug)]
pub struct EnsContractAddresses {
    pub ens_registry: Address,
    pub base_registrar: Address,
    pub registrar_controller: Address,
}

impl EnsContractAddresses {
    pub fn mainnet() -> Self {
        Self {
            ens_registry: address!("0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e"),
            base_registrar: address!("0x57f1887a8BF19b14fC0dF6Fd9B2acc9Af147eA85"),
            registrar_controller: address!("0x253553366Da8546fC250F225fe3d25d0C782303b"),
        }
    }
}
