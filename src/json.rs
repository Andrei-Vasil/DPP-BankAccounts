use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Operation {
    pub from: usize,
    pub to: usize,
    pub amount: f32
}
