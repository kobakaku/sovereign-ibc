use demo_nft_module::call::CallMessage;
use demo_nft_module::query::OwnerResponse;
use demo_nft_module::{NonFungibleToken, NonFungibleTokenConfig};
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Address, Context, Hasher, Module, ModuleInfo, Spec};
use sov_rollup_interface::stf::Event;
use sov_state::{DefaultStorageSpec, ProverStorage, WorkingSet};

pub type C = DefaultContext;
pub type Storage = ProverStorage<DefaultStorageSpec>;

pub fn generate_address(key: &str) -> <C as Spec>::Address {
    let hash = <C as Spec>::Hasher::hash(key.as_bytes());
    Address::from(hash)
}

#[test]
fn genesis_and_mint() {
    // Preparation
    let admin = generate_address("admin");
    let owner1 = generate_address("owner2");
    let owner2 = generate_address("owner2");
    let config: NonFungibleTokenConfig<C> = NonFungibleTokenConfig {
        admin,
        owners: vec![(0, owner1.clone())],
    };
    let mut working_set = WorkingSet::new(ProverStorage::temporary());
    let nft = NonFungibleToken::new();

    // Genesis
    let genesis_result = nft.genesis(&config, &mut working_set);
    assert!(genesis_result.is_ok());

    let query1: OwnerResponse<C> = nft.get_owner(0, &mut working_set);
    assert_eq!(query1.owner, Some(owner1));

    let query2: OwnerResponse<C> = nft.get_owner(1, &mut working_set);
    assert!(query2.owner.is_none());

    // Mint, anybody can mint
    let mint_message = CallMessage::Mint { id: 1 };
    let owner2_context = C::new(owner2.clone());
    nft.call(mint_message.clone(), &owner2_context, &mut working_set)
        .expect("Minting failed");

    assert_eq!(
        working_set.events()[0],
        Event::new("NFT mint", "A token with id 1 was minted")
    );
    let query3: OwnerResponse<C> = nft.get_owner(1, &mut working_set);
    assert_eq!(query3.owner, Some(owner2));

    // Try to mint again same token, should fail
    let mint_attempt = nft.call(mint_message, &owner2_context, &mut working_set);

    assert!(mint_attempt.is_err());
    let error_message = mint_attempt.err().unwrap().to_string();
    assert_eq!("Token with id 1 already exists", error_message);
}

#[test]
fn transfer() {
    // Preparation
    let admin = generate_address("admin");
    let admin_context = C::new(admin.clone());
    let owner1 = generate_address("owner2");
    let owner1_context = C::new(owner1.clone());
    let owner2 = generate_address("owner2");
    let config: NonFungibleTokenConfig<C> = NonFungibleTokenConfig {
        admin: admin.clone(),
        owners: vec![(0, admin.clone()), (1, owner1.clone()), (2, owner2.clone())],
    };
    let mut working_set = WorkingSet::new(ProverStorage::temporary());
    let nft = NonFungibleToken::new();
    nft.genesis(&config, &mut working_set).unwrap();

    let transfer_message = CallMessage::Transfer {
        id: 1,
        to: owner2.clone(),
    };

    // admin cannot transfer token of the owner1
    let transfer_attempt = nft.call(transfer_message.clone(), &admin_context, &mut working_set);

    assert!(transfer_attempt.is_err());
    let error_message = transfer_attempt.err().unwrap().to_string();
    assert_eq!("Only token owner can transfer token", error_message);

    let query_token_owner =
        |token_id: u64, working_set: &mut WorkingSet<Storage>| -> Option<Address> {
            let query: OwnerResponse<C> = nft.get_owner(token_id, working_set);
            query.owner
        };

    // Normal transfer
    let token1_owner = query_token_owner(1, &mut working_set);
    assert_eq!(Some(owner1), token1_owner);
    nft.call(transfer_message, &owner1_context, &mut working_set)
        .expect("Transfer failed");

    assert_eq!(
        working_set.events()[0],
        Event::new("NFT transfer", "A token with id 1 was transferred")
    );

    let token1_owner = query_token_owner(1, &mut working_set);
    assert_eq!(Some(owner2), token1_owner);

    // Attempt to transfer non existing token
    let transfer_message = CallMessage::Transfer { id: 3, to: admin };

    let transfer_attempt = nft.call(transfer_message, &owner1_context, &mut working_set);

    assert!(transfer_attempt.is_err());
    let error_message = transfer_attempt.err().unwrap().to_string();
    assert_eq!("Token with id 3 does not exist", error_message);
}

#[test]
fn burn() {
    // Preparation
    let admin = generate_address("admin");
    let admin_context = C::new(admin.clone());
    let owner1 = generate_address("owner2");
    let owner1_context = C::new(owner1.clone());
    let config: NonFungibleTokenConfig<C> = NonFungibleTokenConfig {
        admin,
        owners: vec![(0, owner1)],
    };

    let mut working_set = WorkingSet::new(ProverStorage::temporary());
    let nft = NonFungibleToken::new();
    nft.genesis(&config, &mut working_set).unwrap();

    let burn_message = CallMessage::Burn { id: 0 };

    // Only owner can burn token
    let burn_attempt = nft.call(burn_message.clone(), &admin_context, &mut working_set);

    assert!(burn_attempt.is_err());
    let error_message = burn_attempt.err().unwrap().to_string();
    assert_eq!("Only token owner can burn token", error_message);

    // Normal burn
    nft.call(burn_message.clone(), &owner1_context, &mut working_set)
        .expect("Burn failed");
    assert!(!working_set.events().is_empty());

    assert_eq!(
        working_set.events()[0],
        Event::new("NFT burn", "A token with id 0 was burned")
    );
    let query: OwnerResponse<C> = nft.get_owner(0, &mut working_set);

    assert!(query.owner.is_none());

    let burn_attempt = nft.call(burn_message, &owner1_context, &mut working_set);
    assert!(burn_attempt.is_err());
    let error_message = burn_attempt.err().unwrap().to_string();
    assert_eq!("Token with id 0 does not exist", error_message);
}