use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Matrix {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>
}

impl Matrix {
    pub fn new(width: usize, height: usize, data: Vec<f32>) -> Matrix {
        Matrix { width, height, data }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathResult(i32, f32);

#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
    pub path: Option<Vec<f32>>,
    pub status: String,
    pub message: Option<String>
}