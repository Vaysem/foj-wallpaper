#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
extern crate winreg;
use serde_json::Value;
use winreg::{enums::*, RegKey};
use std::{io, path::Path, env, process, process::Command, time::Duration, thread::sleep, sync::{Mutex, Arc} };

use serde_json;

use sysinfo::{System};
pub type SharedSystem = Arc<Mutex<System>>;

use sys_locale::get_locale;

use mouse_position::mouse_position::Mouse;

use tauri::{ AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, };
use reqwest::{Client, header::{HeaderMap, HeaderValue}};

use tauri_plugin_wallpaper::Wallpaper;
use once_cell::sync::Lazy;


const REG_PATH_AUTORUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "foj-wallpaper";

const REG_PATH_LANGUAGE: &str = r"Software\dev2\fog-wallpapers";
const LANGUAGE_FILE: &str = include_str!("../../src/language.json");

static MAIN_PID: Lazy<u32> = Lazy::new(|| {
    process::id()
});

#[tauri::command]
async fn get_command(app: AppHandle, on: bool) -> String {
    let mut window = app.get_window("main-fon").unwrap();
    window.eval("if(typeof context_command == 'object'){let a=[];for(let n in context_command){a.push(n+(typeof context_command[n]=='string'?'🗲'+ context_command[n]:''))};location.hash = a.join(';')};").unwrap();

    sleep(Duration::from_millis(100));
    window = app.get_window("main-fon").unwrap();
    let mut menu_item: String = "".to_string();
    if window.url().fragment().is_some() {
        menu_item = window.url().fragment().expect("REASON").to_string();
    }
    if !on {
        window.eval("location.hash = ''").unwrap();
    }
    menu_item.to_string()
}

#[tauri::command]
async fn parse_url(app: AppHandle, url: String) -> String {
    if app.get_window("main-fon").is_none() {
        return Default::default();
    }
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0").unwrap());
    headers.insert("Accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"));
    headers.insert("Accept-Language", HeaderValue::from_static("ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("Priority", HeaderValue::from_static("u=0, i")); 
	
    let client = Client::builder()
    .default_headers(headers)
    .build().unwrap();

    let send_foj = app.get_window("main-fon").unwrap();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        send_foj.eval(&format!("typeof parse_url=='function'&&parse_url(`{}`)", text.replace("`", "\\`"))).unwrap();
                        text.to_string()
                    },
                    Err(e) => {send_foj.eval(&format!("typeof parse_url=='function'&&parse_url('Error reading: {}')", e)).unwrap(); e.to_string()},
                }
            } else {
                send_foj.eval(&format!("typeof parse_url=='function'&&parse_url('HTTP error: {}')", response.status())).unwrap();
                response.status().to_string()
            }
        }
        Err(e) => {send_foj.eval(&format!("typeof parse_url=='function'&&parse_url('Network request error: {}')", e)).unwrap(); e.to_string()},
    }
}

#[tauri::command]
fn mouse_pos(app: AppHandle) -> String {
    if app.get_window("main-fon").is_none() {
        return Default::default();
    }
    let mouse_pos = Mouse::get_mouse_position();
    match mouse_pos {
        Mouse::Position { x, y } => {
            let send_foj = app.get_window("main-fon").unwrap();
            let jscom = format!("typeof set_mouse=='function'&&set_mouse({},{})", x, y);
            send_foj.eval(&jscom).unwrap();
            format!("x={}, y={}", x, y)
        }
        Mouse::Error => todo!(),
    }
}

#[tauri::command]
fn fon(label: &str, show: bool, context_command: Option<bool>, app: AppHandle) {
    let window = app.get_window(label).unwrap();
    if context_command.unwrap_or(false) {
        window.eval("window.onload=()=>{if(typeof context_command == 'object'){let a=[];for(let n in context_command){a.push(n+(typeof context_command[n]=='string'?'🗲'+ context_command[n]:''))};location.hash = a.join(';')}};").unwrap();
    }
    if show {
        Wallpaper::attach(&window);
    } else {
        Wallpaper::detach(&window);
    }
}

