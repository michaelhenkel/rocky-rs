
use cursive::views::{Dialog, TextView};
pub struct Gui{}

impl Gui {
    pub fn new() -> Gui {
        Gui{}
    }
    pub fn run(&self) {
        let mut siv = cursive::default();
        siv.add_layer(
            Dialog::around(TextView::new("Hello Dialog!"))
                .title("Cursive")
                .button("Quit", |s| s.quit()),
        );
        siv.run();
    }
}