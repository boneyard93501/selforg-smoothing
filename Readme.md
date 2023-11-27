# Distributing End-Of-Epoch Submissions

## Overview/Abstract

In all naive epoch-based systems, reaction times tend to cluster around the end-of-epoch time. That is, for a 24 hour epoch bounding some activity period for multiple actors, it is to be expected that all actors respond within a short period of the end of the epoch.

We propose a self-organizing algorithm to distribute p2p agents' submissions into 24 normally distributed submission hereby greatly reducing the stress on the infrastructure.  We achieve our goal by extracting a specific slot value from the Nox peer id, which is then used to determine the submission window against the on-chain blocktime as well as the on-chain verification of a submission request as all PoC proofs carry the peer id.

## Distribution And Verification Algorithms

### Slot Distribution Algorithm

- Step 1: Calculate submission (time) slot from the last eight bytes of the peer id
    - Step 1a: convert peer id to bytes
    - Step 1b: extract last eight bytes
    - Step 1c: calculate u64 from byte slice by the hour (mod 24) — see Figure 1

Figure 1: Submission Slot From PeerId (24 hour range)

```rust
let kp = KeyPair::generate_ed25519();
let kp:[u8;8] = kp.get_peer_id().to_bytes()[30..].try_into().unwrap();
let slot_hour = u64::from_le_bytes(kp);
```

### Submission Determination Algorithm (Nox)

- Step 2: Provider Submission Monitoring
    - Step 2a: Calibrate submission interval against latest blocktime — see Figure 2

Figure 2: Provider Nox Submission Slot Determination (Rust)

```rust
const SECS_DAY:u32 = 86_400;
const SECS_HOUR:u32 = 3_600;

// ignoring peerid to slot conversion
// ts should come from on-chain block.timestamp from latest block
fn check_slot_eligibility(slot: u32,ts: i64) -> bool {
    let secs_today_elapsed:i64 = ts % SECS_DAY as i64;
    let secs_slot_elapsed: i64 = slot as i64 * SECS_HOUR as i64;

    if secs_today_elapsed >= secs_slot_elapsed  && secs_today_elapsed < secs_slot_elapsed + SECS_HOUR as i64 {
				// eligible to submit
        return true;
    }
    false
}
```

Once a peer has their slot and initial submission time calculated based on the on-chain block time, above may be further simplified by running a cronjob based on delta between block time and local time and the slot-governed submission delta.

### Submission Verification Algorithm (On-Chain)

Proof submission includes a much larger data structure including peer id. From the already available proof data, the on-chain contract then needs to calculate the slot from the peer id and verify its submission eligibility. That is, combine Steps 1 and 2 to implement the following algorithm:

- Step 3: Verify Submission From Peer Id On-Chain
    - Step 3a: Calculate submission slot from peer id, see Step 1, on-chain
    - Step 3b: get on-chain timestamp (`block.timestamp` is in seconds)
    - Step 3c: calculate elapsed time (for day) from timestamp mod *86400 (seconds per day)*
    - Step 3d: calculate time slot from peer slot * 3600
    - Step 3e: calculate submission validity if peer_slot_secs ≤ elapsed_secs + 3600 → yes else no (we make the submission window just about one hour)

Decisions such as penalty of submitting out of order beyond gas and additional proof aggregation for missed submission slots are import considerations but outside the scope of this paper.

## Appendix

### Examining Slot Distribution From PeerId

Since the pseudo-random number generator used to generate the ed25519 key pairs follows a normal distribution, it is expected that the u64 derived from the peer id (via public key) also follows a normal distribution. 

Figure 2: Testing Normal Slot Distribution From 1M PeerIds (24 hour range)

```rust
fn get_sliced_pid_distro() {
    let n_peers:usize = 1_000_000;
    let mut peer_ids = Vec::<[u8;8]>::new();

    for p in 0..n_peers {
        let kp = KeyPair::generate_ed25519();
        let kp:[u8;8] = kp.get_peer_id().to_bytes()[30..].try_into().unwrap();
				let v = u64::from_le_bytes(pid);
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
```

which resulted in a rather even (and possibly normal) distribution over the 24 time buckets:

```
{ 0: 41613,  1: 41480,  2: 41617,  3: 41531,  4: 41686,  5: 41277,  6: 41823, 
  7: 41561,  8: 41424,  9: 41899, 10: 41927, 11: 41628, 12: 41849, 13: 41475, 
 14: 41521, 15: 41478, 16: 42051, 17: 41596, 18: 41445, 19: 41738, 20: 41755,
 21: 42073, 22: 41717, 23: 41836
}
```

Running the experiment with a max peer count of 1,000 PeerIds still leads to a near even distribution by slot with a little higher variance —  as one should expect: