use futures::future::try_join_all;
use futures::StreamExt;
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Instant};

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

async fn get_page(i: usize) -> Vec<usize> {
    let millis = Uniform::from(0..10).sample(&mut rand::thread_rng());
    println!(
        "[{}] # get_page({}) will complete in {} ms",
        START_TIME.elapsed().as_millis(),
        i,
        millis
    );

    sleep(Duration::from_millis(millis)).await;
    println!(
        "[{}] # get_page({}) completed",
        START_TIME.elapsed().as_millis(),
        i
    );

    (10 * i..10 * (i + 1)).collect()
}

#[tokio::main]
async fn main() {
    println!("{:?}", run_parallel(100, 8).await);
    //println!("{:?}", run_buffered(100, 8).await);
}

#[allow(dead_code)]
async fn run_parallel(n: usize, max_concurrent: usize) -> Vec<Vec<usize>> {
    let rt = Builder::new_multi_thread()
        .worker_threads(16)
        .thread_name("run-workers")
        .enable_all()
        .build()
        .unwrap();

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut join_handlers = vec![];
    let joints = build_futures(n);

    for joint in joints {
        let semaphore = semaphore.clone();
        let h = rt.spawn(async move {
            let _segment_prune_permit = semaphore.acquire().await;
            joint.await
        });
        join_handlers.push(h);
    }
    try_join_all(join_handlers).await.unwrap()
}

#[allow(dead_code)]
async fn run_buffered(n: usize, max_concurrent: usize) -> Vec<Vec<usize>> {
    let joints = build_futures(n);
    futures::stream::iter(joints)
        .buffer_unordered(max_concurrent)
        .collect()
        .await
}

fn build_futures(n: usize) -> Vec<impl Future<Output = Vec<usize>>> {
    let mut handlers = vec![];
    for i in 0..n {
        handlers.push(get_page(i));
    }
    handlers
}
