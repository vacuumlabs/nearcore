use actix;

// File IO Imports
use std::fs::File;
use std::io::prelude::*;
use std::io::{LineWriter, Write};
use std::path::Path;

use clap::Clap;
use tokio::sync::mpsc;
use tracing::info;

use configs::{init_logging, Opts, SubCommand};
use near_indexer;

mod configs;

fn assign_log(logs: &Vec<Vec<String>>) -> String {
    /*
    Example of log parsing
    038c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925000000000000000000000000cbda96b3f2b8eb962f97ae50c3852ca976740e2b000000000000000000000000db9217df5c41887593e463cfa20036b62a4e331cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
    03 // number of topics
    8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925 // keccak 256 hash of Approval(address,address,uint256)
    000000000000000000000000cbda96b3f2b8eb962f97ae50c3852ca976740e2b // owner address (my address)
    000000000000000000000000db9217df5c41887593e463cfa20036b62a4e331c // spender address (exchange proxy address)
    ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff // amount (max uint256)
    */

    // Event name hashes
    let approval_hash = "8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";
    let log_new_pool_hash = "8ccec77b0cb63ac2cafd0f5de8cdfadab91ce656d262240ba8a6343bccc5f945";
    let log_join_pool_hash = "63982df10efd8dfaaaa0fcc7f50b2d93b7cba26ccc48adee2873220d485dc39a";
    let log_exit_pool_hash = "e74c91552b64c2e2e7bd255639e004e693bd3e1d01cc33e65610b86afcc1ffed";
    let log_swap_hash = "908fb5ee8f16c6bc9bc3690973819f32a4d4b10188134543c88706e0e1d43378";
    let transfer_hash = "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    let ownership_transferred_hash = "8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0";

    // Anonymous Event Hashes
    let log_handleSetSwapFee_hash = "0x34e1990700000000000000000000000000000000000000000000000000000000";
    let log_handleSetController_hash = "0x92eefe9b00000000000000000000000000000000000000000000000000000000";
    let log_handleSetPublicSwap_hash = "0x49b5955200000000000000000000000000000000000000000000000000000000";
    let log_handleFinalize_hash = "0x4bb278f300000000000000000000000000000000000000000000000000000000";
    let log_handleRebind1_hash = "0x3fdddaa200000000000000000000000000000000000000000000000000000000";
    let log_handleRebind2_hash = "0xe4e1e53800000000000000000000000000000000000000000000000000000000"; // why are there two?
    let log_handleUnbind_hash = "0xcf5e7bd300000000000000000000000000000000000000000000000000000000";


    let mut log_message: String = String::new();

    for log_vec in logs.iter() {
        // Is there even a possibility for there to be more than one index in this vector?
        let log: String = match log_vec.get(0) {
            Some(c) => c.clone(),
            None => String::from("")
        }; // Possibly unnecessary cloning, convert to borrowing

        if log == String::from("") {
            return String::from("No logs for this transaction.");
        }

        let num_topics: i8 = if log.len() > 0 {
            log.as_str()[..2].parse::<i8>().unwrap()
        } else {
           0
        };

        let function_hash: &str = if log.len() > 65 {
            &log.as_str()[2..66]
        } else {
            // This must be some sort of erroneous behaviour.
            "No function hash"
        };

        // TODO: Finish this!
        let mut args: Vec<&str> = Vec::new();
        for i in 1..=num_topics {
            let f = 66 + (i as i32 - 1) * 64;
            let t = 66 + i as i32 * 64;

            if log.len() <= t as usize {
                args.push("Missing arg");
                continue;
            }

            args.push(&log.as_str()[f as usize..t as usize]);
        }

        log_message = match function_hash {
            s if s == approval_hash => format!("Event: Approval, Owner: {}, Spender: {}, Amount: {}", args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            s if s == transfer_hash => format!("Event: Transfer, From: {}, To: {}, Amount: {}", &args.get(0).unwrap()[24..], &args.get(1).unwrap()[24..], &args.get(2).unwrap()),
            s if s == log_new_pool_hash => format!("Event: LOG_NEW_POOL, Address 1: {}, Address 2: {}", &args.get(0).unwrap()[24..], &args.get(1).unwrap()[24..]),
            s if s == log_join_pool_hash => format!("Event: LOG_JOIN, Address 1 (Sender?): {}, Address 2 (Pool?): {}, Amount(?): {}", &args.get(0).unwrap()[24..], &args.get(1).unwrap()[24..], args.get(2).unwrap()),
            s if s == log_exit_pool_hash => format!("Event: LOG_EXIT, Address 2 (Sender?): {}, Address 2 (Pool?): {}, Amount(?): {}", &args.get(0).unwrap()[24..], &args.get(1).unwrap()[24..], args.get(2).unwrap()),
            s if s == log_swap_hash => format!("Event: LOG_SWAP, Address 1: {}, Address 2: {}, Address 3: {}, uint256 1 (Swap In?): {}, uint256 2 (Swap Out?): {}", &args.get(0).unwrap()[24..], &args.get(1).unwrap()[24..], &args.get(2).unwrap()[24..], args.get(3).unwrap(), args.get(4).unwrap()),
            s if s == ownership_transferred_hash => format!("Event: OwnershipTransferred, Address 1: {}, Address 2: {}", &args.get(0).unwrap()[24..], &args.get(1).unwrap()[24..]),
            s if s == log_handleSetSwapFee_hash => format!("Event: handleSetSwapFee (Anonymous), bytes4: {}, Address: {}, bytes: {}", args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            s if s == log_handleSetController_hash => format!("Event: handleSetController (Anonymous), bytes4: {}, Address: {}, bytes: {}", args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            s if s == log_handleSetPublicSwap_hash => format!("Event: handleSetPublicSwap (Anonymous), bytes4: {}, Address: {}, bytes: {}", args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            s if s == log_handleFinalize_hash => format!("Event: handleFinalize (Anonymous), bytes4: {}, Address: {}, bytes: {}", args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            s if s == log_handleRebind1_hash => format!("Event: handleRebind (Anonymous), function hash: {} bytes4: {}, Address: {}, bytes: {}", function_hash, args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            s if s == log_handleRebind2_hash => format!("Event: handleRebind (Anonymous), function hash: {} bytes4: {}, Address: {}, bytes: {}", function_hash, args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            s if s == log_handleUnbind_hash => format!("Event: handleUnbind (Anonymous), bytes4: {}, Address: {}, bytes: {}", args.get(0).unwrap(), args.get(1).unwrap(), args.get(2).unwrap()),
            "No function hash" => format!("Event: (Unknown - No function hash), Raw log: {}", log),
            _ => format!("Event: (Unknown) {}, Num. Topics: {}, Args: {:?}", function_hash, num_topics, args)
        };

    }
    log_message
}

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::StreamerMessage>) {
    while let Some(streamer_message) = stream.recv().await {

        // Log EVM Message, if present.
        let mut logs: Vec<Vec<String>> = Vec::new();
        for chunk in streamer_message.chunks.iter() {
            for outcome in chunk.receipt_execution_outcomes.iter() {
                // TODO: Probably unwise to keep the logs as their own vectors
                // necessary to investigate if fields other than log[0] are used
                // in any of the functions we want to track
                logs.push(outcome.execution_outcome.outcome.logs.clone()); // TODO: consider borrowing the value for efficiency reasons later
            }
        }

        let log_message = assign_log(&logs);

        info!(
            target: "indexer_example",
            "#{} {} Chunks: {}, Transactions: {}, Receipts: {}, ExecutionOutcomes: {}, Logs: {:?}",
            streamer_message.block.header.height,
            streamer_message.block.header.hash,
            streamer_message.chunks.len(),
            streamer_message.chunks.iter().map(|chunk| chunk.transactions.len()).sum::<usize>(),
            streamer_message.chunks.iter().map(|chunk| chunk.receipts.len()).sum::<usize>(),
            streamer_message.chunks.iter().map(|chunk| chunk.receipt_execution_outcomes.len()).sum::<usize>(),
            log_message
        );
    }
}

fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();
    init_logging();

    let opts: Opts = Opts::parse();

    let home_dir =
        opts.home_dir.unwrap_or(std::path::PathBuf::from(near_indexer::get_default_home()));

    match opts.subcmd {
        SubCommand::Run => {
            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                sync_mode: near_indexer::SyncModeEnum::FromInterruption,
                await_for_node_synced: near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync,
            };
            actix::System::builder()
                .stop_on_panic(true)
                .run(move || {
                    let indexer = near_indexer::Indexer::new(indexer_config);
                    let stream = indexer.streamer();
                    actix::spawn(listen_blocks(stream));
                })
                .unwrap();
        }
        SubCommand::Init(config) => near_indexer::init_configs(
            &home_dir,
            config.chain_id.as_ref().map(AsRef::as_ref),
            config.account_id.as_ref().map(AsRef::as_ref),
            config.test_seed.as_ref().map(AsRef::as_ref),
            config.num_shards,
            config.fast,
            config.genesis.as_ref().map(AsRef::as_ref),
            config.download,
            config.download_genesis_url.as_ref().map(AsRef::as_ref),
        ),
    }
}

/*
// Example of `block` with all the data
//
// Note that `outcomes` for a given transaction won't be included into the same block.
// Execution outcomes are included into the blocks after the transaction or receipt
// are recorded on a chain; in most cases, it is the next block after the one that has
// the transaction or receipt.
//
// BlockResponse {
//     block: BlockView {
//         author: "test.near",
//         header: BlockHeaderView {
//             height: 426,
//             epoch_id: `11111111111111111111111111111111`,
//             next_epoch_id: `9dH4uF6d3bQtXa7v8CyLPhPXGBUikCCdWB26JWVFRsBY`,
//             hash: `99fpSqxeiMe8iTfh72reaLvx4R16kPYAr1TYuxS3zqkB`,
//             prev_hash: `8try83LRTx76jfbmPv8SxJchgW8XeQH3gt2piKT6Ykpj`,
//             prev_state_root: `DjCJizTpo86umJHv6urA37RJqXkztV1VXpRQysTaKpZA`,
//             chunk_receipts_root: `GcXz5GG5oTYvdYK7jzqUt2gGQtw36gE2Hu2MxXxuBh7`,
//             chunk_headers_root: `GibE7k6ychYbjJHaURHV369WSGR2jjfgCCF3xL4Qgi9Y`,
//             chunk_tx_root: `7tkzFg8RHBmMw1ncRJZCCZAizgq4rwCftTKYLce8RU8t`,
//             outcome_root: `JfGt9hY94ftG2sswDQduL5U3GVb7drdX14oJZvGpuoV`,
//             chunks_included: 1,
//             challenges_root: `11111111111111111111111111111111`,
//             timestamp: 1594306903797198000,
//             timestamp_nanosec: 1594306903797198000,
//             random_value: `EhqGHhUP8W4ULtHF6N7pjB6hSJRE7noUKgmQvy6kfZTZ`,
//             validator_proposals: [],
//             chunk_mask: [
//                 true,
//             ],
//             gas_price: 5000,
//             rent_paid: 0,
//             validator_reward: 0,
//             total_supply: 2049999999999999997877224687500000,
//             challenges_result: [],
//             last_final_block: `2eiUwiZxqo5fRSKPqJ5Nq4oSYRQZE5gRuA9p7TcjFRSJ`,
//             last_ds_final_block: `8try83LRTx76jfbmPv8SxJchgW8XeQH3gt2piKT6Ykpj`,
//             next_bp_hash: `BsoSx2Ea1Vcomv3Ygw95E8ZeNq5QYrZLdcsYdbs3SWpC`,
//             block_merkle_root: `EUmovh7K8yRgboG6vXxCP2dN3ChMLByX846MZG1y6xwG`,
//             approvals: [
//                 Some(
//                     ed25519:42QiF81ZvRx5PfFzZxKgYC3yBfFJ4nSJCSwyiiZoztf34NXx8ottoz9jj3urtuwCHV8u6gJ9GHxUhNqbB1KpTeCH,
//                 ),
//             ],
//             signature: ed25519:27iPMfiR3fh5nC4wmWA3XXjXvA6yffNcnF7PMeMKRnLDpeHytV6GPtzrNNyDsuLVEWjFJvQLfp1kPw8S16zexy2d,
//             latest_protocol_version: 29,
//         },
//         chunks: [
//             ChunkHeaderView {
//                 chunk_hash: `7Qz2B7MumKt68iNFjSwahG35cynVLrkCfXjopFZQFLg4`,
//                 prev_block_hash: `8try83LRTx76jfbmPv8SxJchgW8XeQH3gt2piKT6Ykpj`,
//                 outcome_root: `86VPrDDpopYnn5pXSZWh6LqHjBUui2reaQT3kPYpthUs`,
//                 prev_state_root: `F53dYSv5z9ejgDEt1keCY2JxEeDSTUPeGsEkVg21QbBH`,
//                 encoded_merkle_root: `6s1QWaxhbL7EwzE7ccmhqAAebYBaziXBbumNCEqkKL76`,
//                 encoded_length: 208,
//                 height_created: 426,
//                 height_included: 426,
//                 shard_id: 0,
//                 gas_used: 424555062500,
//                 gas_limit: 1000000000000000,
//                 rent_paid: 0,
//                 validator_reward: 0,
//                 balance_burnt: 2122775312500000,
//                 outgoing_receipts_root: `7LstzSPfxFErjyxZg8nhvcXUxMUpDBysEF53w3uFF5dc`,
//                 tx_root: `11111111111111111111111111111111`,
//                 validator_proposals: [],
//                 signature: ed25519:4P2mYEGHU5L2JW2smLuy92DBRJ1iZmmjFmbHNV6PddCD46UW9Nmb5E285AKcK2XCjishc9NLyByMudursGxCatkf,
//             },
//         ],
//     },
//     chunks: [
//         IndexerChunkView {
//         author: "test.near",
//         header: ChunkHeaderView {
//             chunk_hash: `EF4KJYRKzmtgncQwPkJx3ed2XG3u42e7Dw8G2qLFizYS`,
//             prev_block_hash: `GXkHtzYqzkNN11RJajcGMyW18wsn3Joa2Z8aRoRzy6GK`,
//             outcome_root: `11111111111111111111111111111111`,
//             prev_state_root: `2Aw5LFRbDpcSji61VPeCeT9yX7bSToRmbzVk5vyPhCfp`,
//             encoded_merkle_root: `7SWuaCHcRSMt56B9MwhMtmKUhZLwAkKPUtnShTrvHaqc`,
//             encoded_length: 242,
//             height_created: 1214,
//             height_included: 0,
//             shard_id: 0,
//             gas_used: 0,
//             gas_limit: 1000000000000000,
//             rent_paid: 0,
//             validator_reward: 0,
//             balance_burnt: 0,
//             outgoing_receipts_root: `H4Rd6SGeEBTbxkitsCdzfu9xL9HtZ2eHoPCQXUeZ6bW4`,
//             tx_root: `E1CbQD6XaRPG5ji7K6z4eAhdjAEpTonUo6jK3FN1Bjk`,
//             validator_proposals: [],
//             signature: ed25519:4oRiWAB9hmSUKpn6ADNpckGXiPYF5SYvHeSWDEqbQdzDWEqoRLRzRQCDXwfRrnLcqodvR97CkoDUrfwmrMJydcpu,
//         },
//         transactions: [
//             IndexerTransactionWithOutcome {
//             transaction: SignedTransactionView {
//                 signer_id: "test.near",
//                 public_key: ed25519:8NA7mh6TAWzy2qz68bHp62QHTEQ6nJLfiYeKDRwEbU3X,
//                 nonce: 1,
//                 receiver_id: "some.test.near",
//                 actions: [
//                     CreateAccount,
//                 Transfer {
//                     deposit: 40000000000000000000000000,
//                 },
//                 AddKey {
//                     public_key: ed25519:2syGhqwJ8ba2nUGmP9tkZn9m1DYZPYYobpufiERVnug8,
//                     access_key: AccessKeyView {
//                         nonce: 0,
//                         permission: FullAccess,
//                     },
//                 },
//                 ],
//                 signature: ed25519:Qniuu7exnr6xbe6gKafV5vDhuwM1jt9Bn7sCTF6cHfPpYWVJ4Q6kq8RAxKSeLoxbCreVp1XzMMJmXt8YcUqmMYw,
//                 hash: `8dNv9S8rAFwso9fLwfDQXmw5yv5zscDjQpta96pMF6Bi`,
//             },
//             outcome: IndexerExecutionOutcomeWithReceipt {
//                 execution_outcome: ExecutionOutcomeWithIdView {
//                     proof: [],
//                     block_hash: `G9v6Fsv94xaa7BRY2N5PFF5PJwT7ec6DPzQK73Yf3CZ6`,
//                     id: `8dNv9S8rAFwso9fLwfDQXmw5yv5zscDjQpta96pMF6Bi`,
//                     outcome: ExecutionOutcomeView {
//                         logs: [],
//                         receipt_ids: [
//                         `CbWu7WYYbYbn3kThs5gcxANrxy7AKLcMcBLxLw8Zq1Fz`,
//                         ],
//                         gas_burnt: 424555062500,
//                         tokens_burnt: 424555062500000000000,
//                         executor_id: "test.near",
//                         status: SuccessReceiptId(CbWu7WYYbYbn3kThs5gcxANrxy7AKLcMcBLxLw8Zq1Fz),
//                     },
//                 },
//                 receipt: None,
//             },
//         },
//         ],
//         receipts: [
//             ReceiptView {
//             predecessor_id: "test.near",
//             receiver_id: "some.test.near",
//             receipt_id: `CbWu7WYYbYbn3kThs5gcxANrxy7AKLcMcBLxLw8Zq1Fz`,
//             receipt: Action {
//                 signer_id: "test.near",
//                 signer_public_key: ed25519:8NA7mh6TAWzy2qz68bHp62QHTEQ6nJLfiYeKDRwEbU3X,
//                 gas_price: 1030000000,
//                 output_data_receivers: [],
//                 input_data_ids: [],
//                 actions: [
//                     CreateAccount,
//                 Transfer {
//                     deposit: 40000000000000000000000000,
//                 },
//                 AddKey {
//                     public_key: ed25519:2syGhqwJ8ba2nUGmP9tkZn9m1DYZPYYobpufiERVnug8,
//                     access_key: AccessKeyView {
//                         nonce: 0,
//                         permission: FullAccess,
//                     },
//                 },
//                 ],
//             },
//         },
//         ],
//         receipt_execution_outcomes: [
//             IndexerExecutionOutcomeWithReceipt {
//             execution_outcome: ExecutionOutcomeWithIdView {
//                 proof: [],
//                 block_hash: `BXPB6DQGmBrjARvcgYwS8qKLkyto6dk9NfawGSmfjE9Q`,
//                 id: `CbWu7WYYbYbn3kThs5gcxANrxy7AKLcMcBLxLw8Zq1Fz`,
//                 outcome: ExecutionOutcomeView {
//                     logs: [],
//                     receipt_ids: [
//                     `8vJ1QWM4pffRDnW3c5CxFFV5cMx8wiqxsAqmZTitHvfh`,
//                     ],
//                     gas_burnt: 424555062500,
//                     tokens_burnt: 424555062500000000000,
//                     executor_id: "some.test.near",
//                     status: SuccessValue(``),
//                 },
//             },
//             receipt: Some(
//                 ReceiptView {
//                 predecessor_id: "test.near",
//                 receiver_id: "some.test.near",
//                 receipt_id: `CbWu7WYYbYbn3kThs5gcxANrxy7AKLcMcBLxLw8Zq1Fz`,
//                 receipt: Action {
//                     signer_id: "test.near",
//                     signer_public_key: ed25519:8NA7mh6TAWzy2qz68bHp62QHTEQ6nJLfiYeKDRwEbU3X,
//                     gas_price: 1030000000,
//                     output_data_receivers: [],
//                     input_data_ids: [],
//                     actions: [
//                         CreateAccount,
//                     Transfer {
//                         deposit: 40000000000000000000000000,
//                     },
//                     AddKey {
//                         public_key: ed25519:2syGhqwJ8ba2nUGmP9tkZn9m1DYZPYYobpufiERVnug8,
//                         access_key: AccessKeyView {
//                             nonce: 0,
//                             permission: FullAccess,
//                         },
//                     },
//                     ],
//                 },
//             },
//             ),
//         },
//         ],
//     ],
//     state_changes: [
//         StateChangeWithCauseView {
//             cause: ValidatorAccountsUpdate,
//             value: AccountUpdate {
//                 account_id: "test.near",
//                 account: AccountView {
//                     amount: 1000000000000000000000000000000000,
//                     locked: 50000000000000000000000000000000,
//                     code_hash: `11111111111111111111111111111111`,
//                     storage_usage: 182,
//                     storage_paid_at: 0,
//                 },
//             },
//         },
//     ]
// }
*/

