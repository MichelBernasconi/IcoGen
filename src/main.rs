#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

use rfd::FileDialog;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use IcoGen::{generate_icons, IcoGenConfig};

struct AppState {
    input_files: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    
    let state = Rc::new(RefCell::new(AppState {
        input_files: Vec::new(),
        output_dir: None,
    }));
    
    // Browse Images
    let ui_weak = ui.as_weak();
    let state_clone = state.clone();
    ui.on_browse_images(move || {
        if let Some(paths) = FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
            .pick_files()
        {
            state_clone.borrow_mut().input_files = paths.clone();
            if let Some(ui) = ui_weak.upgrade() {
                if paths.len() == 1 {
                    ui.set_selected_files_text(paths[0].file_name().unwrap_or_default().to_string_lossy().to_string().into());
                } else {
                    ui.set_selected_files_text(format!("{} files selected", paths.len()).into());
                }
            }
        }
    });

    // Browse Output
    let ui_weak = ui.as_weak();
    let state_clone = state.clone();
    ui.on_browse_output(move || {
        if let Some(path) = FileDialog::new().pick_folder() {
            state_clone.borrow_mut().output_dir = Some(path.clone());
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_output_dir_text(path.display().to_string().into());
            }
        }
    });

    // Generate Icons
    let ui_weak = ui.as_weak();
    let state_clone = state.clone();
    ui.on_generate_icons(move || {
        let ui = match ui_weak.upgrade() {
            Some(ui) => ui,
            None => return,
        };
        
        let state = state_clone.borrow();
        let input_files = state.input_files.clone();
        let output_dir = match &state.output_dir {
            Some(d) => d.clone(),
            None => {
                ui.set_log_text("Error: No output folder selected.".into());
                return;
            }
        };
        
        if input_files.is_empty() {
            ui.set_log_text("Error: No image selected.".into());
            return;
        }

        ui.set_is_processing(true);
        ui.set_log_text(format!("Starting processing of {} files...\n", input_files.len()).into());
        
        let config = IcoGenConfig {
            input_files,
            output_dir,
            profile: ui.get_profile().to_string(),
            custom_sizes: ui.get_custom_sizes().to_string(),
            format: ui.get_output_format().to_string(),
            remove_bg: ui.get_remove_bg(),
            bg_tolerance: ui.get_bg_tolerance() as u8,
        };

        let ui_handle = ui_weak.clone();
        
        thread::spawn(move || {
            let send_log = {
                let ui_handle = ui_handle.clone();
                move |msg: String| {
                    let ui_handle = ui_handle.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            let mut current_log = ui.get_log_text().to_string();
                            current_log.push_str(&msg);
                            current_log.push('\n');
                            ui.set_log_text(current_log.into());
                        }
                    });
                }
            };

            generate_icons(config, send_log);

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle.upgrade() { ui.set_is_processing(false); }
            });
        });
    });

    ui.run()
}
