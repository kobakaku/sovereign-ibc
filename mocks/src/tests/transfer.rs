use std::time::Duration;

use ibc_core::host::ValidationContext;
use ibc_core::primitives::Signer;
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use sov_bank::get_genesis_token_address;
use sov_modules_api::default_context::DefaultContext;
use tokio::time::sleep;

use crate::relayer::handle::Handle;
use crate::setup::sovereign_cosmos_setup;
use crate::sovereign::builder::DefaultBuilder;

#[tokio::test]
async fn test_sdk_token_transfer() {
    let mut sov_builder = DefaultBuilder::default();

    let token = sov_builder.get_tokens().first().unwrap();

    let token_address = get_genesis_token_address::<DefaultContext>(&token.token_name, token.salt);

    let sender_on_sov = token.address_and_balances[0].0;

    let receiver_on_cos = dummy_account_id();

    let transfer_amount = 100;

    let expected_sender_balance = token.address_and_balances[0].1 - transfer_amount;

    let rly = sovereign_cosmos_setup(&mut sov_builder, true).await;

    let msg_sdk_token_transfer = rly.src_chain_ctx().build_sdk_transfer(
        token_address,
        Signer::from(sender_on_sov.to_string()),
        receiver_on_cos,
        transfer_amount,
    );

    rly.src_chain_ctx().send_msg(vec![msg_sdk_token_transfer]);

    // Checks that the token has been transferred
    let escrowed_token = rly
        .src_chain_ctx()
        .querier()
        .get_escrowed_token_address(token_address.to_string())
        .unwrap();

    assert_eq!(escrowed_token, token_address);

    // Checks that the sender balance have been updated
    let sender_balance = rly
        .src_chain_ctx()
        .querier()
        .get_balance_of(sender_on_sov, token_address);

    assert_eq!(sender_balance, expected_sender_balance);
}

#[tokio::test]
async fn test_recv_packet() {
    let mut sov_builder = DefaultBuilder::default();

    let rly = sovereign_cosmos_setup(&mut sov_builder, true).await;

    let msg_create_client = rly.build_msg_create_client();

    rly.src_chain_ctx().send_msg(vec![msg_create_client]);

    sleep(Duration::from_secs(1)).await;

    let target_height = rly.dst_chain_ctx().query_ibc().host_height().unwrap();

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height);

    rly.src_chain_ctx().send_msg(vec![msg_update_client]);

    let _cs = rly
        .src_chain_ctx()
        .query_ibc()
        .client_state(rly.src_client_id())
        .unwrap();

    // FIXME: This test already fails as there must be a send packet on the mock
    // cosmos chain

    // let msg_recv_packet = build_msg_recv_packet_for_sov(&rly, cs.latest_height()).await;

    // rly.src_chain_ctx().send_msg(vec![msg_recv_packet]);
}