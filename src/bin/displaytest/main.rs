use gbrust::display::display::Display;
use gbrust::display::terminal::TermDisplay;

const W: usize = 160;
const H: usize = 144;

fn main() {
    let mut disp = TermDisplay::new(W, H);

    disp.clear();
    disp.render();

    for y in 0..H {
        for x in 0..W {
            disp.set_pixel(x, y, 1);
            disp.render();
        }
    }
}
