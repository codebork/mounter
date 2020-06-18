mod udisks2;
mod err;
mod config;
mod notices;
mod notifications;
mod manager;
pub use config::Config;
use manager::Manager;
use udisks2::devices::{Block, Drive};

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut udisks2_listener = udisks2::Listener::new();
    let manager = std::rc::Rc::new(std::cell::RefCell::new(Manager::new(config, udisks2::current_state().ok())));

    let manager_clone = manager.clone();
    udisks2_listener.drive_added(move |drive: Drive| {
        let mut manager = manager_clone.borrow_mut();
        manager.new_drive(drive);
    });

    let manager_clone = manager.clone();
    udisks2_listener.block_device_added(move |block_device: Block| {
        let mut manager = manager_clone.borrow_mut();
        manager.new_device(block_device);
    });

    let manager_clone = manager.clone();
    udisks2_listener.object_removed(move |object_path: String| {
        let mut manager = manager_clone.borrow_mut();
        manager.removed_object(object_path);
    });

    udisks2_listener.run()
}

