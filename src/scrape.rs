use std::time::Duration;

use async_std::sync::{Arc, RwLock};
use scraper::{Html, Selector};

lazy_static::lazy_static! {
	pub static ref CLIENT: reqwest::Client = {
		let mut client = reqwest::Client::builder()
			.timeout(Duration::from_secs(10))
			.user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:94.0) Gecko/20100101 Firefox/94.0");
		if let Some(proxy) = std::env::var("RELATABLE_PROXY").ok() {
			let proxy = reqwest::Proxy::all(proxy).unwrap();
			client = client.proxy(proxy);
		}
		client.build().unwrap()
	};
}

pub async fn scrape(target: String, depth: usize, targets: Arc<RwLock<Vec<(String, usize)>>>) {
	let response = CLIENT.head(&target).send().await;
	// If there is an error
	if let Err(error) = response {
		println!("{} !> {}", target, error);
		return;
	}
	let response = response.unwrap();
	println!("{} x> {}", target, response.status().as_u16());
	// If the MIME type is HTML
	let new_urls: Vec<String> = if response.headers()[reqwest::header::CONTENT_TYPE]
		.to_str()
		.unwrap()
		.starts_with("text/html")
	{
		println!("{} ~= HTML", target);
		// Fetch
		let response = CLIENT.get(&target).send().await;
		// If there is an error
		if let Err(error) = response {
			println!("{} !> {}", target, error);
			return;
		}
		// If the status code indicates an error
		let response = response.unwrap();
		// Create the HTML parser and load metadata
		let document = response.text().await.unwrap();
		let document = Html::parse_document(&document);
		// Get the base URL
		let base = document
			.select(&Selector::parse("base").unwrap())
			.next()
			.map(|base| base.value().attr("href"))
			.flatten()
			.unwrap_or(&target);
		let base = match url::Url::parse(base) {
			Ok(base) => base,
			Err(_) => target.parse::<url::Url>().unwrap(),
		};
		// Find all URL references
		let sel = Selector::parse("*").unwrap();
		document
			.select(&sel)
			.filter_map(|node| match node.value().name() {
				"link" => node.value().attr("href"),
				"a" => node.value().attr("href"),
				"cite" => node.value().attr("href"),
				"audio" => node.value().attr("src"),
				"area" => node.value().attr("href"),
				"img" => node.value().attr("src"),
				"track" => node.value().attr("src"),
				"video" => node.value().attr("src"),
				"source" => node.value().attr("src"),
				_ => node.value().attr("href"),
			})
			.map(|u| u.to_string())
			.filter_map(|u| {
				let url = url::Url::parse(&u);
				url.ok().map(|url| {
					if url.cannot_be_a_base() {
						base.join(&u).unwrap().to_string()
					} else {
						u
					}
				})
			})
			.inspect(|url| {
				println!("{} => {}", target, url);
			})
			.collect()
	} else {
		return;
	};
	// Write the new URLs to the targets vector
	if !new_urls.is_empty() {
		let mut targets = targets.write().await;
		for url in new_urls {
			if !targets.iter().any(|(u, _)| u == &url) {
				targets.push((url, depth + 1));
			}
		}
	}
}
