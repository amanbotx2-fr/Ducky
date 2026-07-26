use serde::{Deserialize, Serialize};
use tauri::{LogicalPosition, Runtime, WebviewWindow};

use crate::desktop::windows::companion;

const MAX_ABSOLUTE_WINDOW_COORDINATE: f64 = 100_000.0;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub(crate) struct ScreenPoint {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CompanionCommandError {
    UnauthorizedWindow,
    InvalidPosition,
    WindowOperationFailed,
}

#[tauri::command]
pub(crate) fn move_companion_window<R: Runtime>(
    window: WebviewWindow<R>,
    position: ScreenPoint,
) -> Result<(), CompanionCommandError> {
    if window.label() != companion::LABEL {
        return Err(CompanionCommandError::UnauthorizedWindow);
    }

    let position = normalize_position(position).ok_or(CompanionCommandError::InvalidPosition)?;

    companion::move_to(&window, position).map_err(|_| CompanionCommandError::WindowOperationFailed)
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
}
