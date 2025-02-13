package metadata

components: transforms: metric_to_log: {
	title: "Metric to Log"

	description: """
		Converts a metric event into a log event, which can be useful for sending metrics
		to log-support downstream components.
		"""

	classes: {
		commonly_used: true
		development:   "stable"
		egress_method: "stream"
		stateful:      false
	}

	features: {
		convert: {}
	}

	support: {
		targets: {
			"aarch64-unknown-linux-gnu":      true
			"aarch64-unknown-linux-musl":     true
			"armv7-unknown-linux-gnueabihf":  true
			"armv7-unknown-linux-musleabihf": true
			"x86_64-apple-darwin":            true
			"x86_64-pc-windows-msv":          true
			"x86_64-unknown-linux-gnu":       true
			"x86_64-unknown-linux-musl":      true
		}
		requirements: []
		warnings: []
		notices: []
	}

	configuration: {
		host_tag: {
			common:      true
			description: "Tag key that identifies the source host."
			required:    false
			warnings: []
			type: string: {
				default: "hostname"
				examples: ["host", "hostname"]
				syntax: "literal"
			}
		}
		timezone: configuration._timezone
	}

	input: {
		logs: false
		metrics: {
			counter:      true
			distribution: true
			gauge:        true
			histogram:    true
			set:          true
			summary:      true
		}
	}

	examples: [
		{
			title: "Metric To Log"

			configuration: {
				host_tag: "host"
			}

			input: metric: {
				kind:      "absolute"
				name:      "histogram"
				timestamp: "2020-08-01T21:15:47+00:00"
				tags: {
					host: "my.host.com"
					code: "200"
				}
				histogram: {
					buckets: [
						{upper_limit: 1.0, count: 10},
						{upper_limit: 2.0, count: 20},
					]
					count: 30
					sum:   50.0
				}
			}
			output: log: {
				name:      "histogram"
				timestamp: "2020-08-01T21:15:47+00:00"
				host:      "my.host.com"
				tags: {
					"code": "200"
				}
				kind: "absolute"
				histogram: {
					buckets: [
						{
							"count":       10
							"upper_limit": 1.0
						},
						{
							"count":       20
							"upper_limit": 2.0
						},
					]
					count: 30
					sum:   50.0
				}
			}
		},
	]

	how_it_works: {}

	telemetry: metrics: {
		processing_errors_total: components.sources.internal_metrics.output.metrics.processing_errors_total
	}
}
