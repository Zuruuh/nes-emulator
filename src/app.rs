use std::{
    sync::{Arc, RwLock, RwLockWriteGuard},
    time::Duration,
};

use emulator::{memory::Memory, Cpu, RunResult, LAST_PRESSED_BUTTON_ADDRESS};
use leptos::{
    component, create_effect, create_node_ref, html, set_interval, set_interval_with_handle, view,
    IntoView,
};
use once_cell::sync::OnceCell;
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

static CPU: OnceCell<Arc<RwLock<Cpu>>> = OnceCell::new();
static SCREEN: OnceCell<Arc<RwLock<[u8; 32 * 3 * 32]>>> = OnceCell::new();

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();

    CPU.set({
        let mut cpu = emulator::Cpu::default();
        cpu.load(emulator::SNAKE.to_vec());
        cpu.reset();
        Arc::new(RwLock::new(cpu))
    })
    .unwrap();

    SCREEN
        .set(Arc::new(RwLock::new([0 as u8; 32 * 3 * 32])))
        .unwrap();

    create_effect(move |_| {
        let canvas = match canvas_ref.get() {
            Some(canvas) => canvas,
            _ => {
                return;
            }
        };

        let ctx = canvas
            .get_context("2d")
            .expect("Could not acquire canvas 2d context")
            .unwrap();

        let ctx = ctx.dyn_into::<CanvasRenderingContext2d>().unwrap();
        for i in 0..32u8 {
            for j in 0..32u8 {
                let color = format!("#00{:x?}{:x?}", i, j);
                // log::debug!("Priting color {color} at {i}:{j}");
                ctx.set_fill_style(&JsValue::from_str(&color));
                ctx.fill_rect(i as f64 * 10.0, j as f64 * 10.0, 10.0, 10.0);
            }
        }
        ctx.fill_rect(1.0, 1.0, 1.0, 1.0);
    });

    let _interval = set_interval_with_handle(
        move || {
            let cpu_lock = CPU.get().unwrap().clone();
            let mut cpu = cpu_lock.write().unwrap();

            let screen_lock = SCREEN.get().unwrap().clone();
            let mut screen = screen_lock.write().unwrap();

            cpu.mem_write(0xfe, rand::thread_rng().gen_range(1..16));

            if read_screen_state(&cpu, &mut screen) {
                log::debug!("{:?}", screen);
            }

            match cpu.run() {
                RunResult::Done => panic!("DONE RUNNING"),
                _ => {}
            }
        },
        Duration::from_nanos(70_000),
    );

    let on_keypress = move |e: leptos::ev::KeyboardEvent| {
        let keycode: u8 = match e.key().to_lowercase().as_str() {
            "w" => 0x77,
            "s" => 0x73,
            "a" => 0x61,
            "d" => 0x64,
            _ => return,
        };
        e.prevent_default();
        let cpu_lock = CPU.get().unwrap().clone();
        let mut cpu = cpu_lock.write().unwrap();

        cpu.mem_write(LAST_PRESSED_BUTTON_ADDRESS.into(), keycode);
        log::debug!(
            "Last pressed button: 0x{:X?}",
            cpu.mem_read(LAST_PRESSED_BUTTON_ADDRESS.into())
        );
    };

    view! {
        <main id="container">
            <canvas autofocus _ref={canvas_ref} id="screen" on:keypress={on_keypress} tabindex="0" />
        </main>
    }
}

fn read_screen_state(cpu: &Cpu, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.mem_read(i as u16);
        let (b1, b2, b3) = color(color_idx);
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}

fn color(byte: u8) -> (u8, u8, u8) {
    match byte {
        0 => (0, 0, 0),
        1 => (255, 255, 255),
        2 | 9 => (92, 92, 92),
        3 | 10 => (255, 00, 00),
        4 | 11 => (0, 255, 0),
        5 | 12 => (0, 0, 255),
        6 | 13 => (255, 0, 255),
        7 | 14 => (255, 255, 0),
        _ => (0, 255, 255),
    }
}
