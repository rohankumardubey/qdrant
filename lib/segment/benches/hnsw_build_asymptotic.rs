mod prof;

use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use itertools::Itertools;
use ndarray::Array1;
use rand::rngs::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use segment::fixtures::index_fixtures::{
    random_vector, FakeConditionChecker, TestRawScorerProducer,
};
use segment::index::hnsw_index::graph_layers::GraphLayers;
use segment::index::hnsw_index::point_scorer::FilteredScorer;
use segment::spaces::metric::Metric;
use segment::spaces::simple::{CosineMetric, EuclidMetric};
use segment::types::{Distance, PointOffsetType, ScoreType, VectorElementType};
use std::sync::atomic::Ordering;

const NUM_VECTORS: usize = 5_000;
const DIM: usize = 16;
const M: usize = 16;
const TOP: usize = 10;
const EF_CONSTRUCT: usize = 64;
const EF: usize = 64;
const USE_HEURISTIC: bool = true;

fn build_index(num_vectors: usize) -> (TestRawScorerProducer<CosineMetric>, GraphLayers) {
    let mut rng = thread_rng();

    let vector_holder = TestRawScorerProducer::new(DIM, num_vectors, CosineMetric {}, &mut rng);
    let mut graph_layers = GraphLayers::new(num_vectors, M, M * 2, EF_CONSTRUCT, 10, USE_HEURISTIC);
    let fake_condition_checker = FakeConditionChecker {};
    for idx in 0..(num_vectors as PointOffsetType) {
        let added_vector = vector_holder.vectors[idx as usize].to_vec();
        let raw_scorer = vector_holder.get_raw_scorer(added_vector);
        let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);
        let level = graph_layers.get_random_layer(&mut rng);
        graph_layers.link_new_point(idx, level, &scorer);
    }
    (vector_holder, graph_layers)
}

fn hnsw_build_asymptotic(c: &mut Criterion) {
    let mut group = c.benchmark_group("hnsw-index-build-asymptotic");

    let mut rng = thread_rng();

    let (vector_holder, graph_layers) = build_index(NUM_VECTORS);

    group.bench_function("build-n-search-hnsw", |b| {
        b.iter(|| {
            let fake_condition_checker = FakeConditionChecker {};
            let query = random_vector(&mut rng, DIM);
            let raw_scorer = vector_holder.get_raw_scorer(query);
            let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);
            graph_layers.search(TOP, EF, &scorer);
        })
    });

    for _ in 0..10 {
        let fake_condition_checker = FakeConditionChecker {};
        let query = random_vector(&mut rng, DIM);
        let raw_scorer = vector_holder.get_raw_scorer(query);
        let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);
        graph_layers.search(TOP, EF, &scorer);
        let num_cmps = scorer.score_counter.load(Ordering::SeqCst);
        eprintln!("num_cmps = {:#?}", num_cmps);
    }

    let (vector_holder, graph_layers) = build_index(NUM_VECTORS * 10);

    group.bench_function("build-n-search-hnsw-10x", |b| {
        b.iter(|| {
            let fake_condition_checker = FakeConditionChecker {};
            let query = random_vector(&mut rng, DIM);
            let raw_scorer = vector_holder.get_raw_scorer(query);
            let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);
            graph_layers.search(TOP, EF, &scorer);
        })
    });

    group.bench_function("build-n-search-hnsw-10x-score-point", |b| {
        b.iter(|| {
            let fake_condition_checker = FakeConditionChecker {};
            let query = random_vector(&mut rng, DIM);
            let raw_scorer = vector_holder.get_raw_scorer(query);
            let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);

            let mut points_to_score =
                (0..1500).map(|_| rng.gen_range(0..(NUM_VECTORS * 10)) as u32);
            scorer.score_iterable_points(&mut points_to_score, 1000, |_| {})
        })
    });

    for _ in 0..10 {
        let fake_condition_checker = FakeConditionChecker {};
        let query = random_vector(&mut rng, DIM);
        let raw_scorer = vector_holder.get_raw_scorer(query);
        let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);
        graph_layers.search(TOP, EF, &scorer);
        let num_cmps = scorer.score_counter.load(Ordering::SeqCst);
        eprintln!("num_cmps = {:#?}", num_cmps);
    }
}

#[derive(Clone)]
struct FakeMetric {}

