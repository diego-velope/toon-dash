// src/game/persistence.rs
//! High-score persistence for Toon Dash

#[cfg(target_arch = "wasm32")]
mod wasm {
    extern "C" {
        pub fn mq_save_highscore(score: i32);
        pub fn mq_load_highscore() -> i32;
    }
}

pub fn save_highscore(score: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm::mq_save_highscore(score as i32);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        if let Ok(mut file) = std::fs::File::create("highscore.dat") {
            let _ = file.write_all(score.to_string().as_bytes());
        }
    }
}

pub fn load_highscore() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        return wasm::mq_load_highscore() as u32;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Read;
        if let Ok(mut file) = std::fs::File::open("highscore.dat") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return contents.trim().parse().unwrap_or(0);
            }
        }
        0
    }
}
