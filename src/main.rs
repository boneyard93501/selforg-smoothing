use fluence_keypair::key_pair::KeyPair;
use std::collections::BTreeMap;
use chrono::{Utc, TimeZone};


 fn get_peer_slot_distro(n_peers: usize) {
    // let n_peers:usize = 1_000;
    let mut peer_ids = Vec::<[u8;8]>::new();

    for p in 0..n_peers {
        let kp = KeyPair::generate_ed25519();
        let kp:[u8;8] = kp.get_peer_id().to_bytes()[30..].try_into().unwrap();
        peer_ids.push(kp);
    }
    
    let mut dist_map = BTreeMap::<u64, u32>::new();
    for pid in peer_ids {
        // let v = u64::from_le_bytes(pid[30..].try_into().unwrap());
        let v = u64::from_le_bytes(pid);
        dist_map.entry(v%24).and_modify(|counter| *counter += 1).or_insert(1);
    }

    println!("{:?}", dist_map);
}


const SECS_DAY:u32 = 86_400;
const SECS_HOUR: u32 = 3_600;

// ignoring peerid to slot conversion
fn check_slot_eligibility(slot: u32, ts: i64) -> bool {
    let secs_today_elapsed:i64 = ts % SECS_DAY as i64;
    let secs_slot_elapsed: i64 = slot as i64 * SECS_HOUR as i64;
    
    // println!("slot_elapsed: {}, today_elapsed: {}", secs_slot_elapsed, secs_today_elapsed);
    if secs_today_elapsed >= secs_slot_elapsed  && secs_today_elapsed < secs_slot_elapsed + SECS_HOUR as i64 {
        return true;
    }
    false
}


fn main() {

    let ts = Utc::now().timestamp();
    println!("dt: {:?}", Utc.timestamp_opt(ts, 0));
    let mut true_count = 0;
    for i in 0..=23 {
        let res = check_slot_eligibility(i as u32, ts);
        if res {
            true_count += 1;
        }
        println!("{}, {}", i, res);
    }
    assert_eq!(true_count, 1);
}




