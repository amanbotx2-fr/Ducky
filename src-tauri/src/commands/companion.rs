use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, LogicalPosition, PhysicalPosition, Runtime, State, WebviewWindow};

use crate::desktop::windows::companion;

const MAX_ABSOLUTE_WINDOW_COORDINATE: f64 = 100_000.0;
const MAX_COMPANION_CONTENT_HEIGHT: f64 = 10_000.0;
const CURSOR_SAMPLE_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 30);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct ScreenPoint {
    x: f64,
    y: f64,
}

#[derive(Clone, Default)]
pub(crate) struct CursorStreamState {
    generation: Arc<AtomicU64>,
}

impl CursorStreamState {
    fn begin(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }

    fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::Acquire) == generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CompanionCommandError {
    UnauthorizedWindow,
    InvalidPosition,
    InvalidContentHeight,
    CursorUnavailable,
    WindowOperationFailed,
}

#[tauri::command]
pub(crate) fn get_cursor_position<R: Runtime>(
    window: WebviewWindow<R>,
) -> Result<ScreenPoint, CompanionCommandError> {
    authorize_companion(&window)?;
    logical_cursor_position(&window).map_err(|_| CompanionCommandError::CursorUnavailable)
}

#[tauri::command]
pub(crate) fn move_companion_window<R: Runtime>(
    window: WebviewWindow<R>,
    position: ScreenPoint,
) -> Result<(), CompanionCommandError> {
    authorize_companion(&window)?;

    let position = normalize_position(position).ok_or(CompanionCommandError::InvalidPosition)?;

    companion::move_to(&window, position).map_err(|_| CompanionCommandError::WindowOperationFailed)
}

#[tauri::command]
pub(crate) fn set_companion_content_height<R: Runtime>(
    window: WebviewWindow<R>,
    height: f64,
) -> Result<(), CompanionCommandError> {
    authorize_companion(&window)?;

    if !is_valid_content_height(height) {
        return Err(CompanionCommandError::InvalidContentHeight);
    }

    companion::set_content_height(&window, height)
        .map_err(|_| CompanionCommandError::WindowOperationFailed)
}

#[tauri::command]
pub(crate) fn stream_cursor_positions<R: Runtime>(
    window: WebviewWindow<R>,
    on_position: Channel<ScreenPoint>,
    state: State<'_, CursorStreamState>,
) -> Result<(), CompanionCommandError> {
    authorize_companion(&window)?;

    let stream_state = state.inner().clone();
    let generation = stream_state.begin();

    thread::Builder::new()
        .name("ducky-cursor-stream".into())
        .spawn(move || {
            run_cursor_stream(window, on_position, stream_state, generation);
        })
        .map_err(|_| CompanionCommandError::WindowOperationFailed)?;

    Ok(())
}

fn authorize_companion<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), CompanionCommandError> {
    if window.label() == companion::LABEL {
        Ok(())
    } else {
        Err(CompanionCommandError::UnauthorizedWindow)
    }
}

fn run_cursor_stream<R: Runtime>(
    window: WebviewWindow<R>,
    on_position: Channel<ScreenPoint>,
    state: CursorStreamState,
    generation: u64,
) {
    let mut last_position = None;

    while state.is_current(generation) {
        let Ok(position) = logical_cursor_position(&window) else {
            break;
        };

        if last_position != Some(position) {
            if on_position.send(position).is_err() {
                break;
            }

            last_position = Some(position);
        }

        thread::sleep(CURSOR_SAMPLE_INTERVAL);
    }
}

fn logical_cursor_position<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<ScreenPoint> {
    let physical_position = window.cursor_position()?;
    let scale_factor = window.scale_factor()?;

    Ok(cursor_position_for_scale(physical_position, scale_factor))
}

fn cursor_position_for_scale(
    physical_position: PhysicalPosition<f64>,
    scale_factor: f64,
) -> ScreenPoint {
    let logical_position = physical_position.to_logical::<f64>(scale_factor);

    ScreenPoint {
        x: logical_position.x,
        y: logical_position.y,
    }
}

fn normalize_position(position: ScreenPoint) -> Option<LogicalPosition<i32>> {
    if !is_valid_coordinate(position.x) || !is_valid_coordinate(position.y) {
        return None;
    }

    Some(LogicalPosition::new(
        round_like_javascript(position.x),
        round_like_javascript(position.y),
    ))
}

fn is_valid_coordinate(value: f64) -> bool {
    value.is_finite() && value.abs() <= MAX_ABSOLUTE_WINDOW_COORDINATE
}

fn is_valid_content_height(value: f64) -> bool {
    value.is_finite() && value > 0.0 && value <= MAX_COMPANION_CONTENT_HEIGHT
}

fn round_like_javascript(value: f64) -> i32 {
    (value + 0.5).floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_rounds_coordinates_like_electron() {
        assert_eq!(
            normalize_position(ScreenPoint { x: 10.49, y: -1.5 }),
            Some(LogicalPosition::new(10, -1)),
        );
        assert_eq!(
            normalize_position(ScreenPoint {
                x: -100_000.0,
                y: 100_000.0,
            }),
            Some(LogicalPosition::new(-100_000, 100_000)),
        );
    }

    #[test]
    fn rejects_non_finite_and_out_of_bounds_coordinates() {
        for point in [
            ScreenPoint {
                x: f64::NAN,
                y: 0.0,
            },
            ScreenPoint {
                x: 0.0,
                y: f64::INFINITY,
            },
            ScreenPoint {
                x: 100_000.1,
                y: 0.0,
            },
            ScreenPoint {
                x: 0.0,
                y: -100_000.1,
            },
        ] {
            assert_eq!(normalize_position(point), None);
        }
    }

    #[test]
    fn accepts_only_finite_bounded_positive_content_heights() {
        for height in [f64::NAN, f64::INFINITY, -1.0, 0.0, 10_000.1] {
            assert!(!is_valid_content_height(height));
        }

        for height in [0.1, 220.0, 10_000.0] {
            assert!(is_valid_content_height(height));
        }
    }

    #[test]
    fn converts_cursor_coordinates_to_the_window_logical_scale() {
        assert_eq!(
            cursor_position_for_scale(PhysicalPosition::new(-640.0, 512.0), 2.0,),
            ScreenPoint {
                x: -320.0,
                y: 256.0,
            },
        );
    }

    #[test]
    fn newer_cursor_stream_generation_supersedes_the_previous_one() {
        let state = CursorStreamState::default();
        let first = state.begin();
        let second = state.begin();

        assert!(!state.is_current(first));
        assert!(state.is_current(second));
    }
}
