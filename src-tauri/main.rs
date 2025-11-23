#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use tauri::{ Manager, CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, SystemTrayMenuItem, api::path::document_dir};
use std::{fs, thread, path::Path };

use tauri_plugin_wallpaper::Wallpaper;

//mod commands;
//mod server;

pub fn get_host_path() -> String {
	let dir = document_dir().unwrap();
	let path: String = String::from(dir.to_string_lossy());
	let _dir = path + "\\FOJ";    
    let result1: bool = Path::new(&_dir).is_dir();	
	if result1 == false {
		let _ = fs::create_dir_all(&_dir);
	}
	format!("{}", _dir)
}

pub const PORT_HOST: u16 = 9852;

#[tauri::command]
fn fon(label: &str, show: bool, app: tauri::AppHandle ) {
	let window = app.get_window(label).unwrap();
	if show {
		Wallpaper::attach(&window);
	}else{
		Wallpaper::detach(&window);
	}
}
/*
3 окна
main (якшо його закрити все вилетить)

*/
#[tauri::command]
fn valid(path: &str ) -> bool {
	let empty: bool = Path::new(&path).is_file();
	return empty;
}

#[tauri::command]
fn host_path() -> String {
	let dir = get_host_path();
    format!("{}>{}", dir, PORT_HOST)
}

#[tauri::command]
fn reload_foj(app: tauri::AppHandle) {
	let window = app.get_window("main").unwrap();
    let _ = window.eval("window.location = window.location;");
}

#[tauri::command]
fn devtools_window(app: tauri::AppHandle)    {
    let window = app.get_window("main").unwrap();
	//println!("{:?}", window);
    window.open_devtools()
}

fn main() {
let setting = CustomMenuItem::new("setting".to_string(), "⚙ Настройки");
let exit = CustomMenuItem::new("exit".to_string(), "✖ Вийти");
let tray_menu = SystemTrayMenu::new()
  .add_item(setting)
  .add_native_item(SystemTrayMenuItem::Separator)
  .add_item(exit);
  
let system_tray = SystemTray::new().with_menu(tray_menu);

  tauri::Builder::default()
  .plugin(
    Wallpaper::init(),
  )
  .invoke_handler(
    tauri::generate_handler![
      commands::common::my_custom_command,host_path, reload_foj,fon,valid,devtools_window,
    ],
  )
  .setup(|app| {
    let window = app.get_window("main").unwrap();
	window.hide().unwrap();
    Ok(())
  })
  .system_tray(system_tray)
  .on_system_tray_event(|app, event| match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
          match id.as_str() {
			"setting" => {
				let _window = tauri::WindowBuilder::new(
				  app,
				  "setting",
				  tauri::WindowUrl::App("setting.html".into())				  
				).visible(false).build();
				
				let window = app.get_window("setting").unwrap();
				
				window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
				  width: 800,
				  height: 340,
				})).unwrap();
				let _ = window.set_title("FOJ Настройки");
				window.center().unwrap();
				window.set_resizable(false).unwrap();
			}
            "exit" => {
              let window = app.get_window("main").unwrap();
              Wallpaper::detach(&window);
              window.hide().unwrap();
                std::process::exit(0);
            }
            _ => {}
          }
        }
        _ => {}
      })
  .run(
    tauri::generate_context!()
  ).expect(
    "error while running tauri application"
  );
}