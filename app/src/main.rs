#![no_main]
#![no_std]

mod matrix;

use assign_resources::assign_resources;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    gpio::{AnyPin, Input, Pull},
    peripherals,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;
use matrix::{
    display_frame, init_leds, scroll_frames, MatrixFrame, ScrollDirection, LETTER_FRAMES,
};
use panic_probe as _;

assign_resources! {
    matrix_pins: LedMatrixPins {
        col1: P0_28,
        col2: P0_11,
        col3: P0_31,
        col4: P1_05,
        col5: P0_30,
        row1: P0_21,
        row2: P0_22,
        row3: P0_15,
        row4: P0_24,
        row5: P0_19,
    }

    buttons: ButtonPins {
        a: P0_14,
        b: P0_23
    }
}

#[derive(Clone, Copy)]
enum Button {
    A,
    B,
}

static SIGNAL: Signal<ThreadModeRawMutex, Button> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting...");
    let p = embassy_nrf::init(Default::default());

    let resources = split_resources!(p);

    spawner.spawn(animate(resources.matrix_pins)).unwrap();

    let buttons = resources.buttons;

    let button_a = button(buttons.a, "A", Button::A);
    let button_b = button(buttons.b, "B", Button::B);
    join(button_a, button_b).await;
}

async fn button<T: Into<AnyPin>>(pin: T, id: &str, b: Button) {
    let mut button = Input::new(pin.into(), Pull::None);
    loop {
        button.wait_for_low().await;
        info!("Button {} pressed (fut)", id);
        SIGNAL.signal(b);
        Timer::after_millis(200).await;
        button.wait_for_high().await;
    }
}

#[embassy_executor::task]
async fn animate(led_pins: LedMatrixPins) {
    let mut matrix = init_leds(led_pins);

    // let mut frame = &frame1;

    let mut frames = [LETTER_FRAMES[&'A'], LETTER_FRAMES[&'B']];

    loop {
        // if let Some(button) = SIGNAL.try_take() {
        //     match button {
        //         Button::A => frame = &frame1,
        //         Button::B => frame = &frame2,
        //     }
        // }

        // display_frame(&mut matrix, frame).await;

        scroll_frames(&frames, &mut matrix, ScrollDirection::Left).await;

        frames.swap(0, 1);

        scroll_frames(&frames, &mut matrix, ScrollDirection::Right).await;

        frames.swap(0, 1);
    }
}
