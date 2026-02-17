//! Integration tests for IPC communication
//!
//! Tests cover:
//! - Protocol serialization/deserialization
//! - Request/response handling

use cliff_watch_core::protocol::{Request, Response};

/// Default socket path for testing
const DEFAULT_SOCKET_PATH: &str = "/tmp/cliff-watch-test.sock";

/// Helper function to clean up socket file
fn cleanup_socket(socket_path: &str) {
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_protocol_request_serialization() {
    let requests = vec![
        Request::GetStatus,
        Request::GetMetrics,
        Request::GetTicket { cost: 10.0 },
        Request::Ping,
        Request::GetWitness { reset: false },
    ];

    for request in requests {
        let serialized = serde_json::to_string(&request);
        assert!(serialized.is_ok(), "Request should be serializable: {:?}", request);

        let deserialized: Result<Request, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok(), "Request should be deserializable: {:?}", request);
    }
}

#[tokio::test]
async fn test_protocol_response_serialization() {
    let responses = vec![
        Response::Status {
            is_running: true,
            uptime_secs: 100,
            events_captured: 50,
        },
        Response::Metrics {
            ldlj: 0.5,
            entropy: 1.0,
            throughput: 2.0,
            human_score: 0.8,
            coupling: 0.7,
            battery_level: 75.0,
            focus_time_mins: 30.0,
            edit_bursts: 10,
            is_focused: true,
            zkp_proof: Some("proof".to_string()),
            score_history: vec![0.7, 0.8, 0.9],
        },
        Response::Ticket {
            success: true,
            signature: Some(vec![1, 2, 3]),
            message: "Ticket issued".to_string(),
        },
        Response::Witness {
            data: "{\"test\": \"data\"}".to_string(),
        },
        Response::Pong,
        Response::Error("Test error".to_string()),
    ];

    for response in responses {
        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok(), "Response should be serializable: {:?}", response);

        let deserialized: Result<Response, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok(), "Response should be deserializable: {:?}", response);
    }
}

#[tokio::test]
async fn test_protocol_roundtrip() {
    let original_requests = vec![
        Request::GetStatus,
        Request::GetMetrics,
        Request::GetTicket { cost: 25.5 },
        Request::Ping,
        Request::GetWitness { reset: true },
    ];

    for request in original_requests {
        let serialized = serde_json::to_vec(&request).expect("Request should serialize");
        let deserialized: Request = serde_json::from_slice(&serialized).expect("Request should deserialize");
        
        let reserialized = serde_json::to_vec(&deserialized).expect("Response should reserialize");
        
        assert_eq!(serialized, reserialized, "Request should survive roundtrip: {:?}", request);
    }
}
