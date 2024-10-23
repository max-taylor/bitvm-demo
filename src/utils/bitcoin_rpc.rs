use bitcoin::{Address, Amount};
use bitcoincore_rpc::{json::GetTransactionResult, Auth, Client, RpcApi};

pub fn setup_client_and_fund_prover(
    wallet_name: &str,
    to_address: &Address,
    amount: Amount,
) -> (Client, GetTransactionResult, u32) {
    let rpc = Client::new(
        "http://localhost:18443",
        Auth::UserPass("admin".to_string(), "admin".to_string()),
    )
    .unwrap();

    // rpc.create_wallet(WALLET_NAME, None, None, None, None)
    //     .unwrap();
    //
    let _ = rpc.load_wallet(wallet_name);

    let wallet_address = rpc.get_new_address(Some(wallet_name), None).unwrap();

    // TODO: Fails on first load when the wallet doesn't exist
    // let wallet_address = rpc
    //     .get_new_address(Some(WALLET_NAME), None)
    //     .unwrap_or_else(|e| {
    //         dbg!(e);
    //         rpc.create_wallet(WALLET_NAME, None, None, None, None)
    //             .unwrap();
    //         rpc.get_new_address(Some(WALLET_NAME), None).unwrap()
    //     });

    rpc.generate_to_address(2, &wallet_address.clone().assume_checked())
        .unwrap();

    let initial_fund_txid = rpc
        .send_to_address(to_address, amount, None, None, None, None, None, None)
        .unwrap_or_else(|e| panic!("Failed to send to address: {}", e));
    // Find the correct output (vout) that matches the 'to_address'

    // thread::sleep(Duration::from_secs(5));

    let initial_fund_tx = rpc
        .get_transaction(&initial_fund_txid, None)
        .unwrap_or_else(|e| panic!("Failed to get transaction: {}", e));

    // Stupidly the send_to_address method may create multiple tx_outs, this pulls out the tx_out
    // that is needed
    let found_vout: u32 = initial_fund_tx
        .transaction()
        .unwrap()
        .output
        .iter()
        .enumerate() // Get the index (vout) along with the TxOut
        .find(|(_, txout)| txout.script_pubkey == to_address.script_pubkey())
        .map(|(vout, _)| vout)
        .expect("Failed to find the correct vout for the to_address")
        .try_into()
        .unwrap();

    (rpc, initial_fund_tx, found_vout)
}
