use craft_crypto::{emoji::EmojiScript, scratch::ScratchBlocks};
use soul_agent::{AgentError, AgentHandle};

#[test]
fn emoji_round_trip() {
    let agent = AgentHandle::spawn(1, None).unwrap();
    let name = agent.load_named("🔢7 🔢6 ✖️ ↩️", &EmojiScript).unwrap();
    assert_eq!(agent.call(&name, vec![]), Ok(42));
    agent.shutdown();
}

#[test]
fn scratch_blocks_round_trip() {
    let source = r#"{
        "type":"const",
        "args":[40],
        "next":{
            "type":"const",
            "args":[2],
            "next":{
                "type":"add",
                "args":[],
                "next":{"type":"return","args":[],"next":null}
            }
        }
    }"#;
    let agent = AgentHandle::spawn(2, None).unwrap();
    let name = agent.load_named(source, &ScratchBlocks).unwrap();
    assert_eq!(agent.call(&name, vec![]), Ok(42));
    agent.shutdown();
}

#[test]
fn shutdown_is_clean() {
    let agent = AgentHandle::spawn(3, None).unwrap();
    let observer = agent.clone();
    agent.shutdown();
    assert!(!observer.is_alive());
    assert_eq!(observer.call("missing", vec![]), Err(AgentError::AgentDead));
}
