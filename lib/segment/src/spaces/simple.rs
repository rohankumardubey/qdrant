extern crate blas_src;

use ndarray::Array1;

use crate::types::{Distance, ScoreType, VectorElementType};

use super::metric::Metric;

#[derive(Clone)]
pub struct DotProductMetric {}

#[derive(Clone)]
pub struct CosineMetric {}

#[derive(Clone)]
pub struct EuclidMetric {}

impl Metric for EuclidMetric {
    fn distance(&self) -> Distance {
        Distance::Euclid
    }

    fn similarity(&self, v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        let s: ScoreType = v1
            .iter()
            .copied()
            .zip(v2.iter().copied())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        -s.sqrt()
    }

    fn blas_similarity(
        &self,
        v1: &Array1<VectorElementType>,
        v2: &Array1<VectorElementType>,
    ) -> ScoreType {
        let s: ScoreType = v1
            .iter()
            .copied()
            .zip(v2.iter().copied())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        -s.sqrt()
    }

    fn preprocess(&self, _vector: &[VectorElementType]) -> Option<Vec<VectorElementType>> {
        None
    }
}

impl Metric for DotProductMetric {
    fn distance(&self) -> Distance {
        Distance::Dot
    }

    fn similarity(&self, v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        v1.iter().zip(v2).map(|(a, b)| a * b).sum()
    }

    fn blas_similarity(
        &self,
        v1: &Array1<VectorElementType>,
        v2: &Array1<VectorElementType>,
    ) -> ScoreType {
        v1.dot(v2)
    }

    fn preprocess(&self, _vector: &[VectorElementType]) -> Option<Vec<VectorElementType>> {
        None
    }
}

impl Metric for CosineMetric {
    fn distance(&self) -> Distance {
        Distance::Cosine
    }

    fn similarity(&self, v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        v1.iter().zip(v2).map(|(a, b)| a * b).sum()
    }

    fn blas_similarity(
        &self,
        v1: &Array1<VectorElementType>,
        v2: &Array1<VectorElementType>,
    ) -> ScoreType {
        v1.dot(v2)
    }

    fn preprocess(&self, vector: &[VectorElementType]) -> Option<Vec<VectorElementType>> {
        let mut length: f32 = vector.iter().map(|x| x * x).sum();
        length = length.sqrt();
        let norm_vector = vector.iter().map(|x| x / length).collect();
        Some(norm_vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_preprocessing() {
        let metric = CosineMetric {};
        let res = metric.preprocess(&[0.0, 0.0, 0.0, 0.0]);
        eprintln!("res = {:#?}", res);
    }
}
