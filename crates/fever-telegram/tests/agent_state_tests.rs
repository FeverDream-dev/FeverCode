use fever_telegram::state::AgentState;

#[test]
fn test_agent_state_transitions() {
    let mut s = AgentState::Idle;
    // simple state progression
    assert_eq!(s, AgentState::Idle);
    s = AgentState::Running;
    assert_eq!(s, AgentState::Running);
    s = AgentState::Paused;
    assert_eq!(s, AgentState::Paused);
    s = AgentState::Stopped;
    assert_eq!(s, AgentState::Stopped);
}
