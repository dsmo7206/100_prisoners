use rand::{prelude::SliceRandom, thread_rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::atomic::{AtomicUsize, Ordering};

const NUM_TRIES: usize = 1000000;
const NUM_BOXES: usize = 100;
const NUM_PICKS: usize = 50;

type Boxes = Vec<usize>;

fn main() {
    let result = run_sim::<RandomStrategy>();
    println!(
        "Random strategy: {}% success",
        100.0 * result as f32 / NUM_TRIES as f32
    );

    let result = run_sim::<LoopStrategy>();
    println!(
        "Loop strategy:   {}% success",
        100.0 * result as f32 / NUM_TRIES as f32
    );
}

// Simulate all prisoners multiple times to get a percentage success rate
fn run_sim<S: Strategy>() -> usize {
    let success = AtomicUsize::new(0);

    // Runs each group of prisoners in parallel
    (0..NUM_TRIES).into_par_iter().for_each(|_| {
        if run_all::<S>() {
            success.fetch_add(1, Ordering::Relaxed);
        }
    });

    success.load(Ordering::Relaxed)
}

// All prisoners searching for themselves. Returns `true` if all found.
fn run_all<S: Strategy>() -> bool {
    let boxes = make_boxes();

    // Loop over prisoners
    (0..NUM_BOXES).all(|index| run_single::<S>(index, &boxes))
}

/// A single prisoner searching for themselves. Returns `true` if found.
fn run_single<S: Strategy>(index: usize, boxes: &[usize]) -> bool {
    let mut strategy = S::new(index);
    let mut last_inside = None;

    for _ in 0..NUM_PICKS {
        // Get index of box
        let next_index = strategy.next_index(last_inside);
        let found = boxes[next_index];
        if found == index {
            return true;
        }
        last_inside = Some(found);
    }

    false
}

fn make_boxes() -> Boxes {
    let mut rng = thread_rng();

    let mut boxes: Boxes = (0..NUM_BOXES).collect();
    boxes.shuffle(&mut rng);
    boxes
}

trait Strategy {
    fn new(index: usize) -> Self;
    fn next_index(&mut self, last_inside: Option<usize>) -> usize;
}

/// Randomly tests boxes. To do this we just create our own "boxes"
/// (shuffled indices) and pop off the back of the list once per turn
/// to get a new index - and we're guaranteed not to have any repeats.
struct RandomStrategy {
    try_queue: Boxes,
}

impl Strategy for RandomStrategy {
    fn new(_: usize) -> Self {
        Self {
            try_queue: make_boxes(),
        }
    }

    fn next_index(&mut self, _: Option<usize>) -> usize {
        self.try_queue.pop().unwrap()
    }
}

/// Tests boxes via the "optimal" (?) strategy. The first call
/// to `next_index` returns the prisoner's position, and every
/// subsequent call moves to the inside of the last opened box.
struct LoopStrategy {
    index: usize,
}

impl Strategy for LoopStrategy {
    fn new(index: usize) -> Self {
        Self { index }
    }

    fn next_index(&mut self, last_inside: Option<usize>) -> usize {
        match last_inside {
            Some(last_inside) => last_inside,
            None => self.index,
        }
    }
}
