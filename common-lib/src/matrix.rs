use core::convert::Infallible;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::{OutputPin, StatefulOutputPin};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MatrixCell {
    Lit,
    Off,
}

pub trait MatrixPin: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible> {}

impl<T> MatrixPin for T where
    T: OutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>
{
}

#[allow(async_fn_in_trait)]
pub trait MatrixDisplay<const SIZE: usize> {
    async fn display_frame(&mut self, frame: &MatrixFrame<SIZE>);
    async fn display_frame_for_duration(&mut self, frame: &MatrixFrame<SIZE>, duration: Duration) {
        let end_time = Instant::now() + duration;

        self.display_frame(frame).await;

        while Instant::now() < end_time {
            let refresh_interval = Duration::from_millis(1);
            Timer::after(refresh_interval).await;
            self.display_frame(frame).await;
        }
    }
}

// Inner = row, outer = col
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MatrixFrame<const SIZE: usize>(pub [[MatrixCell; SIZE]; SIZE]);

impl<const SIZE: usize> Default for MatrixFrame<SIZE> {
    fn default() -> Self {
        use crate::matrix::MatrixCell;
        Self(core::array::from_fn(|_| {
            core::array::from_fn(|_| MatrixCell::Off)
        }))
    }
}

pub struct LedMatrix<P, const SIZE: usize>
where
    P: MatrixPin,
{
    pub col: [P; SIZE],
    pub row: [P; SIZE],
}

impl<P, const SIZE: usize> LedMatrix<P, SIZE>
where
    P: MatrixPin,
{
    pub fn new(col: [P; SIZE], row: [P; SIZE]) -> Self {
        LedMatrix { col, row }
    }

    fn get_cycle_rate(&self) -> Duration {
        Duration::from_millis((1000 / 60) / SIZE as u64)
    }
}

