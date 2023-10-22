#[derive(Debug)]
pub struct Region {
    pub center_x: u32,
    pub center_y: u32,
    pub radius: u32,
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl Region {
    pub fn new(center_x: u32, center_y: u32, radius: u32) -> Self {
        let i32x = center_x as i32;
        let i32y = center_y as i32;
        let i32radius = radius as i32;

        Self {
            center_x,
            center_y,
            radius,
            min_x: i32x - i32radius,
            min_y: i32y - i32radius,
            max_x: i32x + i32radius,
            max_y: i32y + i32radius,
        }
    }

    pub fn real_origin_x(&self) -> u32 {
        if self.min_x < 0 {
            0
        } else {
            self.min_x as u32
        }
    }

    pub fn real_origin_y(&self) -> u32 {
        if self.min_y < 0 {
            0
        } else {
            self.min_y as u32
        }
    }

    pub fn real_width(&self) -> u32 {
        (self.max_x as u32) - self.real_origin_x()
    }

    pub fn real_height(&self) -> u32 {
        (self.max_y as u32) - self.real_origin_y()
    }

    pub fn real_center_x(&self) -> u32 {
        self.center_x - self.real_origin_x()
    }

    pub fn real_center_y(&self) -> u32 {
        self.center_y - self.real_origin_y()
    }
}
