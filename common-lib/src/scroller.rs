use crate::{
    frame_ascii::LETTER_FRAMES,
    matrix::{MatrixCell, MatrixDisplay, MatrixFrame},
};
use embassy_time::Duration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ScrollerError {
    #[error("")]
    UnsupportedCharacter(char),
}

#[derive(Clone)]
pub enum ScrollDirection {
    Left,
    Right,
}

pub const MATRIX_SIZE: usize = 5;

pub struct Scroller<'a, M>
where
    M: MatrixDisplay<MATRIX_SIZE>,
{
    matrix: &'a mut M,
}

impl<'a, M> Scroller<'a, M>
where
    M: MatrixDisplay<MATRIX_SIZE>,
{
    pub fn new(matrix: &'a mut M) -> Self {
        Self { matrix }
    }

    fn generate_transition_frames(
        &mut self,
        frames: &[MatrixFrame<MATRIX_SIZE>; 2],
        direction: ScrollDirection,
    ) -> [MatrixFrame<MATRIX_SIZE>; MATRIX_SIZE - 1] {
        let mut current_frame = frames[0];
        let mut new_frame = frames[1];

        let mut frames: [MatrixFrame<MATRIX_SIZE>; MATRIX_SIZE - 1] = Default::default();

        for i in 0..(MATRIX_SIZE - 1) {
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
            frames[i] = current_frame;
        }

        return frames;
    }

    pub async fn display_string(
        &mut self,
        string: &str,
        direction: ScrollDirection,
        frame_time: Duration,
    ) -> Result<(), ScrollerError> {
        for char in string.chars() {
            if !LETTER_FRAMES.contains_key(&char) {
                return Err(ScrollerError::UnsupportedCharacter(char));
            }
        }

        let initial_chars = [
            LETTER_FRAMES[&' '],
            LETTER_FRAMES[&string.chars().nth(0).unwrap()],
        ];

        let initial_frames = self.generate_transition_frames(&initial_chars, direction.clone());
        for frames in initial_frames {
            self.matrix
                .display_frame_for_duration(&frames, frame_time)
                .await;
        }

        let bytes = string.as_bytes();
        for window in bytes.windows(2) {
            let chars = [(window[0] as char), (window[1] as char)];

            let frame_window = [LETTER_FRAMES[&chars[0]], LETTER_FRAMES[&chars[1]]];

            self.matrix
                .display_frame_for_duration(&frame_window[0], frame_time)
                .await;

            let trans_frames = self.generate_transition_frames(&frame_window, direction.clone());
            for frames in trans_frames {
                self.matrix
                    .display_frame_for_duration(&frames, frame_time)
                    .await;
            }
        }

        let last_char_frame = LETTER_FRAMES[&string.chars().last().unwrap()];

        self.matrix
            .display_frame_for_duration(&last_char_frame, frame_time)
            .await;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::matrix::MatrixDisplay;
    use MatrixCell::Lit;
    use MatrixCell::Off;

    struct MockMatrix<const SIZE: usize> {
        frames: Vec<MatrixFrame<SIZE>>,
    }

    impl<const SIZE: usize> MockMatrix<SIZE> {
        fn new() -> Self {
            Self { frames: Vec::new() }
        }
    }

    impl<const SIZE: usize> MatrixDisplay<SIZE> for MockMatrix<SIZE> {
        async fn display_frame(&mut self, frame: &MatrixFrame<SIZE>) {
            self.frames.push(*frame);
        }
    }

    #[futures_test::test]
    async fn test_scroll_frames_left() {
        let mut matrix = MockMatrix::new();
        let frames = [LETTER_FRAMES[&'A'], LETTER_FRAMES[&'B']];

        let mut scroller = Scroller::new(&mut matrix);

        let outputted_frames = scroller.generate_transition_frames(&frames, ScrollDirection::Left);

        let expected_frames = [
            MatrixFrame([
                [Off, Lit, Off, Off, Off],
                [Lit, Off, Lit, Off, Off],
                [Lit, Lit, Lit, Off, Off],
                [Lit, Off, Lit, Off, Off],
                [Lit, Off, Lit, Off, Off],
            ]),
            MatrixFrame([
                [Lit, Off, Off, Off, Lit],
                [Off, Lit, Off, Off, Lit],
                [Lit, Lit, Off, Off, Lit],
                [Off, Lit, Off, Off, Lit],
                [Off, Lit, Off, Off, Lit],
            ]),
            MatrixFrame([
                [Off, Off, Off, Lit, Lit],
                [Lit, Off, Off, Lit, Off],
                [Lit, Off, Off, Lit, Lit],
                [Lit, Off, Off, Lit, Off],
                [Lit, Off, Off, Lit, Lit],
            ]),
            MatrixFrame([
                [Off, Off, Lit, Lit, Off],
                [Off, Off, Lit, Off, Lit],
                [Off, Off, Lit, Lit, Off],
                [Off, Off, Lit, Off, Lit],
                [Off, Off, Lit, Lit, Off],
            ]),
        ];

        assert_eq!(outputted_frames, expected_frames);
    }

    #[futures_test::test]
    async fn test_scroll_frames_right() {
        let mut matrix = MockMatrix::new();
        let frames = [LETTER_FRAMES[&'B'], LETTER_FRAMES[&'A']];
        let mut scroller = Scroller::new(&mut matrix);
        let outputted_frames = scroller.generate_transition_frames(&frames, ScrollDirection::Right);

        let expected_frames = [
            MatrixFrame([
                [Off, Off, Lit, Lit, Off],
                [Off, Off, Lit, Off, Lit],
                [Off, Off, Lit, Lit, Off],
                [Off, Off, Lit, Off, Lit],
                [Off, Off, Lit, Lit, Off],
            ]),
            MatrixFrame([
                [Off, Off, Off, Lit, Lit],
                [Lit, Off, Off, Lit, Off],
                [Lit, Off, Off, Lit, Lit],
                [Lit, Off, Off, Lit, Off],
                [Lit, Off, Off, Lit, Lit],
            ]),
            MatrixFrame([
                [Lit, Off, Off, Off, Lit],
                [Off, Lit, Off, Off, Lit],
                [Lit, Lit, Off, Off, Lit],
                [Off, Lit, Off, Off, Lit],
                [Off, Lit, Off, Off, Lit],
            ]),
            MatrixFrame([
                [Off, Lit, Off, Off, Off],
                [Lit, Off, Lit, Off, Off],
                [Lit, Lit, Lit, Off, Off],
                [Lit, Off, Lit, Off, Off],
                [Lit, Off, Lit, Off, Off],
            ]),
        ];

        assert_eq!(outputted_frames, expected_frames);
    }

    #[futures_test::test]
    async fn test_display_string() {
        let mut matrix = MockMatrix::new();
        let frame_time = Duration::from_millis(0); // Frames are always displayed once
        let test_string = "TEST";

        let mut scroller = Scroller::new(&mut matrix);
        scroller
            .display_string(&test_string, ScrollDirection::Right, frame_time)
            .await
            .unwrap();

        let initial_transitions = 4;
        let outputted_char_frames = matrix
            .frames
            .iter()
            .skip(initial_transitions)
            .enumerate()
            // Remove the transition frames
            .filter(|(i, _)| i % MATRIX_SIZE == 0)
            .map(|(_, f)| f)
            .collect::<Vec<&MatrixFrame<MATRIX_SIZE>>>();

        let expected_char_frames = test_string
            .as_bytes()
            .iter()
            .filter_map(|&c| LETTER_FRAMES.get(&(c as char)))
            .collect::<Vec<&MatrixFrame<MATRIX_SIZE>>>();

        assert_eq!(outputted_char_frames, expected_char_frames);
    }

    #[futures_test::test]
    async fn test_display_string_error() {
        let mut matrix = MockMatrix::new();
        let frame_time = Duration::from_millis(0); // Frames are always displayed once
        let test_string = "@";

        let mut scroller = Scroller::new(&mut matrix);
        let err = scroller
            .display_string(&test_string, ScrollDirection::Right, frame_time)
            .await
            .unwrap_err();

        assert_eq!(err, ScrollerError::UnsupportedCharacter('@'));
        assert!(matrix.frames.is_empty());
    }
}