#[tauri::command]
fn valid(path: &str) -> bool {
    Path::new(&path).is_file()
}

#[tauri::command]
fn window_eval(cname: &str, app: AppHandle) {
    let window = app.get_window("main-fon").unwrap();
    let s = &("context_command['".to_owned() + cname + "']();");
    window.eval(s).unwrap();
}

#[tauri::command]
fn reload_foj(app: AppHandle) {
    let window = app.get_window("main").unwrap();
    window.eval("location.reload()").unwrap();
}

#[tauri::command]
fn get_autoload() -> bool {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = match hklm.open_subkey_with_flags(REG_PATH_AUTORUN, KEY_READ) {
        Ok(k) => k,
        Err(_) => {
            return false
        },
    };
    match run_key.get_value::<String, _>(APP_NAME) {
        Ok(_path) => {
            return true
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return false
            } else {
                return false
            }
        }
    }
}

#[tauri::command]
fn set_autoload(on: bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(REG_PATH_AUTORUN, KEY_WRITE).unwrap();

    let curr = env::current_exe().unwrap();
    let path: String  = String::from(curr.to_string_lossy());
    
    if on {
        let _ = run_key.set_value(APP_NAME, &path);
    } else {
        if get_autoload(){
            run_key.delete_value(APP_NAME).unwrap();
        }
    }
}

#[tauri::command]
fn devtools_window(label: &str, app: AppHandle) {
    let window = app.get_window(label).unwrap();
    window.open_devtools()
}

#[tauri::command]
fn sys_info(app: AppHandle, state: tauri::State<SharedSystem>) -> String {
    if app.get_window("main-fon").is_none() {
        return Default::default();
    }
    
    let mut sys = state.lock().unwrap();
    let mut ram: f32;

    sys.refresh_all();
    let all_ram = sys.total_memory() as f32;
    let free_ram = sys.free_memory() as f32;
    ram = 100.0 / all_ram * free_ram;
    ram = 100.0 - ram;

    let send_foj = app.get_window("main-fon").unwrap();
    let jscom = format!("typeof sys_info=='function'&&sys_info({{\"cpu\":{},\"ram\":{}}})", sys.global_cpu_usage(), ram);
    send_foj.eval(&jscom).unwrap();
    format!("{{\"cpu\":{},\"ram\":{}}}", sys.global_cpu_usage(), ram)
}

