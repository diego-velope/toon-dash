// src/game/persistence.rs
//! Lifetime stats persistence for Toon Dash

#[cfg(target_arch = "wasm32")]
mod wasm {
    extern "C" {
        pub fn mq_save_stats(high_score: i32, total_coins: i32, total_stars: i32, total_distance: i32);
        pub fn mq_load_stats_high_score() -> i32;
        pub fn mq_load_stats_total_coins() -> i32;
        pub fn mq_load_stats_total_stars() -> i32;
        pub fn mq_load_stats_total_distance() -> i32;
        pub fn mq_load_highscore() -> i32;
    }
}

use super::state::LifetimeStats;

pub fn save_stats(stats: LifetimeStats) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm::mq_save_stats(
            stats.high_score as i32,
            stats.total_coins as i32,
            stats.total_stars as i32,
            stats.total_distance as i32,
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        if let Ok(mut file) = std::fs::File::create("stats.dat") {
            let encoded = format!(
                "{},{},{},{}",
                stats.high_score, stats.total_coins, stats.total_stars, stats.total_distance
            );
            let _ = file.write_all(encoded.as_bytes());
        }
    }
}

pub fn load_stats() -> LifetimeStats {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let high_score = wasm::mq_load_stats_high_score();
        let total_coins = wasm::mq_load_stats_total_coins();
        let total_stars = wasm::mq_load_stats_total_stars();
        let total_distance = wasm::mq_load_stats_total_distance();

        if high_score == 0 && total_coins == 0 && total_stars == 0 && total_distance == 0 {
            let legacy_high = wasm::mq_load_highscore();
            return LifetimeStats {
                high_score: legacy_high.max(0) as u32,
                total_coins: 0,
                total_stars: 0,
                total_distance: 0,
            };
        }

        return LifetimeStats {
            high_score: high_score.max(0) as u32,
            total_coins: total_coins.max(0) as u32,
            total_stars: total_stars.max(0) as u32,
            total_distance: total_distance.max(0) as u32,
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Read;
        if let Ok(mut file) = std::fs::File::open("stats.dat") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                let parts: Vec<&str> = contents.trim().split(',').collect();
                if parts.len() == 4 {
                    return LifetimeStats {
                        high_score: parts[0].parse().unwrap_or(0),
                        total_coins: parts[1].parse().unwrap_or(0),
                        total_stars: parts[2].parse().unwrap_or(0),
                        total_distance: parts[3].parse().unwrap_or(0),
                    };
                }
            }
        }

        if let Ok(mut file) = std::fs::File::open("highscore.dat") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return LifetimeStats {
                    high_score: contents.trim().parse().unwrap_or(0),
                    total_coins: 0,
                    total_stars: 0,
                    total_distance: 0,
                };
            }
        }

        LifetimeStats::default()
    }
}
