/// `ParallelismStrategy` specifies the method for determining how to parallelize calls that can be parallelized.
#[derive(Clone, Copy, Debug)]
pub enum ParallelismStrategy {
    /// `ParallelismStrategy::Sequential` indicates no parallelism. Execution will always be sequential, using only one execution core.
    Sequential,
    /// `ParallelismStrategy::MiddleIndex` indicates parallelism will be index-based. Logic will not explicitly attempt load balancing.
    MiddleIndex,
}

impl ParallelismStrategy {
    /// Currently returns `ParallelismStrategy::MiddleIndex` but is not guaranteed to in perpetuity.
    pub fn default_par() -> Self {
        ParallelismStrategy::MiddleIndex
    }
}

impl Default for ParallelismStrategy {
    /// Guaranteed to return `ParallelismStrategy::Sequential`
    fn default() -> Self {
        ParallelismStrategy::Sequential
    }
}
