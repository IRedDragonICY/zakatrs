use futures::stream::{FuturesUnordered, StreamExt};
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_futures_unordered_parallelism() {
    let start = Instant::now();
    let delays = vec![100, 100, 100]; // 3 tasks, 100ms each
    
    // Serial execution would take ~300ms
    // Parallel execution should take ~100ms
    
    let mut futures = FuturesUnordered::new();
    for ms in delays {
        futures.push(async move {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            ms
        });
    }
    
    let mut count = 0;
    while let Some(_) = futures.next().await {
        count += 1;
    }
    
    let duration = start.elapsed();
    println!("Duration: {:?}", duration);
    
    assert_eq!(count, 3);
    // Provide a generous buffer for CI environments, but definitely faster than serial
    assert!(duration.as_millis() < 280, "Execution took {:?}, expected < 280ms (Parallel)", duration);
    assert!(duration.as_millis() >= 90, "Execution impossibly fast: {:?}", duration);
}
