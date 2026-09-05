//! E24, the unbuilt one: bytes per base row as the base grows. RSS from /proc/self/status
//! before and after seeding, at three sizes two decades apart, with the budget fixed.
fn rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status").ok().and_then(|s| s.lines().find(|l| l.starts_with("VmRSS:")).and_then(|l| l.split_whitespace().nth(1)?.parse().ok())).unwrap_or(0)
}
fn main() {
    let budget: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(2500);
    println!("| accounts | rounds | base rows | RSS before | RSS after | delta KiB | bytes/row |");
    println!("|--:|--:|--:|--:|--:|--:|--:|");
    for (accounts, rounds) in [(10_000i64, 1u32), (100_000, 1), (100_000, 10)] {
        let before = rss_kib();
        let e = nilestream_server::rev_engine::RevEngine::seeded(accounts, rounds, budget, proto_engine::ViewMode::Demand, proto_engine::EvictionPolicy::Lru);
        let after = rss_kib();
        let rows = e.head(); // epochs; each seeded epoch is one conserved pair = 2 rows
        let base_rows = rows * 2;
        println!("| {accounts} | {rounds} | {base_rows} | {before} | {after} | {} | {:.1} |", after.saturating_sub(before), (after.saturating_sub(before) as f64 * 1024.0) / base_rows.max(1) as f64);
        drop(e);
    }
}