impl<P, const SIZE: usize> MatrixDisplay<SIZE> for LedMatrix<P, SIZE>
where
    P: MatrixPin,
{
    async fn display_frame(&mut self, frame: &MatrixFrame<SIZE>) -> () {
        for row_index in 0..SIZE {
            self.row[row_index].set_high().unwrap();
            for col_index in 0..SIZE {
                let col = &frame.0[row_index][col_index];
                match col {
                    MatrixCell::Lit => self.col[col_index].set_low().unwrap(),
                    _ => (),
                }
            }
            Timer::after(self.get_cycle_rate()).await;
            for col in self.col.iter_mut() {
                col.set_high().unwrap();
            }
            self.row[row_index].set_low().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpio_mock::*;
    use embedded_hal::digital::PinState;
    use MatrixCell::Lit;
    use MatrixCell::Off;

    #[futures_test::test]
    async fn test_display_frame_1x1_all_lit() {
        let mut col = [MockOutput::new()];
        let mut row = [MockOutput::new()];

        let frame = MatrixFrame([[Lit]]);

        expect_gpio(&mut row[0], PinState::High);
        expect_gpio(&mut col[0], PinState::Low);
        expect_gpio(&mut col[0], PinState::High);
        expect_gpio(&mut row[0], PinState::Low);

        let mut matrix = LedMatrix { col, row };

        matrix.display_frame(&frame).await;
    }

    const UPPER_BOUND_FACTOR: f64 = 1.1;
    const LOWER_BOUND_FACTOR: f64 = 0.9;

    #[futures_test::test]
    async fn test_display_frame_1x1_all_off() {
        let mut col = [MockOutput::new()];
        let mut row = [MockOutput::new()];

        let frame = MatrixFrame([[Off]]);

        expect_gpio(&mut row[0], PinState::High);
        expect_gpio(&mut col[0], PinState::High);
        expect_gpio(&mut row[0], PinState::Low);

        let mut matrix = LedMatrix { col, row };

        let start_time = Instant::now();

        matrix.display_frame(&frame).await;

        let expected_duration = matrix.get_cycle_rate().as_ticks() as f64;
        let lower_bound = expected_duration * LOWER_BOUND_FACTOR;
        let upper_bound = expected_duration * UPPER_BOUND_FACTOR;
        let elapsed = (Instant::now() - start_time).as_ticks() as f64;

        assert!(elapsed >= lower_bound && elapsed <= upper_bound);
    }

    #[futures_test::test]
    async fn test_display_frame_2x2_all_lit() {
        let mut col = [MockOutput::new(), MockOutput::new()];
        let mut row = [MockOutput::new(), MockOutput::new()];

        let frame = MatrixFrame([[Lit, Lit], [Lit, Lit]]);

        expect_gpio(&mut row[0], PinState::High);
        expect_gpio(&mut col[0], PinState::Low);
        expect_gpio(&mut col[1], PinState::Low);
        expect_gpio(&mut col[0], PinState::High);
        expect_gpio(&mut col[1], PinState::High);
        expect_gpio(&mut row[0], PinState::Low);

        expect_gpio(&mut row[1], PinState::High);
        expect_gpio(&mut col[0], PinState::Low);
        expect_gpio(&mut col[1], PinState::Low);
        expect_gpio(&mut col[0], PinState::High);
        expect_gpio(&mut col[1], PinState::High);
        expect_gpio(&mut row[1], PinState::Low);

        let mut matrix = LedMatrix { col, row };

        let start_time = Instant::now();

        matrix.display_frame(&frame).await;

        let num_cycles = 2.0;
        let expected_duration = matrix.get_cycle_rate().as_ticks() as f64 * num_cycles;
        let lower_bound = expected_duration * LOWER_BOUND_FACTOR;
        let upper_bound = expected_duration * UPPER_BOUND_FACTOR;
        let elapsed = (Instant::now() - start_time).as_ticks() as f64;

        assert!(elapsed >= lower_bound && elapsed <= upper_bound);
    }

    #[futures_test::test]
    async fn test_display_frame_2x2_diagonal() {
        let mut col = [MockOutput::new(), MockOutput::new()];
        let mut row = [MockOutput::new(), MockOutput::new()];

        let frame = MatrixFrame([[Lit, Off], [Off, Lit]]);

        expect_gpio(&mut row[0], PinState::High);
        expect_gpio(&mut col[0], PinState::Low);
        expect_gpio(&mut col[0], PinState::High);
        expect_gpio(&mut col[1], PinState::High);
        expect_gpio(&mut row[0], PinState::Low);

        expect_gpio(&mut row[1], PinState::High);
        expect_gpio(&mut col[1], PinState::Low);
        expect_gpio(&mut col[0], PinState::High);
        expect_gpio(&mut col[1], PinState::High);
        expect_gpio(&mut row[1], PinState::Low);

        let mut matrix = LedMatrix { col, row };

        matrix.display_frame(&frame).await;
    }

    #[futures_test::test]
    async fn test_display_frame_for_duration() {
        let mut col = [MockOutput::new()];
        let mut row = [MockOutput::new()];

        let frame = MatrixFrame([[Lit]]);

        let _ = col[0].expect_set_high().returning(|| Ok(()));
        let _ = col[0].expect_set_low().returning(|| Ok(()));

        let _ = row[0].expect_set_high().returning(|| Ok(()));
        let _ = row[0].expect_set_low().returning(|| Ok(()));

        let mut matrix = LedMatrix { col, row };

        let duration = Duration::from_millis(50);

        let start_time = Instant::now();

        matrix.display_frame_for_duration(&frame, duration).await;

        let lower_bound = duration.as_ticks() as f64 * LOWER_BOUND_FACTOR;
        let upper_bound = duration.as_ticks() as f64 * UPPER_BOUND_FACTOR;
        let elapsed = (Instant::now() - start_time).as_ticks() as f64;

        assert!(elapsed >= lower_bound && elapsed <= upper_bound);
    }
}
