mod config;
mod scrape;

#[macro_use]
extern crate async_std;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

use async_std::channel::Receiver;
use async_std::channel::Sender;
use async_std::fs::read_to_string;
use async_std::future::timeout;
use async_std::prelude::*;
use async_std::sync::{Arc, RwLock};
use async_std::task;
use config::Config;

#[async_std::main]
async fn main() {
	eprintln!("Looking for config");
	let config = read_to_string("./relatable.ron")
		.await
		.expect("Failed to read config");
	eprintln!("Config found");
	let config = ron::from_str::<Config>(&config).expect("Failed to parse config");
	eprintln!("Config read, preparing tables and synchronization primitives");
	let targets: Arc<RwLock<Vec<(String, usize)>>> = Arc::new(RwLock::new(vec![]));
	let mut crawled: Vec<String> = vec![];
	let (send_new_targets, recv_new_targets): (Sender<()>, Receiver<()>) =
		async_std::channel::unbounded();
	eprintln!("Seeding entry points");
	for url in config.entrypoints.iter() {
		targets.write().await.push((url.to_string(), 0));
	}
	eprintln!("Entering main loop on dispatcher thread");
	loop {
		let mut tasks = vec![];
		// Prune targets
		targets.write().await.retain(|(target, depth)| {
			let allowed = config.constraints.test((&target, *depth));
			let fresh = !crawled.contains(&target);
			let result = allowed && fresh;
			#[cfg(feature = "constraint_debug")]
			if !result {
				eprintln!(
					"Removing {} from targets because {}",
					target,
					if !allowed {
						"it is outside our constraints."
					} else {
						"we've already crawled it."
					}
				);
				eprintln!("{:?}", crawled);
			}
			result
		});
		if targets.read().await.iter().count() == 0 {
			eprintln!("All allowed pages have been visited, exiting");
			break;
		}
		while let Some((target, depth)) = targets.write().await.pop() {
			crawled.push(target.clone());
			tasks.push((
				task::spawn(scrape::scrape(
					target.clone(),
					depth,
					targets.clone(),
					send_new_targets.clone(),
				)),
				target,
			));
		}
		for (task, target) in tasks {
			task.await;
		}
	}
}
