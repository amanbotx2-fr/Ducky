use tauri::{
    App, LogicalPosition, LogicalSize, Monitor, PhysicalPosition, PhysicalRect, PhysicalSize,
    Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub const LABEL: &str = "companion";

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

    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
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
        .visible(false)
        .build()?;

    if let Some(monitor) = primary_monitor {
        position_on_monitor(&window, &monitor)?;
    }

    window.show()?;

    Ok(window)
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

fn position_on_monitor<R: Runtime>(
    window: &WebviewWindow<R>,
    monitor: &Monitor,
) -> tauri::Result<()> {
    let window_size = window.outer_size()?;
    let margin = LogicalSize::new(MARGIN, MARGIN)
        .to_physical::<u32>(monitor.scale_factor())
        .width;
    let position = bottom_right_position(*monitor.work_area(), window_size, margin);

    window.set_position(position)
}

fn bottom_right_position(
    work_area: PhysicalRect<i32, u32>,
    window_size: PhysicalSize<u32>,
    margin: u32,
) -> PhysicalPosition<i32> {
    PhysicalPosition::new(
        anchored_coordinate(
            work_area.position.x,
            work_area.size.width,
            window_size.width,
            margin,
        ),
        anchored_coordinate(
            work_area.position.y,
            work_area.size.height,
            window_size.height,
            margin,
        ),
    )
}

fn anchored_coordinate(origin: i32, extent: u32, window_extent: u32, margin: u32) -> i32 {
    let coordinate =
        i64::from(origin) + i64::from(extent) - i64::from(window_extent) - i64::from(margin);

    coordinate.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
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
        let position = bottom_right_position(
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(1920, 1080),
            },
            PhysicalSize::new(220, 220),
            24,
        );

        assert_eq!(position, PhysicalPosition::new(1676, 836));
    }

    #[test]
    fn preserves_negative_virtual_desktop_coordinates() {
        let position = bottom_right_position(
            PhysicalRect {
                position: PhysicalPosition::new(-1920, -120),
                size: PhysicalSize::new(1920, 1080),
            },
            PhysicalSize::new(220, 220),
            24,
        );

        assert_eq!(position, PhysicalPosition::new(-244, 716));
    }

    #[test]
    fn applies_scaled_physical_margin() {
        let margin = LogicalSize::new(MARGIN, MARGIN)
            .to_physical::<u32>(2.0)
            .width;
        let position = bottom_right_position(
            PhysicalRect {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(3840, 2160),
            },
            PhysicalSize::new(440, 440),
            margin,
        );

        assert_eq!(margin, 48);
        assert_eq!(position, PhysicalPosition::new(3352, 1672));
    }

    #[test]
    fn matches_electron_formula_when_work_area_is_smaller_than_window() {
        let position = bottom_right_position(
            PhysicalRect {
                position: PhysicalPosition::new(10, 20),
                size: PhysicalSize::new(200, 150),
            },
            PhysicalSize::new(220, 150),
            24,
        );

        assert_eq!(position, PhysicalPosition::new(-34, -4));
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