impl Metric for FakeMetric {
    fn distance(&self) -> Distance {
        todo!()
    }

    fn similarity(&self, v1: &[VectorElementType], v2: &[VectorElementType]) -> ScoreType {
        v1[0] + v2[0]
    }

    fn blas_similarity(
        &self,
        v1: &Array1<VectorElementType>,
        v2: &Array1<VectorElementType>,
    ) -> ScoreType {
        v1[0] + v2[0]
    }

    fn preprocess(&self, vector: &[VectorElementType]) -> Option<Vec<VectorElementType>> {
        None
    }
}

fn scoring_vectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("scoring-vector");
    let mut rng = thread_rng();
    let points_per_cycle = 1000;
    let base_num_vectors = 10_000;
    let metric = FakeMetric {};

    let num_vectors = base_num_vectors;
    let vector_holder = TestRawScorerProducer::new(DIM, num_vectors, metric.clone(), &mut rng);

    group.bench_function("score-point", |b| {
        b.iter(|| {
            let fake_condition_checker = FakeConditionChecker {};
            let query = random_vector(&mut rng, DIM);
            let raw_scorer = vector_holder.get_raw_scorer(query);
            let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);

            let mut points_to_score =
                (0..points_per_cycle).map(|_| rng.gen_range(0..num_vectors) as u32);
            scorer.score_iterable_points(&mut points_to_score, points_per_cycle, |_| {})
        })
    });

    let num_vectors = base_num_vectors * 10;
    let vector_holder = TestRawScorerProducer::new(DIM, num_vectors, metric.clone(), &mut rng);

    group.bench_function("score-point-10x", |b| {
        b.iter(|| {
            let fake_condition_checker = FakeConditionChecker {};
            let query = random_vector(&mut rng, DIM);
            let raw_scorer = vector_holder.get_raw_scorer(query);
            let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);

            let mut points_to_score =
                (0..points_per_cycle).map(|_| rng.gen_range(0..num_vectors) as u32);
            scorer.score_iterable_points(&mut points_to_score, points_per_cycle, |_| {})
        })
    });

    let num_vectors = base_num_vectors * 50;
    let vector_holder = TestRawScorerProducer::new(DIM, num_vectors, metric.clone(), &mut rng);

    group.bench_function("score-point-50x", |b| {
        b.iter(|| {
            let fake_condition_checker = FakeConditionChecker {};
            let query = random_vector(&mut rng, DIM);
            let raw_scorer = vector_holder.get_raw_scorer(query);
            let scorer = FilteredScorer::new(&raw_scorer, &fake_condition_checker, None);

            let mut points_to_score =
                (0..points_per_cycle).map(|_| rng.gen_range(0..num_vectors) as u32);
            scorer.score_iterable_points(&mut points_to_score, points_per_cycle, |_| {})
        })
    });
}

fn basic_scoring_vectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("scoring-vector");
    let mut rng = thread_rng();
    let points_per_cycle = 1000;
    let base_num_vectors = 10_000_000;
    let metric = FakeMetric {};

    let num_vectors = base_num_vectors;

    let vectors = (0..num_vectors)
        .map(|_| random_vector(&mut rng, DIM))
        .collect_vec();

    group.bench_function("basic-score-point", |b| {
        b.iter(|| {
            let query = random_vector(&mut rng, DIM);
            let mut points_to_score = (0..points_per_cycle).map(|_| rng.gen_range(0..num_vectors));

            let _s: f32 = points_to_score
                .map(|x| metric.similarity(&vectors[x], &query))
                .sum();
        })
    });

    let num_vectors = base_num_vectors * 2;

    let vectors = (0..num_vectors)
        .map(|_| random_vector(&mut rng, DIM))
        .collect_vec();

    group.bench_function("basic-score-point-10x", |b| {
        b.iter(|| {
            let query = random_vector(&mut rng, DIM);
            let mut points_to_score = (0..points_per_cycle).map(|_| rng.gen_range(0..num_vectors));

            let _s: f32 = points_to_score
                .map(|x| metric.similarity(&vectors[x], &query))
                .sum();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(prof::FlamegraphProfiler::new(100));
    targets = hnsw_build_asymptotic, scoring_vectors, basic_scoring_vectors
}

criterion_main!(benches);
