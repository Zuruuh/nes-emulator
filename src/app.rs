use std::{
    sync::{Arc, OnceLock, RwLock},
    time::Duration,
};

use emulator::{memory::Memory, Cpu, RunResult, LAST_PRESSED_BUTTON_ADDRESS};
use leptos::{
    component, create_effect, create_node_ref, create_signal, ev::KeyboardEvent, html,
    leptos_dom::helpers::IntervalHandle, set_interval_with_handle, view, IntoView, SignalGet,
    SignalSet,
};
use rand::Rng;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

static CPU: OnceLock<Arc<RwLock<Cpu>>> = OnceLock::new();
static SCREEN: OnceLock<Arc<RwLock<[u8; 32 * 3 * 32]>>> = OnceLock::new();
const CANVAS_MESSAGE: &'static str = "Could not acquire canvas 2d context";

#[derive(Default, Copy, Clone, PartialEq)]
enum GameState {
    #[default]
    Paused,
    Running,
}

#[component]
pub fn App() -> impl IntoView {
    // Game state
    let (game_state, set_game_state) = create_signal(GameState::default());
    let running = move || matches!(game_state.get(), GameState::Running);
    let paused = move || matches!(game_state.get(), GameState::Paused);

    // Canvas
    let canvas_ref = create_node_ref::<html::Canvas>();
    let canvas_ctx = move || {
        let canvas = match canvas_ref.get() {
            Some(canvas) => canvas,
            _ => {
                return None;
            }
        };

        let ctx = canvas
            .get_context("2d")
            .expect(&CANVAS_MESSAGE)
            .expect(&CANVAS_MESSAGE)
            .dyn_into::<CanvasRenderingContext2d>()
            .expect(&CANVAS_MESSAGE);

        Some(ctx)
    };

    initialize_global_state();

    let (game_loop, set_game_loop) = create_signal::<Option<IntervalHandle>>(None);

    let run_next_cycle = move || {
        let cpu_lock = CPU.get().unwrap().clone();
        let mut cpu = cpu_lock.write().unwrap();

        let screen_lock = SCREEN.get().unwrap().clone();
        let mut screen = screen_lock.write().unwrap();

        cpu.mem_write(0xfe, rand::thread_rng().gen_range(1..16));

        if read_screen_state(&cpu, &mut screen) {
            log::debug!("{:?}", screen);
        }

        match cpu.run_single_cycle() {
            RunResult::Running => {}
            RunResult::Done => set_game_state.set(GameState::Paused),
        }
    };

    create_effect(move |_| match game_state.get() {
        GameState::Paused => {
            if let Some(interval) = game_loop.get() {
                interval.clear();
            }
        }
        GameState::Running => {
            let interval = set_interval_with_handle(run_next_cycle, Duration::from_nanos(70_000))
                .expect("Could not create game loop ?");

            set_game_loop.set(Some(interval));
        }
    });

    let on_keypress = move |e: KeyboardEvent| {
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
            <section id="controls">
                <button disabled={running} on:click={move |_| set_game_state.set(GameState::Running)}>Start</button>
                <button disabled={paused} on:click={move |_| set_game_state.set(GameState::Paused)}>Stop</button>
                <button disabled={running} on:click={move|_| run_next_cycle()}>{"Advance 1 frame"}</button>
            </section>
        </main>
    }
}

// TODO: plug this into the canvas
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

/// Map a NES color id to an rgb sequence
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

fn initialize_global_state() {
    // technically this should never return an error since we mount the component only once but idk
    CPU.set({
        let mut cpu = emulator::Cpu::default();
        cpu.load(emulator::SNAKE.to_vec());
        cpu.reset();
        Arc::new(RwLock::new(cpu))
    })
    .unwrap();

    // Same as above
    SCREEN
        .set(Arc::new(RwLock::new([0 as u8; 32 * 3 * 32])))
        .unwrap();
}
