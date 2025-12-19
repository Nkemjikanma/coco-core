use crate::errors::CocoError;
use crate::services::ens::BaseRegistrarImplementation::BaseRegistrarImplementationInstance;
use crate::services::ens::ETHRegistrarController::ETHRegistrarControllerInstance;
use crate::types::{
    alloy_providers::AppProvider,
    api::{AppState, CheckExpiryResponse, CheckNameResponse, PriceResponse},
};
use alloy::{
    primitives::{Address, B256, FixedBytes, U256, keccak256},
    providers::{MulticallError, Network, Provider},
    sol,
};
use alloy_ens::namehash;

// ENS REGISTRY
// sol! {
//     #[allow(missing_docs)]
//     #[sol(rpc)]
//     contract ENSRegistry {
//         function owner(bytes32 node) public view returns (address);
//     }
// }

// BASE REGISTRY
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract BaseRegistrarImplementation {
        function nameExpires(uint256 id) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
    }
}

// ETH REGISTRAR CONTROLLER
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract ETHRegistrarController {

        struct Price {
            uint256 base;
            uint256 premium;
        }

    function available(string calldata label) public view returns (bool);

    function rentPrice(string calldata label, uint256 duration) public view returns (Price price);

    }
}

pub async fn check_name_availability(
    state: &AppState,
    names: &[String],
) -> Result<Vec<CheckNameResponse>, CocoError> {
    let mut normalised_names = Vec::new();
    let mut failures = 0;

    // normalise names
    for name in names.into_iter() {
        match normalise_and_hash_name(&name) {
            Ok(data) => normalised_names.push(data),
            Err(_) => failures += 1,
        }
    }

    let labels: Vec<String> = normalised_names.iter().map(|n| n.label.clone()).collect();
    let namehashes: Vec<B256> = normalised_names.iter().map(|n| n.name_hash).collect();
    let labelhashes: Vec<B256> = normalised_names.iter().map(|n| n.label_hash).collect();

    // let ens_registry = ENSRegistry::new(state.ens_contract_addresses.ens_registry, &state.provider);
    let base_registrar = BaseRegistrarImplementation::new(
        state.ens_contract_addresses.base_registrar,
        &state.provider,
    );
    let controller = ETHRegistrarController::new(
        state.ens_contract_addresses.registrar_controller,
        &state.provider,
    );

    // for availability_result in
    let (availabilities, prices, owners, expiries) = tokio::try_join!(
        fetch_availability(&state.provider, &controller, &labels),
        fetch_rent_prices(&state.provider, &controller, &labels),
        fetch_owners(&state.provider, &base_registrar, &labelhashes),
        fetch_expires(&state.provider, &base_registrar, &labelhashes)
    )
    .map_err(|e| CocoError::Ens(e))?;

    // format response for handler
    let mut out = Vec::with_capacity(normalised_names.len());

    for i in 0..normalised_names.len() {
        let available = availabilities[i];

        out.push(CheckNameResponse {
            name: normalised_names[i].name.clone(),
            available,
            price: if available {
                Some(PriceResponse {
                    base: prices[i].base,
                    premium: prices[i].premium,
                })
            } else {
                None
            },
            owner: if available { None } else { Some(owners[i]) },
            expires: if available { None } else { Some(expiries[i]) },
        })
    }

    Ok(out)
}

pub async fn check_name_expiry(
    state: &AppState,
    names: &[String],
) -> Result<Vec<CheckExpiryResponse>, CocoError> {
    let mut normalised_names = Vec::new();
    let mut failures = 0;

    // normalise names
    for name in names.into_iter() {
        match normalise_and_hash_name(&name) {
            Ok(data) => normalised_names.push(data),
            Err(_) => failures += 1,
        }
    }

    let labels: Vec<String> = normalised_names.iter().map(|n| n.label.clone()).collect();
    let labelhashes: Vec<B256> = normalised_names.iter().map(|n| n.label_hash).collect();

    let base_registrar = BaseRegistrarImplementation::new(
        state.ens_contract_addresses.base_registrar,
        &state.provider,
    );

    let controller = ETHRegistrarController::new(
        state.ens_contract_addresses.registrar_controller,
        &state.provider,
    );

    let (availabilities, expiries) = tokio::try_join!(
        fetch_availability(&state.provider, &controller, &labels),
        fetch_expires(&state.provider, &base_registrar, &labelhashes)
    )
    .map_err(|e| CocoError::Ens(e))?;

    let mut out = Vec::with_capacity(normalised_names.len());

    for i in 0..normalised_names.len() {
        let available = availabilities[i];

        out.push(CheckExpiryResponse {
            name: normalised_names[i].name.clone(),
            available,
            expiry_date: Some(expiries[i]),
        })
    }

    Ok(out)
}

