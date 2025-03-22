use core::borrow::BorrowMut;

use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Instant, Timer};
use phf::phf_map;

use crate::LedMatrixPins;

pub const MATRIX_SIZE: usize = 5;

#[derive(Clone, Copy)]
pub enum MatrixCell {
    Lit,
    Off,
}

use MatrixCell::{Lit, Off};

pub enum ScrollDirection {
    Left,
    Right,
}

pub struct LedMatrix {
    col: [Output<'static>; MATRIX_SIZE],
    row: [Output<'static>; MATRIX_SIZE],
}

// Inner = row, outer = col
#[derive(Clone, Copy)]
pub struct MatrixFrame(pub [[MatrixCell; MATRIX_SIZE]; MATRIX_SIZE]);

struct Animation([Option<(MatrixFrame, Duration)>; 10]);

pub fn init_leds(matrix_pins: LedMatrixPins) -> LedMatrix {
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

pub async fn display_frame(matrix: &mut LedMatrix, frame: &MatrixFrame) -> () {
    const CYCLE_RATE: Duration = Duration::from_millis((1000 / 60) / MATRIX_SIZE as u64);

    for row_index in 0..MATRIX_SIZE {
        matrix.row[row_index].set_high();
        for col_index in 0..MATRIX_SIZE {
            let col = &frame.0[row_index][col_index];
            match col {
                MatrixCell::Lit => matrix.col[col_index].set_low(),
                _ => (),
            }
        }
        Timer::after(CYCLE_RATE).await;
        for col in matrix.col.iter_mut() {
            col.set_high();
        }
        matrix.row[row_index].set_low();
    }
}

pub static LETTER_FRAMES: phf::Map<char, MatrixFrame> = phf_map! {
    'A' => MatrixFrame([
        [Off, Off, Lit, Off, Off],
        [Off, Lit, Off, Lit, Off],
        [Off, Lit, Lit, Lit, Off],
        [Off, Lit, Off, Lit, Off],
        [Off, Lit, Off, Lit, Off],
    ]),

    'B' => MatrixFrame([
        [Off, Lit, Lit, Off, Off],
        [Off, Lit, Off, Lit, Off],
        [Off, Lit, Lit, Off, Off],
        [Off, Lit, Off, Lit, Off],
        [Off, Lit, Lit, Off, Off],
    ]),
};

pub async fn scroll_frames(
    frames: &[MatrixFrame; 2],
    matrix: &mut LedMatrix,
    direction: ScrollDirection,
) {
    const DURATION: Duration = Duration::from_millis(500);

    let mut current_frame = frames[0];
    let mut new_frame = frames[1];

    display_frame_for_duration(matrix, &current_frame, DURATION).await;

    for _ in 0..MATRIX_SIZE {
        for row_index in 0..MATRIX_SIZE {
            let current_row = &mut current_frame.0[row_index];
            let new_row = &mut new_frame.0[row_index];

            match direction {
                ScrollDirection::Left => {
                    current_row.rotate_left(1);
                    *current_row.last_mut().unwrap() = *new_row.first().unwrap();
                    new_row.rotate_left(1);
                    *new_row.last_mut().unwrap() = MatrixCell::Off;
                }
                ScrollDirection::Right => {
                    current_row.rotate_right(1);
                    *current_row.first_mut().unwrap() = *new_row.last().unwrap();
                    new_row.rotate_right(1);
                    *new_row.first_mut().unwrap() = MatrixCell::Off;
                }
            }
        }
        display_frame_for_duration(matrix, &current_frame, DURATION).await;
    }
}

async fn display_frame_for_duration(
    matrix: &mut LedMatrix,
    frame: &MatrixFrame,
    duration: Duration,
) {
    let end_time = Instant::now() + duration;

    loop {
        if Instant::now() >= end_time {
            break;
        }

        display_frame(matrix, &frame).await;

        embassy_futures::yield_now().await;
    }
}

async fn display_animation(animation: Animation, matrix: &mut LedMatrix) {
    for data in animation.0 {
        if let Some((frame, duration)) = data {
            let end_time = Instant::now() + duration;

            loop {
                if Instant::now() >= end_time {
                    break;
                }

                display_frame(matrix, &frame).await;

                embassy_futures::yield_now().await;
            }
        } else {
            break;
        }
    }
}
