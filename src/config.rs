use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
	pub entrypoints: Vec<String>,
	pub constraints: Constraint,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Constraint {
	/// Restricts the crawler to the given domain and its subdomains.
	Domain(String),
	/// Restricts the crawler to the given path and its children.
	Path(String),
	/// Restricts the crawler to the given scheme.
	/// Only http and https are supported.
	Scheme(String),
	/// Restricts recursion depth.
	Depth(usize),
	/// Requires none of the contained constraints to be satisfied.
	None(Vec<Constraint>),
	/// Requires all contained constraints to be satisfied.
	All(Vec<Constraint>),
	/// Requires any contained constraints to be satisfied.
	Any(Vec<Constraint>),
}

impl Constraint {
	pub fn test(&self, target: (&String, usize)) -> bool {
		let url = url::Url::parse(target.0).unwrap();
		let result = match self {
			Constraint::Domain(domain) => {
				url.domain().map(|d| d.ends_with(domain)).unwrap_or(false)
			}
			Constraint::Path(p) => url.path().starts_with(p),
			Constraint::Scheme(s) => url.scheme() == s,
			Constraint::Depth(max) => target.1 < *max,
			Constraint::None(c) => c.iter().all(|l| !l.test(target)),
			Constraint::All(c) => c.iter().all(|l| l.test(target)),
			Constraint::Any(c) => c.iter().any(|l| l.test(target)),
		};
		#[cfg(feature = "constraint_debug")]
		eprintln!("{:?} {:?} => {}", self, target, result);
		result
	}
}

#[cfg(test)]
mod tests {
	use ron::ser::PrettyConfig;

	const EXAMPLE: &str = include_str!("./example_config.ron");

	#[test]
	async fn serialize() {
		let config = super::Config {
			entrypoints: vec!["https://example.com".to_string()],
			constraints: super::Constraint::Any(vec![
				super::Constraint::All(vec![
					super::Constraint::Domain("example.com".to_string()),
					super::Constraint::Path("/".to_string()),
				]),
				super::Constraint::All(vec![
					super::Constraint::Domain("iana.org".to_string()),
					super::Constraint::Path("/domains/reserved".to_string()),
				]),
			]),
		};
		let config_serialized = ron::ser::to_string_pretty(
			&config,
			PrettyConfig::new()
				// infinitely superior to the default
				// you can fight me on this
				.indentor("\t".to_owned()),
		)
		.unwrap();
		assert_eq!(config_serialized, EXAMPLE);
	}
	#[test]
	async fn deserialize() {
		let config = EXAMPLE;
		let config_de = ron::from_str::<super::Config>(config).unwrap();
		assert_eq!(
			config_de.entrypoints,
			vec!["https://example.com".to_string()]
		);
	}
}
