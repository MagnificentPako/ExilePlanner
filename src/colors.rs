type Color = [f32; 4];

macro_rules! from_rgba {
    ($r:literal, $g:literal, $b:literal, $a:literal) => {
        [$r / 255.0, $g / 255.0, $b / 255.0, $a]
    };
}

pub const WHITE: Color = from_rgba!(255.0, 255.0, 255.0, 1.0);
pub const RED: Color = from_rgba!(231.0, 76.0, 60.0, 1.0);
pub const BLUE: Color = from_rgba!(52.0, 152.0, 219.0, 1.0);
pub const GREEN: Color = from_rgba!(46.0, 204.0, 113.0, 1.0);
pub const ORANGE: Color = from_rgba!(230.0, 126.0, 34.0, 1.0);
pub const PURPLE: Color = from_rgba!(155.0, 89.0, 182.0, 1.0);
