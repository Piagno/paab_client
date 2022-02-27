use eframe::{run_native, NativeOptions};
use paab_client::Paab;

fn main() {
    let app = Paab::new();
    let win_option = NativeOptions::default();
    run_native(Box::new(app), win_option);
}
