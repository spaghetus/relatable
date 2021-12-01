# Relatable

Relatable is a web relations scraper. It tries to map out all of the links between websites.

To use Relatable, you need to know about **Constraints**. They determine where Relatable will be allowed to crawl. Without Constraints, Relatable would just keep crawling forever until it crawls every page on the internet, which would be very time- and network-intensive.

Available constraints are:

* `Domain(String)` - The "domain" part of the URL ends with its contents.
* `Path(String)` - The "path" part of the URL begins with its contents.
* `Scheme(String)` - The "scheme" part of the URL is equal to its contents.
* `Depth(Integer)` - The URL must be within the given number of steps from the entry point.
* `None([Constraint])` - None of the contained constraints are satisfied.
* `All([Constraint])` - All of the contained constraints are satisfied.
* `Any([Constraint])` - Any of the contained constraints are satisfied.

To use Constraints, create a file named `relatable.ron` and open it with your favorite text editor. A configuration file will look like this:

```ron
(
	entrypoints: [
		"https://example.com",
	],
	constraints: All([
		Any([
			All([
				Domain("example.com"),
				Path("/"),
			]),
			Depth(4),
		]),
		None([
			Domain("icann.org"),
		]),
	]),
)
```

Once you've configured Relatable to your satisfaction, you can run it, but its output might not seem that machine-readable. Don't worry: we only output machine-readable data on `stdout`, everything else is on `stderr` for ease of separation. You can pipe `stdout` to a file to get a machine-readable file matching the following format:

```python
URL => URL # The URL on the left references the URL on the right.
URL x> CODE # The URL on the left returns the status CODE on the right.
URL ~= HTML # The URL's result is HTML, so we can continue crawling.
URL !> ERROR # The ERROR on the right occurred while trying to obtain the result of the URL on the left.
```