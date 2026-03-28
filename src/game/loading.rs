// src/game/loading.rs
//! Progress bridge for the WASM splash screen

#[cfg(target_arch = "wasm32")]
mod wasm {
    extern "C" {
        pub fn mq_set_progress(percent: f32);
        pub fn mq_hide_splash();
    }
}

pub fn set_progress(percent: f32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm::mq_set_progress(percent);
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        let bar = (0..(percent as i32 / 5)).map(|_| "=").collect::<String>();
        println!("[Loading] [{:<20}] {}%", bar, percent as i32);
    }
}

pub fn hide_splash() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm::mq_hide_splash();
    }
}
