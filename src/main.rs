use anyhow::{anyhow, Context, Result};
use clap::Parser;
use chrono::{DateTime};
use colorized::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::{Value};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::clock::Slot;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;
use tokio::time::{sleep, Duration};

#[derive(Parser, Debug)]
struct Args {
    /// The validator identity public key
    #[clap(short, long)]
    validator: String,
    // The epoch provided to check on leader schedule for, default: current epoch
    #[clap(short, long)]
    epoch: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let validator_pubkey_str = &args.validator;

    // Validate solana pubkey
    let _ = Pubkey::from_str(validator_pubkey_str)
        .map_err(|_| anyhow!("Invalid validator pubkey: {}", validator_pubkey_str))?;

    let rpc_url = "https://api.mainnet-beta.solana.com";
    let rpc = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::finalized());

    // Get current epoch info
    let epoch_info = rpc
        .get_epoch_info()
        .await
        .context("Failed to get epoch info")?;
    let current_epoch = epoch_info.epoch;

    // Determine the epoch to query
    let epoch = args.epoch.unwrap_or(current_epoch);

    // Get the current slot
    let current_slot = rpc.get_slot().await.context("Failed to get current slot")?;

    // Calculate the first absolute slot of the current epoch
    let epoch_first_slot = if epoch == current_epoch {
        println!("Using current epoch: {}", current_epoch);
        // For the current epoch, calculate based on `epoch_info`
        epoch_info.absolute_slot - epoch_info.slot_index
    } else {
        println!("Using configured epoch: {}", epoch);
        // For past or future epochs, calculate from epoch info
        let slots_per_epoch = epoch_info.slots_in_epoch;
        let slots_between_epochs = (epoch as i64 - current_epoch as i64) * slots_per_epoch as i64;
        (epoch_info.absolute_slot as i64 - epoch_info.slot_index as i64 + slots_between_epochs)
            .max(0) as u64 // Ensure non-negative
    };

    // Retrieve the leader schedule for the current epoch
    let leader_schedule_raw = rpc
        .get_leader_schedule(Some(epoch_first_slot))
        .await
        .context("Failed to get leader schedule")?;
    let leader_schedule_raw = leader_schedule_raw.ok_or_else(|| anyhow!("No leader schedule returned for epoch {}", epoch))?;

    // Convert Vec<usize> to Vec<u64> if necessary
    let leader_schedule: HashMap<String, Vec<u64>> = leader_schedule_raw
        .into_iter()
        .map(|(k, v)| {
            let converted: Vec<u64> = v.into_iter().map(|x| x as u64).collect();
            (k, converted)
        })
        .collect();

    // Check if our validator is in the leader schedule
    let our_slots = match leader_schedule.get(validator_pubkey_str) {
        Some(slots) => slots,
        None => {
            println!("Validator {} is not scheduled to lead in the current epoch.", validator_pubkey_str);
            return Ok(());
        }
    };

    println!("Validator {} is assigned to {} slots in epoch {}.",
             validator_pubkey_str, our_slots.len(), epoch);

    // Build a slot-to-validator mapping with absolute slots using a BTreeMap
    let mut slot_validator_map = BTreeMap::new();
    for (validator, slots) in &leader_schedule {
        for slot in slots {
            let absolute_slot = epoch_first_slot + *slot;
            slot_validator_map.insert(absolute_slot, validator.clone());
        }
    }

    // Convert our slots to absolute slots
    let mut our_absolute_slots: Vec<u64> = our_slots
        .iter()
        .map(|slot| epoch_first_slot + *slot)
        .collect();

    // Sort our slots
    our_absolute_slots.sort_unstable();

    // Group slots into blocks of consecutive slots up to 4 slots
    let mut blocks = Vec::new();
    let mut block = Vec::new();
    for (i, &slot) in our_absolute_slots.iter().enumerate() {
        block.push(slot);
        if i + 1 == our_absolute_slots.len()
            || our_absolute_slots[i + 1] != slot + 1
            || block.len() == 4
        {
            blocks.push(block.clone());
            block.clear();
        }
    }

    // Time estimation setup
    let current_unix_time = {
        let system_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        system_time.as_secs() as i64
    };

    // Average slot duration (rough estimate) in seconds
    let average_slot_duration = 0.4;
    println!("Using average slot duration: {:.3} seconds", average_slot_duration);

    // Count total slots to process for progress bar
    let total_slots: usize = blocks.iter().map(|b| b.len()).sum();

    // Create a progress bar
    let pb = ProgressBar::new(total_slots as u64);
    pb.set_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
    ).unwrap());

    for block in blocks {
        // We'll track if we printed anything about this block
        let mut non_produced_slots = Vec::new();

        // First pass: check which slots are not produced by us
        for &slot in &block {
            // Skip future slots
            if slot > current_slot {
                continue;
            }
            let produced = is_slot_produced(&rpc, slot).await?;
            let mut non_produced = false;

            if produced {
                let leaders = rpc.get_slot_leaders(slot, 1).await?;
                if let Some(final_leader) = leaders.get(0) {
                    let final_leader_str = final_leader.to_string();
                    if final_leader_str != *validator_pubkey_str {
                        // Slot produced by someone else
                        non_produced_slots.push((slot, Some(final_leader_str)));
                        non_produced = true;
                    }
                } else {
                    // No leader info
                    non_produced_slots.push((slot, None));
                    non_produced = true;
                }
            } else {
                // Slot skipped
                non_produced_slots.push((slot, None));
                non_produced = true;
            }

            // Update the progress bar for every slot checked
            pb.inc(1);

            // small delay
            if non_produced {
                sleep(Duration::from_millis(20)).await;
            }
        }

        // Only print block and leader info if we have non-produced slots
        if !non_produced_slots.is_empty() {
            let first_slot = block.first().unwrap();
            let last_slot = block.last().unwrap();

            // Estimate time for first slot
            let slot_offset = *first_slot as i64 - current_slot as i64;
            let estimated_time = current_unix_time as f64 + (average_slot_duration * slot_offset as f64);
            let datetime = DateTime::from_timestamp(
                estimated_time.floor() as i64,
                ((estimated_time.fract()) * 1e9) as u32,
            );
            let datetime_str = 
                datetime.expect("Broken Time").format("%Y-%m-%d %H:%M:%S%.3f").to_string();

            let previous_slot = first_slot.saturating_sub(1);
            let next_slot = last_slot + 1;
            let previous_validator = slot_validator_map.get(&previous_slot);
            let next_validator = slot_validator_map.get(&next_slot);

            println!("----------------------------------------");
            println!("Block of slots: {:?} at approximately {} UTC", block, datetime_str);

            if let Some(prev_val) = previous_validator {
                // Fetch skip blame data
                let response = reqwest::get("https://api.trillium.so/skip_blame/")
                    .await
                    .context("Failed to fetch skip blame data")?
                    .text()
                    .await
                    .context("Failed to read skip blame response text")?;
                let json: Value = serde_json::from_str(&response)
                    .context("Failed to parse skip blame response JSON")?;
        
                // Parse bad validators list
                if let Some(bad_validators) = json["data"]["validators"].as_array() {
                    // Check if the previous validator is in the bad list
                    let is_bad_validator = bad_validators.iter().any(|bad_validator| {
                        bad_validator["identity_pubkey"]
                            .as_str()
                            .map_or(false, |bad_pubkey| bad_pubkey == prev_val)
                    });
        
                    if is_bad_validator {
                        let url = "https://api.vx.tools/epochs/leaderboard/voting";
                        let client = Client::new();
                        let payload = serde_json::json!({});
                        // Send POST to vx tools for validator latency stats calculation later
                        let response = client.post(url).header("Content-Type", "application/json")
                            .json(&payload)
                            .send()
                            .await?
                            .text()
                            .await
                            .context("Failed to read latency response text")?;
                        // Make sure response is in JSON format
                        let json: Value = serde_json::from_str(&response)
                            .context("Failed to parse latency response JSON")?;
                        // Access the records array response
                        if let Some(records) = json["records"].as_array() {
                            // Find the bad nodes latency
                            if let Some((index, record)) = records.iter().enumerate().find(|(_, r)| r["nodeAddress"].as_str() == Some(&*prev_val)) {
                                let total_latency = record["totalLatency"].as_u64();
                                let voted_slots = record["votedSlots"].as_u64();
                                if let (Some(total_latency), Some(voted_slots)) = (
                                    total_latency,
                                    voted_slots,
                                ) {
                                    // Calculate rank
                                    let rank = index + 1;
                                    // Calculate average latency
                                    let avg_latency = total_latency as f64 / voted_slots as f64;
                                    println!(
                                        "Previous Slot {} Leader: {} {} (Latency: {:.6}, Rank: {})",
                                        previous_slot,
                                        prev_val,
                                        "##ON BAD SKIP LIST##".color(Colors::BrightRedFg),
                                        avg_latency,
                                        rank
                                    );
                                }
                            }
                        }
                    } else {
                        println!("Previous Slot {} Leader: {}", previous_slot, prev_val);
                    }
                } else {
                    eprintln!("Warning: 'validators' array missing or JSON response changed.");
                }
            } else {
                println!("Previous Slot {} Leader: Unknown or No Leader", previous_slot);
            }

            println!("Our Validator Slots {} - {}: {}", first_slot, last_slot, validator_pubkey_str);

            if let Some(next_val) = next_validator {
                println!("Next Slot {} Leader: {}", next_slot, next_val);
            } else {
                println!("Next Slot {} Leader: Unknown or No Leader", next_slot);
            }

            // Print the details of non-produced slots
            for (slot, maybe_leader) in non_produced_slots {
                match maybe_leader {
                    Some(leader_str) => {
                        println!("Slot {}: block produced by {}, not us!", slot, leader_str);
                    }
                    None => {
                        println!("Slot {}: no block produced (skipped?) or no leader info. Not produced by us.", slot);
                    }
                }
            }
        }
    }

    pb.finish_with_message("Done checking slots!");

    Ok(())
}

/// Check if a slot was produced by using `get_blocks`.
/// If `get_blocks(slot, Some(slot))` returns a non-empty vector, a block exists at that slot.
async fn is_slot_produced(rpc: &RpcClient, slot: Slot) -> Result<bool> {
    let blocks = rpc.get_blocks(slot, Some(slot)).await?;
    Ok(!blocks.is_empty())
}

/// Fetches all produced slots in the given range using `get_blocks`.
/// Returns a HashSet of slots that have produced blocks.
async fn fetch_produced_slots(rpc: &RpcClient, start_slot: Slot, end_slot: Slot) -> Result<HashSet<Slot>> {
    let confirmed_blocks = rpc.get_blocks(start_slot, Some(end_slot)).await?;
    Ok(confirmed_blocks.into_iter().collect())
}