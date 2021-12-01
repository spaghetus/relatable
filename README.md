# Relatable

Relatable is a web relations scraper. It tries to map out all of the links between websites.

To use relatable, you need to know about **Constraints**. They determine where Relatable will be allowed to crawl.

Available constraints are:

* `Domain(String)` - The "domain" part of the URL ends with its contents.
* `Path(String)` - The "path" part of the URL begins with its contents.
* `Scheme(String)` - The "scheme" part of the URL is equal to its contents.
* `Depth(Integer)` - The URL must be within the given number of steps from the entry point.
* `None([Constraint])` - None of the contained constraints are satisfied.
* `All([Constraint])` - All of the contained constraints are satisfied.
* `Any([Constraint])` - Any of the contained constraints are satisfied.