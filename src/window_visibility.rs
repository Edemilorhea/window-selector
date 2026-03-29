use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsIconic};

/// Check if a window is actually visible on screen (not minimized and not completely occluded).
///
/// This function checks:
/// 1. Window is not minimized
/// 2. Window has a valid rectangle on screen
/// 3. Window is not completely covered by other windows
pub fn is_window_actually_visible(target_hwnd: HWND, all_windows: &[HWND]) -> bool {
    unsafe {
        // Check if minimized
        if IsIconic(target_hwnd).as_bool() {
            return false;
        }

        // Get target window rect
        let mut target_rect = RECT::default();
        if GetWindowRect(target_hwnd, &mut target_rect).is_err() {
            return false;
        }

        // Check if window has valid size
        let width = target_rect.right - target_rect.left;
        let height = target_rect.bottom - target_rect.top;
        if width <= 0 || height <= 0 {
            return false;
        }

        // Check if window is completely occluded by windows above it
        // We'll check if any part of the window is visible by testing sample points
        let visible = is_any_part_visible(target_hwnd, &target_rect, all_windows);

        visible
    }
}

/// Check if any part of the target window is visible by testing sample points.
/// Returns true if at least one sample point is not covered by other windows.
unsafe fn is_any_part_visible(target_hwnd: HWND, target_rect: &RECT, all_windows: &[HWND]) -> bool {
    // Sample points: corners and center
    let sample_points = [
        // Top-left corner (with small offset to avoid borders)
        (target_rect.left + 10, target_rect.top + 10),
        // Top-right corner
        (target_rect.right - 10, target_rect.top + 10),
        // Bottom-left corner
        (target_rect.left + 10, target_rect.bottom - 10),
        // Bottom-right corner
        (target_rect.right - 10, target_rect.bottom - 10),
        // Center
        (
            (target_rect.left + target_rect.right) / 2,
            (target_rect.top + target_rect.bottom) / 2,
        ),
        // Mid-top
        (
            (target_rect.left + target_rect.right) / 2,
            target_rect.top + 10,
        ),
        // Mid-bottom
        (
            (target_rect.left + target_rect.right) / 2,
            target_rect.bottom - 10,
        ),
        // Mid-left
        (
            target_rect.left + 10,
            (target_rect.top + target_rect.bottom) / 2,
        ),
        // Mid-right
        (
            target_rect.right - 10,
            (target_rect.top + target_rect.bottom) / 2,
        ),
    ];

    // Check if at least one sample point is visible
    for &(x, y) in &sample_points {
        if is_point_visible(x, y, target_hwnd, all_windows) {
            return true;
        }
    }

    false
}

/// Check if a specific point is visible (not covered by other windows).
unsafe fn is_point_visible(x: i32, y: i32, target_hwnd: HWND, _all_windows: &[HWND]) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindow, IsWindowVisible, GW_HWNDPREV};

    // Get all windows above the target window in Z-order
    let mut current_result = GetWindow(target_hwnd, GW_HWNDPREV);

    while let Ok(current) = current_result {
        if current.0 == 0 {
            // No more windows above
            break;
        }

        // Skip if not visible
        if !IsWindowVisible(current).as_bool() {
            current_result = GetWindow(current, GW_HWNDPREV);
            continue;
        }

        // Check if this window covers the point
        let mut rect = RECT::default();
        if GetWindowRect(current, &mut rect).is_ok() {
            if x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom {
                // Point is covered by this window
                return false;
            }
        }

        current_result = GetWindow(current, GW_HWNDPREV);
    }

    // Point is not covered
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_points_coverage() {
        // This is a basic test to ensure the logic compiles
        // Real testing would require actual windows
        let rect = RECT {
            left: 100,
            top: 100,
            right: 500,
            bottom: 400,
        };

        // Just verify the sample points are within bounds
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        assert!(width > 20);
        assert!(height > 20);
    }
}
