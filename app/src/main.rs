#![no_main]
#![no_std]

use assign_resources::assign_resources;
use common_lib::matrix::LedMatrix;
use common_lib::scroller::{ScrollDirection, Scroller, ScrollerError, MATRIX_SIZE};
use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::{bind_interrupts, peripherals, uarte};
use embassy_time::{Duration, Timer};
use panic_probe as _;

static VAR: i32 = 5;
static VAR: i32 = 5;

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

    uarte_resources: UartResources {
        uarte: UARTE0,
        rx: P1_08,
        tx: P0_06,
        ppi1: PPI_CH0,
        ppi2: PPI_CH1,
        timer: TIMER0,
    }

    buttons: ButtonPins {
        a: P0_14,
        b: P0_23
    }
}

bind_interrupts!(struct Irqs {
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting...");
    let p = embassy_nrf::init(Default::default());

    let resources = split_resources!(p);

    spawner.spawn(animate(resources.matrix_pins)).unwrap();
    spawner
        .spawn(command_line(resources.uarte_resources))
        .unwrap();
}

pub fn init_leds(matrix_pins: LedMatrixPins) -> LedMatrix<Output<'static>, MATRIX_SIZE> {
    LedMatrix {
        col: [
            Output::new(matrix_pins.col1, Level::High, OutputDrive::Standard),
            Output::new(matrix_pins.col2, Level::High, OutputDrive::Standard),
            Output::new(matrix_pins.col3, Level::High, OutputDrive::Standard),
            Output::new(matrix_pins.col4, Level::High, OutputDrive::Standard),
            Output::new(matrix_pins.col5, Level::High, OutputDrive::Standard),
        ],
        row: [
            Output::new(matrix_pins.row1, Level::Low, OutputDrive::Standard),
            Output::new(matrix_pins.row2, Level::Low, OutputDrive::Standard),
            Output::new(matrix_pins.row3, Level::Low, OutputDrive::Standard),
            Output::new(matrix_pins.row4, Level::Low, OutputDrive::Standard),
            Output::new(matrix_pins.row5, Level::Low, OutputDrive::Standard),
        ],
    }
}

#[embassy_executor::task]
async fn animate(matrix_pins: LedMatrixPins) {
    let mut matrix = init_leds(matrix_pins);
    let frame_time = Duration::from_millis(300);

    let mut scroller = Scroller::new(&mut matrix);

    loop {
        let input = "HELLO WORLD";
        let out = scroller
            .display_string(&input, ScrollDirection::Left, frame_time)
            .await;
        match out {
            Err(ScrollerError::UnsupportedCharacter(c)) => {
                warn!("Unknown Character {}", c);
                return;
            }
            _ => (),
        }
    }
}

#[embassy_executor::task]
async fn command_line(uarte_resources: UartResources) {
    let rx = uarte_resources.rx;
    let tx = uarte_resources.tx;
    let uarte = uarte_resources.uarte;
    let ppi1 = uarte_resources.ppi1;
    let ppi2 = uarte_resources.ppi2;
    let timer = uarte_resources.timer;

    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;
    let uarte_device = uarte::Uarte::new(uarte, Irqs, rx, tx, config);

    let (_, uarte_rx) = uarte_device.split_with_idle(timer, ppi1, ppi2);

    // uart_re_test(uarte_rx).await;
}
