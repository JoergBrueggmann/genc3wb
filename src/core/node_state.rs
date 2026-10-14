//! The *node states* and the *network mode* the *build system* told last, and the gate of the *store request*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::{NetworkMode, NodeState};

use std::collections::BTreeSet;

// realises FR-229, FR-258
/// The *node states* of every *node* of the network and the *network mode*, as the latest
/// *state response* of the *build system* carried them; none before the first one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeStates {
    /// the *node states*, in the order of the *network file*
    states: Vec<NodeState>,
    /// the *network mode*; `None` before the first *state response* and while no
    /// *build system* runs
    mode: Option<NetworkMode>,
}

impl NodeStates {
    // realises FR-229
    /// Keeps `states` and `mode` in place of those kept before, since every *state response*
    /// carries the state of every *node* and the *network mode* anew.
    pub fn take(&mut self, states: Vec<NodeState>, mode: Option<NetworkMode>) {
        self.states = states;
        self.mode = mode;
    }

    // realises FR-067, FR-229
    /// Forgets the *node states* and the *network mode*: the *build system* that told them runs
    /// no more.
    pub fn clear(&mut self) {
        self.states.clear();
        self.mode = None;
    }

    // realises FR-109, FR-252
    /// Yields whether the *node state* of a *node* counts an error.
    pub fn is_erroneous(&self) -> bool {
        self.states.iter().any(|state| state.errors > 0)
    }

    // realises FR-258, FR-259, FR-262, FR-263
    /// Yields the *network mode* kept, `None` where none is.
    pub fn mode(&self) -> Option<NetworkMode> {
        self.mode
    }

    // realises FR-109, FR-252, FR-263, FR-275, FR-283
    /// Yields whether the *network mode* kept is *active network*.
    pub fn is_active(&self) -> bool {
        self.mode == Some(NetworkMode::Active)
    }

    // realises FR-085, FR-109, FR-235, FR-241, FR-252, FR-262, FR-263, FR-281, FR-282
    /// Yields whether *nodes* are served in the *network mode* kept: it is *valid network*, in
    /// which the *held nodes* run, or *active network*; neither in an *invalid network* nor
    /// where none is kept.
    pub fn serves_nodes(&self) -> bool {
        matches!(
            self.mode,
            Some(NetworkMode::Valid) | Some(NetworkMode::Active)
        )
    }

    // realises FR-259
    /// Yields whether a *network mode* is kept: the *build system* runs and has transmitted a
    /// *state response*, so that the *activation switch* is enabled.
    pub fn has_mode(&self) -> bool {
        self.mode.is_some()
    }

    // realises FR-229
    /// Yields the *node state* of the *node* `name`, `None` where none is kept of it.
    pub fn state_of(&self, name: &str) -> Option<&NodeState> {
        self.states.iter().find(|state| state.name == name)
    }

    // realises FR-229
    /// Yields the *node states* kept, in the order of the *network file*.
    pub fn states(&self) -> &[NodeState] {
        &self.states
    }
}

// realises FR-109, FR-241, FR-252
/// The gate of the *store request*: the requesters, the names of the *nodes* of the
/// *node editors*, whose *store request* is held back until the *node states* count no error
/// and the network is a *valid network* or an *active network*.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoreGate {
    /// the requesters whose *store request* is held back
    held: BTreeSet<String>,
}

impl StoreGate {
    // realises FR-109, FR-252
    /// Asks for the *store request* that follows a change the *node* of `requester` answered
    /// without an error: yields whether it is transmitted now, which it is where `states` admit
    /// it, the requester then held back no more; the requester is held back otherwise.
    pub fn request(&mut self, requester: &str, states: &NodeStates) -> bool {
        if admits(states) {
            self.held.remove(requester);
            return true;
        }
        self.held.insert(requester.to_owned());
        false
    }

    // realises FR-241
    /// Takes the *node states* and the *network mode* that changed: yields the requesters whose
    /// *store request* is transmitted now, which they all are where any is held back and
    /// `states` admit it, none otherwise.
    pub fn states_changed(&mut self, states: &NodeStates) -> Vec<String> {
        if self.held.is_empty() || !admits(states) {
            return Vec::new();
        }
        std::mem::take(&mut self.held).into_iter().collect()
    }

    // realises FR-109, FR-241, FR-252
    /// Gives up the *store request* held back for `requester`: its *node* answered a later
    /// change with an error.
    pub fn cancel(&mut self, requester: &str) {
        self.held.remove(requester);
    }