#[tauri::command]
fn run_command(path: &str) -> String {
    if path.trim() == "" { return serde_json::from_str(r#"{"error":"not command"}"#).expect("Can't parse json"); }
    let child = Command::new(path.trim()).spawn();    
    match child {
        Ok(_) => format!("{{\"success\":\"{}\"}}", path),
        Err(e) => format!("{{\"error\":\"{}\"}}", e),
    }
}

#[tauri::command]
fn get_language_code() -> String {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let (key, disp) = hklm.create_subkey(REG_PATH_LANGUAGE).expect("error 209");

    match disp {
        REG_CREATED_NEW_KEY => {
            let locale: String = get_locale().unwrap_or_else(|| "en-US".to_string());
            let mut lang_code:String = locale.split('-').next().unwrap_or("en").to_string();
            let json: Value = serde_json::from_str(LANGUAGE_FILE).expect("Can't parse json");
        
            if json[lang_code.clone()].is_null() {        
                lang_code = "en".to_string();      
            }

            let _ = key.set_value("language", &lang_code);
            lang_code
        },
        REG_OPENED_EXISTING_KEY => {
            let cod: String = key.get_value("language").expect("en"); 
            cod
        }
    }
}

#[tauri::command]
fn set_language_code(app: AppHandle, language: &str) -> bool{
    let json: Value = serde_json::from_str(LANGUAGE_FILE).expect("Can't parse json");

    match json[language.trim()].is_object() {
        true => {
            let trey = app.tray_handle();
            
            let localize = json[language.trim()].clone();

            let _ = trey.get_item("setting").set_title(format!("🛠 {}", localize["setting"].as_str().expect("Setting")));
            let _ = trey.get_item("reload").set_title(format!("⟲ {}", localize["reload_html"].as_str().expect("Reload_html")));
            let _ = trey.get_item("doc").set_title(format!("ⓘ {}", localize["doc"].as_str().expect("Documentation")));
            let _ = trey.get_item("exit").set_title(format!("✖ {}", localize["exit"].as_str().expect("Exit")));


            let hklm = RegKey::predef(HKEY_CURRENT_USER);
            let (key, _disp) = hklm.create_subkey(REG_PATH_LANGUAGE).expect("error 235");
            let _ = key.set_value("language", &language.trim());
            true
        },
        false => false
    }
}

#[tauri::command]
fn exit_program(app: AppHandle, pid: u32) {
    if *MAIN_PID == pid {
        let window = app.get_window("main").unwrap();
        window.eval(&"localStorage.setItem('old_pid',0)").unwrap();        
        app.exit(0);
    }
}

fn main() {
    let lang_code = get_language_code();
    let json: Value = serde_json::from_str(&LANGUAGE_FILE).expect("Can't parse json");   
    let loaclization: Value = json[lang_code.clone()].clone();


    let current_exe_name = env::current_exe().expect("REASON")
        .file_name()
        .ok_or_else(|| "Not name.".to_string()).expect("REASON")
        .to_string_lossy()
        .to_lowercase();
    
    if current_exe_name != "foj-wallpaper.exe" {
        process::exit(1);
    }

    let mut initial_sys = System::new_all();
    
    let args: Vec<String> = env::args().collect();
    let mut dropped_file_path: String = "".to_string();
    let mut old_pid = 0;

    if args.len() > 1 && valid(&args[1]) {
        dropped_file_path.push_str(&args[1]);
        initial_sys.refresh_all(); 
        for (pid, process) in initial_sys.processes() {
            if process.name().to_string_lossy().to_lowercase() == current_exe_name && pid.as_u32() != *MAIN_PID {
                old_pid = pid.as_u32();
                break;
            }
        }        
    }else{
        let count_run: i16 = initial_sys.processes().values()
            .filter(|p| p.name() == "foj-wallpaper.exe")
            .count() as i16;
        if count_run > 1 {
            tauri::api::dialog::blocking::message(
                None::<&tauri::Window>, 
                loaclization["duble_run_title"].as_str().expect("duble_run_title"), 
                loaclization["duble_run_text"].as_str().expect("duble_run_text")
            );      
            process::exit(0);
        }   
    }
    
    let shared_sys = Arc::new(Mutex::new(initial_sys));
    
    let setting = CustomMenuItem::new("setting".to_string(), format!("🛠 {}", loaclization["setting"].as_str().expect("Setting")));
    let reload = CustomMenuItem::new("reload".to_string(), format!("⟲ {}", loaclization["reload_html"].as_str().expect("Retting")));
    let doc = CustomMenuItem::new("doc".to_string(), format!("ⓘ {}", loaclization["doc"].as_str().expect("Documentation")));
    let exit = CustomMenuItem::new("exit".to_string(), format!("✖ {}", loaclization["exit"].as_str().expect("Exit")));
    let tray_menu = SystemTrayMenu::new()
        .add_item(setting)
        .add_item(reload)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(doc)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(exit);
        
    let system_tray = SystemTray::new().with_menu(tray_menu);    

    tauri::Builder::default()
        .plugin(Wallpaper::init())
        .invoke_handler(tauri::generate_handler![
            get_autoload,
            set_autoload,
            reload_foj,
            fon,
            valid,
            devtools_window,
            window_eval,
            mouse_pos,
            run_command,
            get_command,
            sys_info,
            parse_url,
            set_language_code,
            exit_program,
        ])
        .setup(move |app| {
            let window = app.get_window("main").unwrap();
            window.hide().unwrap();
            if !dropped_file_path.is_empty() {
                window.eval(&format!("localStorage.setItem('path','{}');", dropped_file_path.replace("\\", "\\\\"))).unwrap();
            }
            if old_pid > 0 {
                window.eval(&format!("localStorage.setItem('old_pid',{});", old_pid)).unwrap();
            }
            let _ = app.tray_handle().set_tooltip(APP_NAME);
            
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                if app.get_window("main-fon").is_none() {
                    return;
                }
                if app.get_window("menu_page").is_none() == false {
                    let close_me = app.get_window("menu_page").unwrap();
                    close_me.close().unwrap();
                }
                let window = app.get_window("main-fon").unwrap();

                if window.url().fragment().is_some() {
                    let variable_menu = window.url().fragment().expect("variable left menu").to_string();
                    let count_item: u32 = variable_menu.split(";").count().try_into().unwrap();
                    let height_menu = 37 + (26 * count_item);

                    let _window = tauri::WindowBuilder::new(
                        app,
                        "menu_page",
                        tauri::WindowUrl::App(
                            ("menu_page.html#".to_owned() + &variable_menu).into(),
                        ),
                    )
                    .visible(false)
                    .decorations(false)
                    .always_on_top(true)
                    .transparent(true)
                    .title(" ")
                    .build();

                    let left_menu = app.get_window("menu_page").unwrap();
                    left_menu
                        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                            width: 180,
                            height: height_menu,
                        }))
                        .unwrap();

                    let mouse_pos = Mouse::get_mouse_position();
                    match mouse_pos {
                        Mouse::Position { x, y } => {
                            left_menu
                                .set_position(tauri::PhysicalPosition {
                                    x: x - 178,
                                    y: y - (height_menu as i32) + 2,
                                })
                                .unwrap();
                        }
                        Mouse::Error => {
                            left_menu.center().unwrap();
                        }
                    }
                    left_menu.set_skip_taskbar(true).unwrap();
                    left_menu.set_resizable(false).unwrap();
                    left_menu.set_focus().unwrap();
                    left_menu.show().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "setting" => {
                    let _window = tauri::WindowBuilder::new(
                        app,
                        "setting",
                        tauri::WindowUrl::App("setting.html".into()),
                    )
                    .visible(false)
                    .decorations(false)
                    .transparent(true)
                    .title(format!("FOJ wallpaper {}", loaclization["setting"].as_str().expect("Setting")))
                    .build();

                    let window = app.get_window("setting").unwrap();

                    window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                            width: 900,
                            height: 450,
                        }))
                        .unwrap();
                    window.center().unwrap();
                    window.set_resizable(false).unwrap();
                    let convert = format!("window.lang_code = '{}';", lang_code.clone());
                    window.eval(&convert).unwrap();
                }
                "reload" => {
                    let close = app.get_window("main-fon").unwrap();
                    close.close().unwrap();
                    let window = app.get_window("main").unwrap();
                    window.eval("location.reload()").unwrap();
                }
                "doc" => {
                    let _window = tauri::WindowBuilder::new(
                        app,
                        "doc",
                        tauri::WindowUrl::App("doc.html".into()),
                    )
                    .visible(false)
                    .build();
                    let window = app.get_window("doc").unwrap();
                    window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                            width: 1000,
                            height: 600,
                        }))
                        .unwrap();
                    window.center().unwrap();
                    let convert = format!("window.lang_code = '{}';", lang_code.clone());
                    window.eval(&convert).unwrap();
                }
                "exit" => {
                    app.exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .manage(shared_sys)
        .run(tauri::generate_context!())
        .expect("error while running foj application");
}