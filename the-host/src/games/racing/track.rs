use bevy::math::Vec3;
use bevy::prelude::{Component, Transform};

/// Which lap you're on (starts at 0)
pub type Lap = usize;
/// Which sector on the race track you're on (starts at 0)
pub type Sector = usize;

#[derive(Component)]
pub enum Tether {
    Anchor(AnchorId),
    Lost
}

#[derive(Component)]
pub struct LapCounter {
    lap: Lap,
    sector: Sector,
    checkpoints: usize
}

#[derive(Copy, Clone)]
pub struct AnchorId(usize);

/// Portmanteau of "track" and "magnet"
/// Collection of points ("anchors") along the center line of the track
/// that are used to keep cars from going outside the track.
#[derive(Component)]
pub struct Tragnet {
    /// These act as a circular buffer
    points: Vec<TragnetAnchor>,
    /// How many checkpoints there are
    checkpoints: usize,
}

#[derive(Debug)]
pub struct TragnetAnchor {
    pub transform: Transform
}

impl Tragnet {
    pub fn new(points: Vec<TragnetAnchor>, num_checkpoints: usize) -> Tragnet {
        Tragnet {
            points,
            checkpoints: num_checkpoints,
        }
    }
    
    pub fn get_current_sector(&self, tether: &Tether) -> Sector {
        if let Tether::Anchor(anchor_id) = tether {
            (anchor_id.0 as f32 / self.points.len() as f32 * self.checkpoints as f32) as usize
        } else {
            0
        }
    }

    pub fn find_nearest(&self, query: Vec3) -> AnchorId {
        if self.points.len() == 0 {
            panic!("Tragnet is empty, cannot find a nearest point.");
        }
        let mut nearest = 0;
        let mut first = true;
        let mut dist = 0.0;
        for (i, p) in self.points.iter().enumerate() {
            let dist_i = p.transform.translation.distance(query);
            if dist_i < dist || first {
                first = false;
                nearest = i;
                dist = dist_i;
            }
        }
        AnchorId(nearest)
    }

    pub fn update_tether(&self, current: &mut Tether, query: Vec3, k: usize) -> i32 {
        let current = match current {
            Tether::Lost => {
                *current = Tether::Anchor(self.find_nearest(query));
                return 0;
            },
            Tether::Anchor(anchor_id) => anchor_id,
        };
        let start_i = (current.0 + self.points.len() - k) % self.points.len();
        let mut nearest = current.0;
        let mut nearest_dist = self.points[nearest].transform.translation.distance(query);
        let mut progress = 0;
        for j in 0..=k * 2 {
            let i = (start_i + j) % self.points.len();
            let dist = self.points[i].transform.translation.distance(query);
            if dist < nearest_dist {
                nearest_dist = dist;
                nearest = i;
                progress = j as i32 - k as i32;
            }
        }
        current.0 = nearest;
        progress
    }
    
    pub fn get_tether_transform(&self, current: &Tether) -> Transform {
        if let Tether::Anchor(anchor_id) = current {
            self.points[anchor_id.0].transform.clone()
        } else {
            panic!("Tether is lost!");
        }
    }
}

impl LapCounter {
    pub fn at_start(num_checkpoints: usize) -> LapCounter {
        LapCounter {
            lap: 0,
            sector: 0,
            checkpoints: num_checkpoints,
        }
    }
    pub fn lap(&self) -> Lap {
        self.lap
    }
    pub fn sector(&self) -> Sector {
        self.sector
    }
    pub fn update_sector(&mut self, sector: Sector) {
        if sector == (self.sector + 1) % self.checkpoints {
            self.sector += 1;
            if self.sector == self.checkpoints {
                self.sector = 0;
                self.lap += 1;
            }
        }
    }
}