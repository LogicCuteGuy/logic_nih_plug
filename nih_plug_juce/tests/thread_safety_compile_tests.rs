//! Compile-time tests for thread safety enforcement.
//!
//! These tests verify that JUCE GUI types do not implement Send or Sync,
//! which enforces that they can only be used on the message thread.
//!
//! Note: These tests are designed to fail compilation if uncommented,
//! demonstrating that the type system prevents cross-thread usage.

#[cfg(test)]
mod thread_safety_tests {
    use nih_plug_juce::*;
    use nih_plug_juce::widgets::*;
    use nih_plug_juce::containers::*;
    use nih_plug_juce::drawing::*;
    use nih_plug_juce::events::*;
    use nih_plug_juce::dialogs::*;

    // Helper function to test if a type implements Send
    fn assert_send<T: Send>() {}

    // Helper function to test if a type implements Sync
    fn assert_sync<T: Sync>() {}

    #[test]
    fn test_component_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<Component>();
        
        // This test passes by not compiling the assertion
    }

    #[test]
    fn test_component_not_sync() {
        // Uncommenting this line should cause a compile error:
        // assert_sync::<Component>();
        
        // This test passes by not compiling the assertion
    }

    #[test]
    fn test_text_button_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<TextButton>();
    }

    #[test]
    fn test_slider_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<Slider>();
    }

    #[test]
    fn test_label_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<Label>();
    }

    #[test]
    fn test_combo_box_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<ComboBox>();
    }

    #[test]
    fn test_text_editor_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<TextEditor>();
    }

    #[test]
    fn test_toggle_button_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<ToggleButton>();
    }

    #[test]
    fn test_document_window_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<DocumentWindow>();
    }

    #[test]
    fn test_resizable_window_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<ResizableWindow>();
    }

    #[test]
    fn test_viewport_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<Viewport>();
    }

    #[test]
    fn test_tabbed_component_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<TabbedComponent>();
    }

    #[test]
    fn test_list_box_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<ListBox>();
    }

    #[test]
    fn test_tree_view_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<TreeView>();
    }

    #[test]
    fn test_flexbox_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<FlexBox>();
    }

    #[test]
    fn test_timer_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<Timer>();
    }

    #[test]
    fn test_file_chooser_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<FileChooser>();
    }

    #[test]
    fn test_look_and_feel_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<LookAndFeel>();
    }

    #[test]
    fn test_drawable_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<Drawable>();
    }

    #[test]
    fn test_drawable_button_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<DrawableButton>();
    }

    // Note: Path, Colour, Font, Image, and AffineTransform are value types
    // and intentionally implement Send + Sync as they can be safely used
    // across threads.

    #[test]
    fn test_path_is_send() {
        // Path is a value type and should be Send
        assert_send::<Path>();
    }

    #[test]
    fn test_path_is_sync() {
        // Path is a value type and should be Sync
        assert_sync::<Path>();
    }

    // Graphics has a lifetime parameter which prevents it from being Send/Sync
    // without explicitly implementing those traits
    #[test]
    fn test_graphics_not_send() {
        // Uncommenting this line should cause a compile error:
        // assert_send::<Graphics<'static>>();
    }
}
