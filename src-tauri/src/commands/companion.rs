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

use crate::{
    authorization::{
        authorize_command, CommandAuthorization, GET_COMPANION_WINDOW_POSITION,
        GET_CURSOR_POSITION, MOVE_COMPANION_WINDOW, SET_COMPANION_CONTENT_HEIGHT,
        SHOW_COMPANION_CONTEXT_MENU, STOP_CURSOR_POSITIONS, STREAM_CURSOR_POSITIONS,
    },
    desktop::windows::companion,
};

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

    fn stop(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
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
    authorize(&window, GET_CURSOR_POSITION)?;
    logical_cursor_position(&window).map_err(|_| CompanionCommandError::CursorUnavailable)
}

#[tauri::command]
pub(crate) fn get_companion_window_position<R: Runtime>(
    window: WebviewWindow<R>,
) -> Result<ScreenPoint, CompanionCommandError> {
    authorize(&window, GET_COMPANION_WINDOW_POSITION)?;

    let position = companion::logical_position(&window)
        .map_err(|_| CompanionCommandError::WindowOperationFailed)?;

    Ok(ScreenPoint {
        x: position.x,
        y: position.y,
    })
}

#[tauri::command]
pub(crate) fn move_companion_window<R: Runtime>(
    window: WebviewWindow<R>,
    position: ScreenPoint,
) -> Result<(), CompanionCommandError> {
    authorize(&window, MOVE_COMPANION_WINDOW)?;

    let position = normalize_position(position).ok_or(CompanionCommandError::InvalidPosition)?;

    companion::move_to(&window, position).map_err(|_| CompanionCommandError::WindowOperationFailed)
}

#[tauri::command]
pub(crate) fn set_companion_content_height<R: Runtime>(
    window: WebviewWindow<R>,
    height: f64,
) -> Result<(), CompanionCommandError> {
    authorize(&window, SET_COMPANION_CONTENT_HEIGHT)?;

    if !is_valid_content_height(height) {
        return Err(CompanionCommandError::InvalidContentHeight);
    }

    companion::set_content_height(&window, height)
        .map_err(|_| CompanionCommandError::WindowOperationFailed)
}

#[tauri::command]
pub(crate) fn show_companion_context_menu<R: Runtime>(
    window: WebviewWindow<R>,
) -> Result<(), CompanionCommandError> {
    authorize(&window, SHOW_COMPANION_CONTEXT_MENU)?;

    crate::desktop::menus::show_companion_context_menu(&window)
        .map_err(|_| CompanionCommandError::WindowOperationFailed)
}

#[tauri::command]
pub(crate) fn stream_cursor_positions<R: Runtime>(
    window: WebviewWindow<R>,
    on_position: Channel<ScreenPoint>,
    state: State<'_, CursorStreamState>,
) -> Result<(), CompanionCommandError> {
    authorize(&window, STREAM_CURSOR_POSITIONS)?;

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

#[tauri::command]
pub(crate) fn stop_cursor_positions<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, CursorStreamState>,
) -> Result<(), CompanionCommandError> {
    authorize(&window, STOP_CURSOR_POSITIONS)?;
    state.stop();

    Ok(())
}

fn authorize<R: Runtime>(
    window: &WebviewWindow<R>,
    command: CommandAuthorization,
) -> Result<(), CompanionCommandError> {
    authorize_command(window.label(), command)
        .map(|_| ())
        .map_err(|_| CompanionCommandError::UnauthorizedWindow)
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
    let primary_scale_factor = window
        .primary_monitor()?
        .map(|monitor| monitor.scale_factor());
    let companion_scale_factor = window.scale_factor()?;

    Ok(cursor_position_in_desktop_logical_space(
        physical_position,
        primary_scale_factor,
        companion_scale_factor,
    ))
}

fn cursor_position_in_desktop_logical_space(
    physical_position: PhysicalPosition<f64>,
    primary_scale_factor: Option<f64>,
    companion_scale_factor: f64,
) -> ScreenPoint {
    // Tao represents its global cursor point using the primary monitor's scale
    // factor. Convert with that same factor so the result matches Electron's
    // desktop DIP coordinates and the renderer's CSS screen coordinates.
    //
    // The companion factor is only a defensive fallback for platforms or
    // transient display states where no primary monitor is available.
    let source_scale_factor = primary_scale_factor.unwrap_or(companion_scale_factor);
    let logical_position = physical_position.to_logical::<f64>(source_scale_factor);

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
    fn converts_mixed_dpi_cursor_coordinates_with_the_primary_scale() {
        assert_eq!(
            cursor_position_in_desktop_logical_space(
                PhysicalPosition::new(6_630.0, 846.0),
                Some(2.0),
                1.0,
            ),
            ScreenPoint {
                x: 3_315.0,
                y: 423.0,
            },
        );
    }

    #[test]
    fn ignores_retina_companion_scale_when_primary_scale_is_one() {
        assert_eq!(
            cursor_position_in_desktop_logical_space(
                PhysicalPosition::new(1_280.0, 720.0),
                Some(1.0),
                2.0,
            ),
            ScreenPoint {
                x: 1_280.0,
                y: 720.0,
            },
        );
    }

    #[test]
    fn preserves_retina_to_external_monitor_origin_and_offset() {
        let external_monitor_origin = ScreenPoint {
            x: 1_710.0,
            y: 32.0,
        };
        let cursor_offset_on_external_monitor = ScreenPoint { x: 820.0, y: 391.0 };
        let expected = ScreenPoint {
            x: external_monitor_origin.x + cursor_offset_on_external_monitor.x,
            y: external_monitor_origin.y + cursor_offset_on_external_monitor.y,
        };
        let tao_physical_position = PhysicalPosition::new(expected.x * 2.0, expected.y * 2.0);

        assert_eq!(
            cursor_position_in_desktop_logical_space(tao_physical_position, Some(2.0), 1.0,),
            expected,
        );
    }

    #[test]
    fn falls_back_to_companion_scale_when_primary_monitor_is_unavailable() {
        assert_eq!(
            cursor_position_in_desktop_logical_space(
                PhysicalPosition::new(-640.0, 512.0),
                None,
                2.0,
            ),
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

    #[test]
    fn stopping_cursor_stream_invalidates_the_active_generation() {
        let state = CursorStreamState::default();
        let active = state.begin();

        state.stop();

        assert!(!state.is_current(active));
    }
}
