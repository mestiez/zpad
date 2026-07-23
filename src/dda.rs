pub struct DDA {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    step: f32,
    i: i32,
}

impl DDA {
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> DDA {
        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let step = if dx.abs() >= dy.abs() { dx.abs() } else { dy.abs() };

        DDA {
            x: x1 as f32,
            y: y1 as f32,
            dx: dx / step,
            dy: dy / step,
            step,
            i: 0,
        }
    }
}

impl Iterator for DDA {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i as f32 >= self.step {
            return None;
        }

        self.x += self.dx;
        self.y += self.dy;
        self.i += 1;

        Some((
            self.x.round() as i32,
            self.y.round() as i32
        ))
    }
}
