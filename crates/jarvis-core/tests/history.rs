use jarvis_core::history::{save, list, Message};
use chrono::Utc;

#[tokio::test]
async fn save_and_list_roundtrip() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::env::set_var("HISTORY_DB_PATH", tmp.path());

    let session_id = "test-session".to_string();
    let msg = Message {
        id: 0,
        session_id: session_id.clone(),
        role: "user".into(),
        content: "hello world".into(),
        created_at: Utc::now(),
    };

    save(msg.clone()).await.unwrap();

    let items = list(&session_id).await.unwrap();
    assert_eq!(items.len(), 1);
    let got = &items[0];
    assert_eq!(got.session_id, msg.session_id);
    assert_eq!(got.role, msg.role);
    assert_eq!(got.content, msg.content);
}

#[tokio::test]
async fn multiple_messages_ordered() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::env::set_var("HISTORY_DB_PATH", tmp.path());

    let session_id = "multi".to_string();

    // create three messages
    for i in 0..3 {
        let msg = Message {
            id: 0,
            session_id: session_id.clone(),
            role: if i % 2 == 0 { "user".into() } else { "assistant".into() },
            content: format!("msg-{i}"),
            created_at: Utc::now(),
        };
        save(msg).await.unwrap();
    }

    let items = list(&session_id).await.unwrap();
    assert_eq!(items.len(), 3);
    for (i, m) in items.iter().enumerate() {
        assert_eq!(m.content, format!("msg-{i}"));
    }
}
