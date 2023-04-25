use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct PathResult(pub i32, pub f32);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct Result {
    pub path: Option<Vec<PathResult>>,
    pub status: String,
    pub message: Option<String>
}