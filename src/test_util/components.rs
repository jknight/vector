#![deny(missing_docs)]

//! This is a framework for testing components for their compliance with
//! the component spec in `docs/specs/component.md` by capturing emitted
//! internal events and metrics, and testing that they fit the required
//! patterns.

use crate::event::{Metric, MetricValue};
use crate::metrics::{self, Controller};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::collections::HashSet;
use std::env;

thread_local!(
    /// A buffer for recording internal events emitted by a single test.
    static EVENTS_RECORDED: RefCell<HashSet<String>> = RefCell::new(Default::default());
);

/// This struct is used to describe a set of component tests.
pub struct ComponentTests {
    /// The list of event (suffixes) that must be emitted by the component
    events: &'static [&'static str],
    /// The list of counter metrics (with given tags) that must be incremented
    tagged_counters: &'static [&'static str],
    /// The list of counter metrics (with no particular tags) that must be incremented
    untagged_counters: &'static [&'static str],
}

lazy_static! {
    /// The component test specification for all sources
    pub static ref SOURCE_TESTS: ComponentTests = ComponentTests {
        events: &["BytesReceived", "EventsReceived", "EventsSent"],
        tagged_counters: &[
            "component_received_bytes_total",
            "component_received_events_total",
            "component_received_event_bytes_total",
        ],
        untagged_counters: &[
            "component_sent_events_total",
            "component_sent_event_bytes_total",
        ],
    };
}

impl ComponentTests {
    /// Run the test specification, and assert that all tests passed
    pub fn assert(&self, tags: &[&str]) {
        let mut test = ComponentTester::new();
        test.emitted_all_events(self.events);
        test.emitted_all_counters(self.tagged_counters, tags);
        test.emitted_all_counters(self.untagged_counters, &[]);
        if !test.errors.is_empty() {
            panic!(
                "Failed to assert compliance, errors:\n    {}\n",
                test.errors.join("\n    ")
            );
        }
    }
}

/// Initialize the necessary bits needed to run a component test specification.
pub fn init() {
    EVENTS_RECORDED.with(|er| er.borrow_mut().clear());
    // Handle multiple initializations.
    if let Err(error) = metrics::init_test() {
        if error != metrics::Error::AlreadyInitialized {
            panic!("Failed to initialize metrics recorder: {:?}", error);
        }
    }
}

/// Record an emitted internal event. This is somewhat dumb at this
/// point, just recording the pure string value of the `emit!` call
/// parameter. At some point, making all internal events implement
/// `Debug` or `Serialize` might allow for more sophistication here, but
/// this is good enough for these tests. This should only be used by the
/// test `emit!` macro. The `check-events` script will test that emitted
/// events contain the right fields, etc.
pub fn record_internal_event(event: &str) {
    // Remove leading '&'
    // Remove trailing '{fields…}'
    let event = event.strip_prefix('&').unwrap_or(event);
    let event = event.find('{').map(|par| &event[..par]).unwrap_or(event);
    EVENTS_RECORDED.with(|er| er.borrow_mut().insert(event.into()));
}

/// Tests if the given metric contains all the given tag names
fn has_tags(metric: &Metric, names: &[&str]) -> bool {
    metric
        .tags()
        .map(|tags| names.iter().all(|name| tags.contains_key(*name)))
        .unwrap_or_else(|| names.is_empty())
}

/// Standard metrics test environment data
struct ComponentTester {
    metrics: Vec<Metric>,
    errors: Vec<String>,
}

impl ComponentTester {
    fn new() -> Self {
        let mut metrics: Vec<_> = Controller::get().unwrap().capture_metrics().collect();

        if env::var("DEBUG_COMPONENT_COMPLIANCE").is_ok() {
            EVENTS_RECORDED.with(|events| {
                for event in events.borrow().iter() {
                    println!("{}", event);
                }
            });
            metrics.sort_by(|a, b| a.name().cmp(b.name()));
            for metric in &metrics {
                println!("{}", metric);
            }
        }

        let errors = Vec::new();
        Self { metrics, errors }
    }

    fn emitted_all_counters(&mut self, names: &[&str], tags: &[&str]) {
        let tag_suffix = (!tags.is_empty())
            .then(|| format!("{{{}}}", tags.join(",")))
            .unwrap_or_else(String::new);
        for name in names {
            if !self.metrics.iter().any(|m| {
                matches!(m.value(), MetricValue::Counter { .. })
                    && m.name() == *name
                    && has_tags(m, tags)
            }) {
                self.errors
                    .push(format!("Missing metric named {}{}", name, tag_suffix));
            }
        }
    }

    fn emitted_all_events(&mut self, names: &[&str]) {
        for name in names {
            if !EVENTS_RECORDED
                .with(|events| events.borrow().iter().any(|event| event.ends_with(name)))
            {
                self.errors.push(format!("Missing emitted event {}", name));
            }
        }
    }
}
