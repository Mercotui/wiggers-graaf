// SPDX-FileCopyrightText: 2026 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

// A strongly typed response to the event
pub enum Claim {
    Claimed,
    Ignored,
}
pub trait AppContext: 'static {
    // All events that the app root needs to dispatch
    type Event;
    // The ID of the component that sourced the event
    type EventSource;

    // Handle an event by dispatching it to needed components, returns the claim status
    fn handle_event(&mut self, source: Self::EventSource, event: &Self::Event) -> Claim;
}
