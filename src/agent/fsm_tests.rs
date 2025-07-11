#[cfg(test)]
mod tests {
    use super::fsm::{AgentContext, AgentEvent, AgentState, AgentStateMachine};
    use crate::llm::{ChatMessage, Tool};
    use proptest::prelude::*;
    use std::collections::HashMap;

    // Test helper function to create a basic FSM
    fn create_test_fsm() -> AgentStateMachine {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "test message".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];

        AgentStateMachine::new(messages, vec![], HashMap::new())
    }

    // Test helper function to create FSM with tools
    fn create_test_fsm_with_tools() -> AgentStateMachine {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "test message".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];

        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: crate::llm::Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({}),
            },
        }];

        AgentStateMachine::new(messages, tools, HashMap::new())
    }

    #[test]
    fn test_initial_state() {
        let fsm = create_test_fsm();
        assert_eq!(*fsm.current_state(), AgentState::ReadyToCallLlm);
        assert!(!fsm.is_terminal());
    }

    #[test]
    fn test_valid_transition_process_input() {
        let mut fsm = create_test_fsm();
        let result = fsm.transition(AgentEvent::ProcessInput);

        assert!(result.is_ok());
        assert_eq!(*fsm.current_state(), AgentState::AwaitingLlmResponse);
    }

    #[test]
    fn test_invalid_transition_returns_error() {
        let mut fsm = create_test_fsm();
        let result = fsm.transition(AgentEvent::LlmRespondedWithContent);

        assert!(result.is_err());
        assert_eq!(*fsm.current_state(), AgentState::ReadyToCallLlm);
    }

    #[test]
    fn test_terminal_states() {
        let mut fsm = create_test_fsm();
        assert!(!fsm.is_terminal());

        // Transition to Done state
        fsm.transition(AgentEvent::ProcessInput).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::AwaitingLlmResponse);

        fsm.transition(AgentEvent::LlmRespondedWithContent).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::Done);
        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_error_state_is_terminal() {
        let mut fsm = create_test_fsm();
        fsm.transition(AgentEvent::ProcessInput).unwrap();
        
        // Force to error state
        fsm.transition(AgentEvent::ErrorOccurred).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::Error);
        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_fsm_complete_tool_execution_cycle() {
        let mut fsm = create_test_fsm_with_tools();

        // Complete cycle: Ready -> Awaiting -> ExecutingTools -> Ready -> Awaiting -> Done
        fsm.transition(AgentEvent::ProcessInput).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::AwaitingLlmResponse);

        fsm.transition(AgentEvent::LlmRequestedTools).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::ExecutingTools);

        fsm.transition(AgentEvent::ToolsExecutionCompleted).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::ReadyToCallLlm);

        fsm.transition(AgentEvent::ProcessInput).unwrap();
        fsm.transition(AgentEvent::LlmRespondedWithContent).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::Done);
        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_tool_execution_failure() {
        let mut fsm = create_test_fsm_with_tools();

        fsm.transition(AgentEvent::ProcessInput).unwrap();
        fsm.transition(AgentEvent::LlmRequestedTools).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::ExecutingTools);

        // Tool execution fails
        fsm.transition(AgentEvent::ToolsExecutionFailed).unwrap();
        assert_eq!(*fsm.current_state(), AgentState::Error);
        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_context_message_handling() {
        let mut fsm = create_test_fsm();
        
        // Add a message to context
        let new_message = ChatMessage {
            role: "assistant".to_string(),
            content: "response".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        
        fsm.context.messages.push(new_message);
        assert_eq!(fsm.context.messages.len(), 2);
        assert_eq!(fsm.context.messages[1].role, "assistant");
    }

    #[test]
    fn test_context_tool_call_handling() {
        let mut fsm = create_test_fsm_with_tools();
        
        // Add tool call requests
        let tool_call = crate::mcp::McpToolCallRequest {
            name: "test_tool".to_string(),
            arguments: std::collections::HashMap::new(),
        };
        
        fsm.context.pending_tool_calls.push(tool_call);
        assert_eq!(fsm.context.pending_tool_calls.len(), 1);
        
        // Prepare tool execution should return the pending calls
        let tool_calls = fsm.prepare_tool_execution();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "test_tool");
    }

    #[test]
    fn test_get_final_content() {
        let mut fsm = create_test_fsm();
        
        // Add assistant message
        fsm.context.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: "Final response".to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        
        assert_eq!(fsm.get_final_content(), "Final response");
    }

    #[test]
    fn test_get_final_content_no_assistant_message() {
        let fsm = create_test_fsm();
        assert_eq!(fsm.get_final_content(), "No response available");
    }

    #[test]
    fn test_error_handling() {
        let mut fsm = create_test_fsm();
        
        fsm.context.last_error = Some("Test error".to_string());
        assert_eq!(fsm.get_last_error(), Some("Test error".to_string()));
    }

    // Property-based tests
    prop_compose! {
        fn valid_event_sequence()(
            events in prop::collection::vec(
                prop_oneof![
                    Just(AgentEvent::ProcessInput),
                    Just(AgentEvent::LlmRespondedWithContent),
                    Just(AgentEvent::LlmRequestedTools),
                    Just(AgentEvent::ToolsExecutionCompleted),
                    Just(AgentEvent::ToolsExecutionFailed),
                    Just(AgentEvent::ErrorOccurred),
                ],
                1..20
            )
        ) -> Vec<AgentEvent> {
            events
        }
    }

    proptest! {
        #[test]
        fn test_fsm_never_transitions_to_invalid_state(
            events in valid_event_sequence()
        ) {
            let mut fsm = create_test_fsm();

            for event in events {
                let _ = fsm.transition(event); // May succeed or fail

                // Property: FSM should never be in an undefined state
                match fsm.current_state() {
                    AgentState::ReadyToCallLlm |
                    AgentState::AwaitingLlmResponse |
                    AgentState::ExecutingTools |
                    AgentState::Done |
                    AgentState::Error => {
                        // Valid states - property holds
                    }
                }
            }
        }

        #[test]
        fn test_terminal_states_stay_terminal(
            events in valid_event_sequence()
        ) {
            let mut fsm = create_test_fsm();

            // Force to terminal state (Done)
            fsm.transition(AgentEvent::ProcessInput).unwrap();
            fsm.transition(AgentEvent::LlmRespondedWithContent).unwrap();
            prop_assert!(fsm.is_terminal());
            let terminal_state = *fsm.current_state();

            // Property: Once terminal, should stay terminal and not change state
            for event in events {
                let _ = fsm.transition(event);
                prop_assert!(fsm.is_terminal());
                prop_assert_eq!(*fsm.current_state(), terminal_state);
            }
        }

        #[test]
        fn test_error_state_stays_terminal(
            events in valid_event_sequence()
        ) {
            let mut fsm = create_test_fsm();

            // Force to error state
            fsm.transition(AgentEvent::ProcessInput).unwrap();
            fsm.transition(AgentEvent::ErrorOccurred).unwrap();
            prop_assert_eq!(*fsm.current_state(), AgentState::Error);
            prop_assert!(fsm.is_terminal());

            // Property: Error state should remain terminal
            for event in events {
                let _ = fsm.transition(event);
                prop_assert_eq!(*fsm.current_state(), AgentState::Error);
                prop_assert!(fsm.is_terminal());
            }
        }

        #[test]
        fn test_message_count_only_increases(
            num_messages in 1usize..50
        ) {
            let mut fsm = create_test_fsm();
            let initial_count = fsm.context.messages.len();

            // Add messages
            for i in 0..num_messages {
                fsm.context.messages.push(ChatMessage {
                    role: "test".to_string(),
                    content: format!("message {}", i),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }

            // Property: Message count should only increase
            prop_assert_eq!(fsm.context.messages.len(), initial_count + num_messages);
        }
    }

    // Edge case tests
    #[test]
    fn test_empty_tool_calls_list() {
        let fsm = create_test_fsm();
        let tool_calls = fsm.prepare_tool_execution();
        assert!(tool_calls.is_empty());
    }

    #[test]
    fn test_multiple_tool_execution_results() {
        let mut fsm = create_test_fsm_with_tools();
        
        let results = vec![
            crate::mcp::McpToolCallResponse {
                content: vec![crate::mcp::McpContent::Text {
                    text: "Result 1".to_string(),
                }],
                is_error: false,
            },
            crate::mcp::McpToolCallResponse {
                content: vec![crate::mcp::McpContent::Text {
                    text: "Result 2".to_string(),
                }],
                is_error: false,
            },
        ];
        
        fsm.add_tool_execution_results(results);
        assert_eq!(fsm.context.tool_call_results.len(), 2);
        assert!(!fsm.context.tool_call_results[0].is_error);
        assert!(!fsm.context.tool_call_results[1].is_error);
    }
}