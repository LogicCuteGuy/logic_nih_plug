//! Integration tests for message thread utilities.

use nih_plug_juce::{MessageManager, assert_message_thread};

#[test]
fn test_is_message_thread_returns_bool() {
    // Test that is_message_thread returns a boolean value
    let result = MessageManager::is_message_thread();
    // We can't assert true or false because it depends on JUCE initialization
    // Just verify it returns without crashing
    println!("Is message thread: {}", result);
}

#[test]
fn test_call_async_compiles() {
    // Test that call_async compiles and accepts a closure
    // Note: This may fail if JUCE MessageManager isn't initialized
    // but we're just testing that the API compiles correctly
    
    let result = MessageManager::call_async(|| {
        println!("Callback executed on message thread");
    });
    
    // The call might fail if MessageManager isn't initialized,
    // but that's okay for this test - we're just verifying the API
    match result {
        Ok(()) => println!("Callback posted successfully"),
        Err(e) => println!("Callback posting failed (expected if JUCE not initialized): {}", e),
    }
}

#[test]
fn test_call_async_with_captured_data() {
    // Test that call_async works with closures that capture data
    let value = 42;
    let text = String::from("test");
    
    let result = MessageManager::call_async(move || {
        println!("Captured value: {}, text: {}", value, text);
    });
    
    match result {
        Ok(()) => println!("Callback with captured data posted successfully"),
        Err(e) => println!("Callback posting failed (expected if JUCE not initialized): {}", e),
    }
}

#[test]
fn test_assert_message_thread_macro_compiles() {
    // Test that the assert_message_thread! macro compiles
    // In debug builds, this will panic if not on the message thread
    // In release builds, it compiles to nothing
    
    if MessageManager::is_message_thread() {
        assert_message_thread!();
        println!("On message thread - assertion passed");
    } else {
        println!("Not on message thread - skipping assertion");
    }
}

#[test]
fn test_message_manager_is_send() {
    // MessageManager itself should be Send since it's just a marker type
    fn assert_send<T: Send>() {}
    assert_send::<MessageManager>();
}

#[test]
fn test_message_manager_is_sync() {
    // MessageManager itself should be Sync since it's just a marker type
    fn assert_sync<T: Sync>() {}
    assert_sync::<MessageManager>();
}