    // realises FR-241
    /// Gives up every *store request* held back: the *build system* runs no more.
    pub fn clear(&mut self) {
        self.held.clear();
    }

    /// Yields whether a *store request* is held back.
    pub fn is_holding(&self) -> bool {
        !self.held.is_empty()
    }
}

// realises FR-109, FR-241, FR-252
/// Yields whether `states` admit a *store request*: no *node* counts an error, and the network
/// is a *valid network* or an *active network*, in which *nodes* run.
fn admits(states: &NodeStates) -> bool {
    !states.is_erroneous() && states.serves_nodes()
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::store, NetworkEditor::cancel_store, NetworkEditor::activate,
 *                    NetworkEditor::deactivate,
 *                    NetworkEditor::open_node_at, NetworkEditor::open_node_named,
 *                    NetworkEditor::close_node, NetworkEditor::report_arrived */
#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::api_message::NodeMode;

    fn state(name: &str, errors: u64, origins: &[&str]) -> NodeState {
        NodeState {
            name: name.to_owned(),
            errors,
            origins: origins.iter().map(|origin| (*origin).to_owned()).collect(),
            mode: Some(NodeMode::Active),
        }
    }

    /// The *node states* of an *active network*.
    fn states(of: &[(&str, u64)]) -> NodeStates {
        states_in(of, Some(NetworkMode::Active))
    }

    fn states_in(of: &[(&str, u64)], mode: Option<NetworkMode>) -> NodeStates {
        let mut states = NodeStates::default();
        states.take(
            of.iter()
                .map(|(name, errors)| state(name, *errors, &[]))
                .collect(),
            mode,
        );
        states
    }

    fn names(of: &[&str]) -> Vec<String> {
        of.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn no_node_state_counts_no_error_and_is_no_active_network() {
        // FR-109, FR-252: before the first state response
        let none = NodeStates::default();
        assert_eq!(
            (
                none.is_erroneous(),
                none.mode(),
                none.is_active(),
                none.serves_nodes()
            ),
            (false, None, false, false)
        );
    }

    #[test]
    fn latest_node_states_and_network_mode_replace_those_kept_before() {
        // FR-229
        let mut kept = states(&[("a", 1), ("b", 0)]);
        kept.take(
            vec![state("a", 0, &[]), state("c", 0, &["a"])],
            Some(NetworkMode::Valid),
        );
        assert_eq!(
            (
                kept.is_erroneous(),
                kept.state_of("b").cloned(),
                kept.state_of("c").cloned(),
                kept.states().len(),
                kept.mode()
            ),
            (
                false,
                None,
                Some(state("c", 0, &["a"])),
                2,
                Some(NetworkMode::Valid)
            )
        );
    }

    #[test]
    fn network_mode_decides_active_served_and_whether_a_mode_is_kept() {
        // FR-259, FR-263, FR-281: nodes are served in a valid network and in an active network
        let modes = [
            None,
            Some(NetworkMode::Invalid),
            Some(NetworkMode::Valid),
            Some(NetworkMode::Active),
        ];
        let judged: Vec<(bool, bool, bool)> = modes
            .iter()
            .map(|mode| {
                let states = states_in(&[], *mode);
                (states.is_active(), states.serves_nodes(), states.has_mode())
            })
            .collect();
        assert_eq!(
            judged,
            vec![
                (false, false, false),
                (false, false, true),
                (false, true, true),
                (true, true, true)
            ]
        );
    }

    #[test]
    fn an_error_of_any_node_makes_the_node_states_erroneous() {
        // FR-109, FR-252
        assert!(states(&[("a", 0), ("b", 2), ("c", 0)]).is_erroneous());
    }

    #[test]
    fn origins_alone_count_no_error() {
        // FR-109: a node downstream of an error counts none itself
        let mut kept = NodeStates::default();
        kept.take(vec![state("c", 0, &["b"])], Some(NetworkMode::Active));
        assert!(!kept.is_erroneous());
    }

    #[test]
    fn cleared_node_states_are_none_and_have_no_network_mode() {
        // FR-067, FR-229
        let mut kept = states(&[("a", 1)]);
        kept.clear();
        assert_eq!(kept, NodeStates::default());
    }

    #[test]
    fn store_is_held_back_while_no_node_is_served_and_transmitted_once_nodes_are() {
        // FR-109, FR-241, FR-252: no node counts an error, no network mode is kept, then an
        // invalid network, then a valid network, in which the held nodes run
        let mut gate = StoreGate::default();
        let requested = gate.request("a", &states_in(&[("a", 0)], None));
        let still_invalid =
            gate.states_changed(&states_in(&[("a", 0)], Some(NetworkMode::Invalid)));
        let valid = gate.states_changed(&states_in(&[("a", 0)], Some(NetworkMode::Valid)));
        assert_eq!(
            (requested, still_invalid, valid, gate.is_holding()),
            (false, vec![], names(&["a"]), false)
        );
    }

    #[test]
    fn store_is_transmitted_at_once_in_a_valid_network_and_in_an_active_network() {
        // FR-109: the nodes of the node windows run as held nodes in a valid network
        let mut gate = StoreGate::default();
        assert_eq!(
            (
                gate.request("a", &states_in(&[("a", 0)], Some(NetworkMode::Valid))),
                gate.request("a", &states(&[("a", 0)])),
                gate.is_holding()
            ),
            (true, true, false)
        );
    }

    #[test]
    fn store_waits_for_both_no_error_and_served_nodes() {
        // FR-241: the network becomes active while a node still counts an error
        let mut gate = StoreGate::default();
        let requested = gate.request("a", &states_in(&[("a", 1)], Some(NetworkMode::Invalid)));
        let active_but_erroneous = gate.states_changed(&states(&[("a", 1)]));
        let clear = gate.states_changed(&states(&[("a", 0)]));
        assert_eq!(
            (requested, active_but_erroneous, clear),
            (false, vec![], names(&["a"]))
        );
    }

    #[test]
    fn store_is_held_back_while_a_node_counts_an_error_and_transmitted_once_none_does() {
        // FR-109, FR-241, FR-252
        let mut gate = StoreGate::default();
        let requested = gate.request("a", &states(&[("a", 0), ("b", 1)]));
        let still_erroneous = gate.states_changed(&states(&[("a", 1), ("b", 0)]));
        let clear = gate.states_changed(&states(&[("a", 0), ("b", 0)]));
        let again = gate.states_changed(&states(&[("a", 0), ("b", 0)]));
        assert_eq!(
            (requested, still_erroneous, clear, again, gate.is_holding()),
            (false, vec![], names(&["a"]), vec![], false)
        );
    }

    #[test]
    fn stores_of_several_requesters_are_held_back_and_transmitted_together() {
        // FR-241: two node editors asked while a node counted an error; one store answers both
        let mut gate = StoreGate::default();
        let erroneous = states(&[("a", 1), ("b", 0)]);
        let requested = (
            gate.request("b", &erroneous),
            gate.request("a", &erroneous),
            gate.request("b", &erroneous),
        );
        let clear = gate.states_changed(&states(&[("a", 0), ("b", 0)]));
        assert_eq!(
            (requested, clear, gate.is_holding()),
            ((false, false, false), names(&["a", "b"]), false)
        );
    }

    #[test]
    fn clear_node_states_transmit_nothing_where_no_store_is_held_back() {
        // FR-241
        let mut gate = StoreGate::default();
        assert_eq!(
            gate.states_changed(&states(&[("a", 0)])),
            vec![] as Vec<String>
        );
    }

    #[test]
    fn cancelled_store_of_one_requester_leaves_the_others_held_back() {
        // FR-241: a later change of one node was answered with an error
        let mut gate = StoreGate::default();
        let erroneous = states(&[("b", 1)]);
        let _ = gate.request("a", &erroneous);
        let _ = gate.request("b", &erroneous);
        gate.cancel("a");
        assert_eq!(
            (gate.is_holding(), gate.states_changed(&states(&[("b", 0)]))),
            (true, names(&["b"]))
        );
    }

    #[test]
    fn cleared_gate_holds_nothing_back() {
        // FR-241: the build system was started anew
        let mut gate = StoreGate::default();
        let _ = gate.request("a", &states(&[("b", 1)]));
        gate.clear();
        assert_eq!(
            (gate.is_holding(), gate.states_changed(&states(&[("b", 0)]))),
            (false, vec![])
        );
    }

    #[test]
    fn later_request_while_the_states_are_clear_ends_the_holding_of_that_requester() {
        // FR-109, FR-252
        let mut gate = StoreGate::default();
        let _ = gate.request("a", &states(&[("b", 1)]));
        assert_eq!(
            (gate.request("a", &states(&[("b", 0)])), gate.is_holding()),
            (true, false)
        );
    }
}
