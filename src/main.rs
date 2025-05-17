use winreg::enums::*;
use winreg::{RegKey, RegValue};
use winreg::enums::RegType;
use std::io;
use std::io::Read;
use inquire::Select;
use is_elevated::is_elevated;

struct Resolution {
    name: &'static str,
    cx: u32,
    cy: u32,
    stride: u32,
}


fn update_if_needed(
    key: &RegKey,
    stride: u32,
    cx: u32,
    cy: u32,
) -> io::Result<bool> {
    let expected = [
        ("Stride", stride),
        ("PrimSurfSize.cx", cx),
        ("PrimSurfSize.cy", cy),
    ];

    let mut changed = false;

    for (name, target) in expected.iter() {
        match key.get_raw_value(name) {
            Ok(existing) => {
                let target_bytes = target.to_le_bytes().to_vec();

                if existing.vtype != RegType::REG_DWORD || existing.bytes != target_bytes {
                    let reg = RegValue {
                        vtype: RegType::REG_DWORD,
                        bytes: target_bytes,
                    };
                    key.set_raw_value(*name, &reg)?;
                    changed = true;
                }
            }
            Err(_) => {
                let reg = RegValue {
                    vtype: RegType::REG_DWORD,
                    bytes: target.to_le_bytes().to_vec(),
                };
                key.set_raw_value(*name, &reg)?;
                changed = true;
            }
        }
    }

    Ok(changed)
}

fn main() -> io::Result<()> {
    println!(
        r#"
 █████╗ ███████╗██╗██╗     ███████╗
██╔══██╗██╔════╝██║██║     ██╔════╝
███████║███████╗██║██║     █████╗
██╔══██║╚════██║██║██║     ██╔══╝
██║  ██║███████║██║███████╗██║
╚═╝  ╚═╝╚══════╝╚═╝╚══════╝╚═╝
        "#
    );

    println!("ANOMALY SYSTEM INPUT LAG FIX");

    if !is_elevated() {
        println!("⛔ Administrator rights required! Run the program as administrator .");
        let _ = std::io::stdin().read(&mut [0u8]).unwrap();
        return Ok(());
    }

    let resolutions = vec![
        Resolution {
            name: "Full HD (1920x1080)",
            cx: 1920,
            cy: 1080,
            stride: 7680,
        },
        Resolution {
            name: "Quad HD 2K (2560x1440)",
            cx: 2560,
            cy: 1440,
            stride: 10240,
        },
    ];

    let names: Vec<&str> = resolutions.iter().map(|r| r.name).collect();

    let selected_name = Select::new("Select Resolution:", names.clone())
        .prompt()
        .expect("Failed to select resolution");

    let selected = resolutions
        .iter()
        .find(|r| r.name == selected_name)
        .expect("Selected resolution not found");

    println!(
        "You selected: {} (Stride = {}, cx = {}, cy = {})",
        selected.name, selected.stride, selected.cx, selected.cy
    );

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let config_path = r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers\Configuration";
    let config_key = hklm.open_subkey_with_flags(config_path, KEY_READ | KEY_WRITE)?;

    let mut simulated_deleted = false;
    let mut settings_changed = false;

    // Удаление SIMULATED_*
    let keys_to_delete: Vec<String> = config_key
        .enum_keys()
        .filter_map(Result::ok)
        .filter(|k| k.starts_with("SIMULATED"))
        .collect();

    for key in &keys_to_delete {
        match config_key.delete_subkey_all(key) {
            Ok(_) => simulated_deleted = true,
            Err(e) => println!("❌ Error to delete: {}: {}", key, e),
        }
    }

    // Обновление конфигураций
    for subkey_name in config_key.enum_keys().filter_map(Result::ok) {
        if let Ok(subkey) = config_key.open_subkey_with_flags(&subkey_name, KEY_READ | KEY_WRITE) {
            if let Ok(subkey_00) = subkey.open_subkey_with_flags("00", KEY_READ | KEY_WRITE) {
                let changed_00 = update_if_needed(&subkey_00, selected.stride, selected.cx, selected.cy)?;
                if changed_00 {
                    settings_changed = true;
                }

                if let Ok(subkey_00_00) = subkey_00.open_subkey_with_flags("00", KEY_READ | KEY_WRITE) {
                    let changed_00_00 = update_if_needed(&subkey_00_00, selected.stride, selected.cx, selected.cy)?;
                    if changed_00_00 {
                        settings_changed = true;
                    }
                }
            }
        }
    }

    // Финальное сообщение
    if settings_changed {
        println!("📝 Resolutions and Stride has been updated.");
    }
    if simulated_deleted {
        println!("🗑️ Deleted keys SIMULATED_*.");
    }

    println!("✅ Done. Ready to launch!\nIdea from video OGjuzy\nCreated by twitch.tv/antonmoler");

    let _ = std::io::stdin().read(&mut [0u8]).unwrap();

    Ok(())
}
