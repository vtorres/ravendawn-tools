use winres::WindowsResource;

fn main() {
    let mut res = WindowsResource::new();

    res.set_icon("./assets/icon.ico").set("InternalName", "Ravendawn - Exp Tracker").set_language(0x0409);

    res.compile().unwrap();
}