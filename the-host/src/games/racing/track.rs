use bevy::math::Vec3;
use bevy::prelude::{Component, Transform};

#[derive(Component)]
pub enum Tether {
    Anchor(AnchorId),
    Lost
}

#[derive(Copy, Clone)]
pub struct AnchorId(usize);

// circular buffer of points
#[derive(Component)]
pub struct Tragnet {
    points: Vec<TragnetAnchor>
}

#[derive(Debug)]
pub struct TragnetAnchor {
    pub transform: Transform
}

impl Tragnet {
    pub fn new(points: Vec<TragnetAnchor>) -> Tragnet {
        Tragnet {
            points
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