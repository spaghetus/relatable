mod config;
mod scrape;

#[cfg_attr(test, macro_use)]
extern crate async_std;

use async_std::fs::read_to_string;
use async_std::sync::{Arc, RwLock};
use async_std::task;
use config::Config;

#[async_std::main]
async fn main() {
	if let Some(proxy) = std::env::var("RELATABLE_PROXY").ok() {
		println!("Waiting for proxy: {}", proxy);
		while scrape::CLIENT
			.get("https://example.com")
			.send()
			.await
			.is_err()
		{
			eprintln!("Proxy not ready yet, retrying in 1 second");
			task::sleep(std::time::Duration::from_secs(1)).await;
		}
	}
	eprintln!("Looking for config");
	let config = read_to_string("./relatable.ron")
		.await
		.expect("Failed to read config");
	eprintln!("Config found");
	let config = ron::from_str::<Config>(&config).expect("Failed to parse config");
	eprintln!("Config read, preparing tables and synchronization primitives");
	let targets: Arc<RwLock<Vec<(String, usize)>>> = Arc::new(RwLock::new(vec![]));
	let mut crawled: Vec<String> = vec![];
	eprintln!("Seeding entry points");
	for url in config.entrypoints.iter() {
		targets.write().await.push((url.to_string(), 0));
	}
	eprintln!("Entering main loop on dispatcher thread");
	loop {
		let mut tasks = vec![];
		// Prune targets
		targets.write().await.retain(|(target, depth)| {
			let allowed = config.constraints.test((target, *depth));
			let fresh = !crawled.contains(target);
			#[cfg(feature = "constraint_debug")]
			{
				let result = allowed && fresh;
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
			}
			#[cfg(not(feature = "constraint_debug"))]
			{
				allowed && fresh
			}
		});
		if targets.read().await.iter().count() == 0 {
			eprintln!("All allowed pages have been visited, exiting");
			break;
		}
		while let Some((target, depth)) = targets.write().await.pop() {
			crawled.push(target.clone());
			tasks.push((
				task::spawn(scrape::scrape(target.clone(), depth, targets.clone())),
				target,
			));
		}
		for (task, _) in tasks {
			task.await;
		}
	}
}
