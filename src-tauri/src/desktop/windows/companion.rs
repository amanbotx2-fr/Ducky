use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::{
    webview::PageLoadEvent, App, LogicalPosition, LogicalSize, Monitor, PhysicalPosition,
    PhysicalRect, PhysicalSize, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub const LABEL: &str = crate::authorization::COMPANION_LABEL;

const TITLE: &str = "Ducky";
const WIDTH: f64 = 220.0;
const HEIGHT: f64 = 220.0;
const MARGIN: f64 = 24.0;

pub fn create<R: Runtime>(app: &App<R>) -> tauri::Result<WebviewWindow<R>> {
    let primary_monitor = app.primary_monitor()?;
    let initial_height = primary_monitor
        .as_ref()
        .map(companion_height_for)
        .unwrap_or(HEIGHT);
    let initial_position = primary_monitor
        .as_ref()
        .map(|monitor| initial_position_for(monitor, initial_height));
    let startup_handled = Arc::new(AtomicBool::new(false));

    let mut window_builder =
        WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
            .title(TITLE)
            .inner_size(WIDTH, initial_height)
            .transparent(true)
            .decorations(false)
            .shadow(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .fullscreen(false)
            .skip_taskbar(true)
            .visible(false);

    if let Some(position) = initial_position {
        window_builder = window_builder.position(position.x, position.y);
    }

    let window = window_builder
        .on_page_load(move |window, payload| {
            if payload.event() == PageLoadEvent::Finished
                && startup_handled
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                show_after_page_load(&window);
            }
        })
        .build()?;

    Ok(window)
}

fn show_after_page_load<R: Runtime>(window: &WebviewWindow<R>) {
    if let Err(error) = window.show() {
        eprintln!("[window] companion_show_failed: {error}");
    }
}

pub fn move_to<R: Runtime>(
    window: &WebviewWindow<R>,
    position: LogicalPosition<i32>,
) -> tauri::Result<()> {
    window.set_position(position)
}

pub fn set_content_height<R: Runtime>(
    window: &WebviewWindow<R>,
    requested_height: f64,
) -> tauri::Result<()> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };

    let current_position = window.outer_position()?;
    let current_size = window.outer_size()?;
    let next_bounds = content_height_bounds(
        current_position,
        current_size,
        *monitor.work_area(),
        monitor.scale_factor(),
        requested_height,
    );

    if current_size.height == next_bounds.size.height {
        return Ok(());
    }

    window.set_size(next_bounds.size)?;
    window.set_position(next_bounds.position)
}

fn companion_height_for(monitor: &Monitor) -> f64 {
    let work_area_height = monitor.work_area().size.height as f64 / monitor.scale_factor();
    HEIGHT.min(work_area_height)
}

fn initial_position_for(monitor: &Monitor, height: f64) -> LogicalPosition<f64> {
    initial_position_from_work_area(*monitor.work_area(), monitor.scale_factor(), height)
}

fn initial_position_from_work_area(
    work_area: PhysicalRect<i32, u32>,
    scale_factor: f64,
    height: f64,
) -> LogicalPosition<f64> {
    let work_area_position = work_area.position.to_logical::<f64>(scale_factor);
    let work_area_size = work_area.size.to_logical::<f64>(scale_factor);

    LogicalPosition::new(
        work_area_position.x + work_area_size.width - WIDTH - MARGIN,
        work_area_position.y + work_area_size.height - height - MARGIN,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PhysicalBounds {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
}

fn content_height_bounds(
    current_position: PhysicalPosition<i32>,
    current_size: PhysicalSize<u32>,
    work_area: PhysicalRect<i32, u32>,
    scale_factor: f64,
    requested_height: f64,
) -> PhysicalBounds {
    let work_area_height = work_area.size.height as f64 / scale_factor;
    let next_logical_height = HEIGHT.max(requested_height.ceil().min(work_area_height));
    let next_height = LogicalSize::new(WIDTH, next_logical_height)
        .to_physical::<u32>(scale_factor)
        .height;
    let current_bottom = i64::from(current_position.y) + i64::from(current_size.height);
    let maximum_y = i64::from(work_area.position.y)
        + i64::from(work_area.size.height.saturating_sub(next_height));
    let next_y = (current_bottom - i64::from(next_height))
        .max(i64::from(work_area.position.y))
        .min(maximum_y)
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;

    PhysicalBounds {
        position: PhysicalPosition::new(current_position.x, next_y),
        size: PhysicalSize::new(current_size.width, next_height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_at_bottom_right_of_primary_work_area() {
        let position = initial_position_from_work_area(
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(1920, 1080),
            },
            1.0,
            HEIGHT,
        );

        assert_eq!(position, LogicalPosition::new(1676.0, 836.0));
    }

    #[test]
    fn preserves_negative_virtual_desktop_coordinates() {
        let position = initial_position_from_work_area(
            PhysicalRect {
                position: PhysicalPosition::new(-1920, -120),
                size: PhysicalSize::new(1920, 1080),
            },
            1.0,
            HEIGHT,
        );

        assert_eq!(position, LogicalPosition::new(-244.0, 716.0));
    }

    #[test]
    fn converts_retina_work_area_to_logical_builder_coordinates() {
        let position = initial_position_from_work_area(
            PhysicalRect {
                position: PhysicalPosition::new(0, 78),
                size: PhysicalSize::new(3420, 1994),
            },
            2.0,
            HEIGHT,
        );

        assert_eq!(position, LogicalPosition::new(1466.0, 792.0));
    }

    #[test]
    fn matches_electron_formula_when_work_area_is_smaller_than_window() {
        let position = initial_position_from_work_area(
            PhysicalRect {
                position: PhysicalPosition::new(10, 20),
                size: PhysicalSize::new(200, 150),
            },
            1.0,
            150.0,
        );

        assert_eq!(position, LogicalPosition::new(-34.0, -4.0));
    }

    #[test]
    fn grows_upward_and_preserves_the_bottom_edge() {
        let bounds = content_height_bounds(
            PhysicalPosition::new(1676, 836),
            PhysicalSize::new(220, 220),
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(1920, 1080),
            },
            1.0,
            480.2,
        );

        assert_eq!(
            bounds,
            PhysicalBounds {
                position: PhysicalPosition::new(1676, 575),
                size: PhysicalSize::new(220, 481),
            },
        );
        assert_eq!(bounds.position.y + bounds.size.height as i32, 1056);
    }

    #[test]
    fn clamps_height_and_position_to_the_current_work_area() {
        let bounds = content_height_bounds(
            PhysicalPosition::new(-400, -40),
            PhysicalSize::new(440, 440),
            PhysicalRect {
                position: PhysicalPosition::new(-1280, -200),
                size: PhysicalSize::new(1280, 900),
            },
            2.0,
            800.0,
        );

        assert_eq!(
            bounds,
            PhysicalBounds {
                position: PhysicalPosition::new(-400, -200),
                size: PhysicalSize::new(440, 900),
            },
        );
    }

    #[test]
    fn enforces_the_existing_minimum_height() {
        let bounds = content_height_bounds(
            PhysicalPosition::new(50, 100),
            PhysicalSize::new(220, 400),
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(1920, 1080),
            },
            1.0,
            80.0,
        );

        assert_eq!(
            bounds,
            PhysicalBounds {
                position: PhysicalPosition::new(50, 280),
                size: PhysicalSize::new(220, 220),
            },
        );
    }
}
