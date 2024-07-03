use emulator::{memory::Memory, Cpu, RunResult, LAST_PRESSED_BUTTON_ADDRESS};
use leptos::{
    component, create_effect, create_node_ref, create_signal, ev::KeyboardEvent, html,
    leptos_dom::logging::console_warn, view, IntoView, Signal, SignalGet, SignalSet, SignalUpdate,
    SignalWith,
};
use leptos_use::use_raf_fn;
use rand::Rng;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{CanvasRenderingContext2d, ImageData};

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
    let (cpu, set_cpu) = create_signal({
        let mut cpu = emulator::Cpu::default();
        cpu.load(emulator::SNAKE.to_vec());
        cpu.reset();
        cpu
    });
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

    let run_next_cycle = move || {
        set_cpu.update(|cpu| cpu.mem_write(0xfe, rand::thread_rng().gen_range(1..16)));

        cpu.with(|cpu| {
            let screen_state = read_screen_state(cpu);
            // console_warn(&format!("{:?}", &screen_state));
            let screen_state = Clamped(&screen_state[..]);

            let image_data =
                ImageData::new_with_u8_clamped_array_and_sh(screen_state, 32, 32).unwrap();

            let canvas_ctx = canvas_ctx.get().unwrap();
            canvas_ctx.scale(10.0, 10.0).unwrap();
            canvas_ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
        });

        set_cpu.update(|cpu| match cpu.run_single_cycle() {
            RunResult::Running => {}
            RunResult::Done => set_game_state.set(GameState::Paused),
        });
    };

    // create_effect(move |_| {
    //     let operations = operations.get();
    //     if !operations.is_empty() {
    //         set_screen.update(|screen| {
    //             for (index, color) in operations {
    //                 screen[index] = color;
    //                 let canvas_ctx = canvas_ctx.get_untracked().unwrap();
    //                 let color = format!("#{:X?}{:X?}{:X?}", color.0, color.1, color.2);
    //                 canvas_ctx.set_fill_style(&JsValue::from_str(&color));
    //
    //                 let x = index as f64 % 32.0;
    //                 let y = (index as f64 / 32.0).floor();
    //                 console_warn(&format!("Filling pixel at {x}:{y} with color {color}"));
    //                 canvas_ctx.fill_rect(x, y, 1.0, 1.0);
    //             }
    //         });
    //     }
    // });

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
                <button on:click={move |_| { set_game_state.set(GameState::Paused); set_cpu.update(|cpu| cpu.reset())}}>Reset</button>
            </section>
        </main>
    }
}

// Screen is 32x32, and has four color channels (rgba) (A will always be 255, but it is required
// within the canvas api)
fn read_screen_state(cpu: &Cpu) -> [u8; 32 * 32 * 4] {
    let mut screen_state = [0; 32 * 32 * 4];

    // Games will place pixels between these two addresses in memory
    (0x0200..0x0600)
        .into_iter()
        .enumerate()
        .for_each(|(frame_index, memory_address)| {
            let color_idx = cpu.mem_read(memory_address as u16);
            let (r, g, b) = color(color_idx);

            let screen_index = frame_index * 4;
            screen_state[screen_index] = r;
            screen_state[screen_index + 1] = g;
            screen_state[screen_index + 2] = b;
            screen_state[screen_index + 3] = 255;
        });

    screen_state
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
