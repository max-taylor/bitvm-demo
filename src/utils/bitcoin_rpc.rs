use bitcoin::{Address, Amount};
use bitcoincore_rpc::{json::GetTransactionResult, Auth, Client, RpcApi};

pub fn setup_client_and_fund_prover(
    wallet_name: &str,
    to_address: &Address,
    amount: u64,
) -> (Client, GetTransactionResult) {
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

    rpc.generate_to_address(101, &wallet_address.assume_checked())
        .unwrap();

    let initial_fund_txid = rpc
        .send_to_address(
            to_address,
            Amount::from_sat(amount),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(|e| panic!("Failed to send to address: {}", e));

    let initial_fund_tx = rpc
        .get_transaction(&initial_fund_txid, None)
        .unwrap_or_else(|e| panic!("Failed to get transaction: {}", e));

    (rpc, initial_fund_tx)
}