async fn fetch_availability<P, N>(
    provider: &AppProvider,
    controller: &ETHRegistrarControllerInstance<P, N>,
    labels: &Vec<String>,
) -> Result<Vec<bool>, MulticallError>
where
    P: Provider<N>,
    N: Network,
{
    let mut multicall = provider.multicall().dynamic();
    for label in labels {
        multicall = multicall.add_dynamic(controller.available(label.to_string()));
    }
    multicall.aggregate().await
}

async fn fetch_rent_prices<P, N>(
    provider: &AppProvider,
    controller: &ETHRegistrarControllerInstance<P, N>,
    labels: &Vec<String>,
) -> Result<Vec<ETHRegistrarController::Price>, MulticallError>
where
    P: Provider<N>,
    N: Network,
{
    let duration = U256::from(365u64 * 24 * 60 * 60); // example: 1 year in seconds

    let mut multicall = provider.multicall().dynamic();
    for label in labels {
        multicall = multicall.add_dynamic(controller.rentPrice(label.to_string(), duration));
    }

    multicall.aggregate().await
}

async fn fetch_owners<P, N>(
    provider: &AppProvider,
    base_registrar: &BaseRegistrarImplementationInstance<P, N>,
    label_hashes: &[B256],
) -> Result<Vec<Address>, MulticallError>
where
    P: Provider<N>,
    N: Network,
{
    let mut multicall = provider.multicall().dynamic();
    for label_hash in label_hashes {
        let id = U256::from_be_bytes(**label_hash);
        multicall = multicall.add_dynamic(base_registrar.ownerOf(id));
    }
    multicall.aggregate().await
}

async fn fetch_expires<P, N>(
    provider: &AppProvider,
    base_registrar: &BaseRegistrarImplementationInstance<P, N>,
    label_hashes: &[B256],
) -> Result<Vec<U256>, MulticallError>
where
    P: Provider<N>,
    N: Network,
{
    let mut multicall = provider.multicall().dynamic();
    for label_hash in label_hashes {
        let id = U256::from_be_bytes(**label_hash);
        multicall = multicall.add_dynamic(base_registrar.nameExpires(id));
    }
    multicall.aggregate().await
}

struct NormalisedNameData {
    name: String,
    label: String,
    name_hash: FixedBytes<32>,
    label_hash: FixedBytes<32>,
}

pub fn normalise_and_hash_name(name: &String) -> Result<NormalisedNameData, CocoError> {
    // if name is empty
    if name.trim().is_empty() {
        return Err(CocoError::InvalidName("No name provided".to_string()));
    }

    // get label and extension
    let mut trimmed_name = name.trim().to_lowercase();

    if !trimmed_name.ends_with(".eth") {
        trimmed_name = trimmed_name + ".eth";
    }

    let label_and_ext = trimmed_name.split(".").collect::<Vec<&str>>();

    // currently support only 2LD eg alice.eth
    if label_and_ext.len() > 2 {
        return Err(CocoError::InvalidName(
            format!("Only 2LD names supported currently. {} is invalid", name).to_string(),
        ));
    }

    let label = label_and_ext[0];
    let ext = label_and_ext[1];

    if label.trim().is_empty() || ext.to_lowercase() != "eth" {
        return Err(CocoError::InvalidName(
            format!("Something wrong with {}. Can't seem to normalise it", name).to_string(),
        ));
    }

    let name_hash = namehash(&name);
    let label_hash = keccak256(label.as_bytes());

    println!("{:?}", trimmed_name);
    Ok(NormalisedNameData {
        name: trimmed_name.to_string(),
        label: label.to_string(),
        name_hash,
        label_hash,
    })
}
