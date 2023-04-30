use gbrust::display::curses::CursesDisplay;
use gbrust::display::display::{Color, Display};

const W: usize = 160;
const H: usize = 144;

fn main() {
    let mut disp = CursesDisplay::new(W, H, 60);

    disp.clear();
    disp.render();

    for y in 0..H {
        for x in 0..W {
            disp.set_pixel(x, y, x as Color);
            disp.render();
        }
    }
}
