use emulator::{memory::Memory, Cpu, RunResult, LAST_PRESSED_BUTTON_ADDRESS};
use leptos::{
    component, create_effect, create_node_ref, create_signal, ev::KeyboardEvent, html,
    leptos_dom::logging::console_warn, view, IntoView, Signal, SignalGet, SignalGetUntracked,
    SignalSet, SignalUpdate, SignalWith, SignalWithUntracked,
};
use leptos_use::use_raf_fn;
use rand::Rng;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

const CANVAS_MESSAGE: &'static str = "Could not acquire canvas 2d context";

#[derive(Default, Copy, Clone, PartialEq)]
enum GameState {
    #[default]
    Paused,
    Running,
}

type Screen = [(u8, u8, u8); 32 * 32];

#[component]
pub fn App() -> impl IntoView {
    // Game state
    let (game_state, set_game_state) = create_signal(GameState::default());
    let (cpu, set_cpu) = create_signal({
        let mut cpu = emulator::Cpu::default();
        cpu.load(emulator::SNAKE.to_vec());
        cpu.reset();
        cpu
    });
    let (screen, set_screen) = create_signal::<Screen>([(0, 0, 0); 32 * 32]);
    let running = move || matches!(game_state.get(), GameState::Running);
    let paused = move || matches!(game_state.get(), GameState::Paused);

    // Canvas
    let canvas_ref = create_node_ref::<html::Canvas>();
    let canvas_ctx = Signal::derive(move || {
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
    });

    let (operations, set_operations) = create_signal(Vec::<(usize, (u8, u8, u8))>::new());

    let run_next_cycle = move || {
        set_cpu.update(|cpu| cpu.mem_write(0xfe, rand::thread_rng().gen_range(1..16)));

        cpu.with(|cpu| {
            screen.with(|screen| {
                let operations = read_screen_state(cpu, screen);
                if !operations.is_empty() {
                    set_operations.set(operations);
                }
            });
        });

        set_cpu.update(|cpu| match cpu.run_single_cycle() {
            RunResult::Running => {}
            RunResult::Done => set_game_state.set(GameState::Paused),
        });
    };

    create_effect(move |_| {
        let operations = operations.get();
        if !operations.is_empty() {
            set_screen.update(|screen| {
                for (index, color) in operations {
                    screen[index] = color;
                    let canvas_ctx = canvas_ctx.get_untracked().unwrap();
                    canvas_ctx.set_fill_style(&JsValue::from_str(&format!(
                        "#{:X?}{:X?}{:X?}",
                        color.0, color.1, color.2
                    )));

                    canvas_ctx.fill_rect(
                        index as f64 % 32.0,
                        (index as f64 / 32.0).floor(),
                        1.0,
                        1.0,
                    );
                }
            });
        }
    });

    // create_effect(move |_| screen);

    let game_loop = use_raf_fn(move |_| run_next_cycle());
    (game_loop.pause)();

    create_effect(move |_| {
        match game_state.get() {
            GameState::Paused => (game_loop.pause)(),
            GameState::Running => (game_loop.resume)(),
        };
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
        set_cpu.update(|cpu| cpu.mem_write(LAST_PRESSED_BUTTON_ADDRESS.into(), keycode));
        log::debug!(
            "Last pressed button: 0x{:X?}",
            cpu.with(|cpu| cpu.mem_read(LAST_PRESSED_BUTTON_ADDRESS.into()))
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

fn read_screen_state(cpu: &Cpu, screen: &Screen) -> Vec<(usize, (u8, u8, u8))> {
    // Our display is located between these addresses in memory
    (0x0200..0x0600)
        .into_iter()
        .enumerate()
        .filter_map(|(frame_index, memory_address)| {
            let color_idx = cpu.mem_read(memory_address as u16);
            // aX is the color currently displayed in the canvas
            // bX is the color that was set in memory and that should now be displayed
            let (a1, a2, a3) = screen[frame_index];
            let (b1, b2, b3) = color(color_idx);

            console_warn(&format!(
                "Comparing #{:X?}{:X?}{:X?} with #{:X?}{:X?}{:X?} ",
                a1, a2, a3, b1, b2, b3
            ));
            if a1 != b1 || a2 != b2 || a3 != b3 {
                return Some((frame_index, (b1, b2, b3)));
            }

            None
        })
        .collect()
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
