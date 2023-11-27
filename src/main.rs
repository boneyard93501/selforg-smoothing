use asciigraph::*;
use chrono::{TimeZone, Utc};
use fluence_keypair::key_pair::KeyPair;
use rv::misc::x2_test;
use std::collections::BTreeMap;


const SECS_DAY: u32 = 86_400;
const SECS_HOUR: u32 = 3_600;

fn make_hg(freqs: &Vec<u32>) -> Graph {
    let total_freqs:u32 = freqs.iter().sum();
    let mut hg = Graph::default();
    hg.set_1d_data(freqs)
        .set_y_min(36)
        .set_y_max(45)
        .set_plot_height(10)
        .set_block_width(2)
        .set_y_label_margin(1)
        // .set_title("Slot From PeerId Frequencies for {freqs.len()} peers")
        .set_title(&format!("Frequencies -- {} Peers",total_freqs ))
        .set_paddings([1; 4])
        .set_big_title(true)
        .set_x_axis_label("buckets")
        .set_y_axis_label("frequencies");
    hg
}

fn chi_square(freqs: Vec<u32>) -> f64 {
    let freq_sum: u32 = freqs.iter().sum();
    let expected_freq = (1 as f64 / freqs.len() as f64) * freq_sum as f64;

    let chi_square: f64 = freqs
        .iter()
        .map(|&i| (i as f64 - expected_freq).powf(2.0) / expected_freq)
        .sum();

    chi_square
}

fn get_peer_slot_distro(n_peers: usize) {
    // let n_peers:usize = 1_000;
    let mut peer_ids = Vec::<[u8; 8]>::new();

    for p in 0..n_peers {
        let kp = KeyPair::generate_ed25519();
        let kp: [u8; 8] = kp.get_peer_id().to_bytes()[30..].try_into().unwrap();
        peer_ids.push(kp);
    }

    let mut dist_map = BTreeMap::<u64, u32>::new();
    for pid in peer_ids {
        // let v = u64::from_le_bytes(pid[30..].try_into().unwrap());
        let v = u64::from_le_bytes(pid);
        dist_map
            .entry(v % 24)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    // println!("{:?}", dist_map);
    let freqs: Vec<u32> = dist_map.values().cloned().collect();
    let freqs: [u32; 24] = freqs.try_into().unwrap();
    // println!("freqs {:?}", freqs);
    let (stat, p) = x2_test(&freqs, &vec![1 as f64 / 24 as f64; 24]);
    println!("stat: {}, p: {}", stat, p);
    let hg = make_hg(&freqs.to_vec());
    println!("histogram:\n{}", hg);
}

// ignoring peerid to slot conversion
fn check_slot_eligibility(slot: u32, ts: i64) -> bool {
    let secs_today_elapsed: i64 = ts % SECS_DAY as i64;
    let secs_slot_elapsed: i64 = slot as i64 * SECS_HOUR as i64;

    // println!("slot_elapsed: {}, today_elapsed: {}", secs_slot_elapsed, secs_today_elapsed);
    if secs_today_elapsed >= secs_slot_elapsed
        && secs_today_elapsed < secs_slot_elapsed + SECS_HOUR as i64
    {
        return true;
    }
    false
}


fn check_submission_eligibilty_for_now() {

    let ts = Utc::now().timestamp();
    // println!("dt: {:?}", Utc.timestamp_opt(ts, 0));
    let mut true_count = 0;
    for i in 0..=23 {
        let res = check_slot_eligibility(i as u32, ts);
        if res {
            true_count += 1;
        }
        // println!("{}, {}", i, res);
    }
    assert_eq!(true_count, 1);
}

fn main() {
    println!("distribution for 1,000 peer ids:");
    get_peer_slot_distro(1_000);
    println!("distribution for 1,000,000 peer ids:");
    get_peer_slot_distro(1_000_000);

    check_submission_eligibilty_for_now();
    
}

