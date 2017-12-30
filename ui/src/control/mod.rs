mod json;
mod types;
mod paint;
mod mouse;
mod bounds;
mod actions;
mod control;
mod modifier;
mod position;
mod attributes;

pub use self::json::*;
pub use self::types::*;
pub use self::paint::*;
pub use self::mouse::*;
pub use self::bounds::*;
pub use self::actions::*;
pub use self::control::*;
pub use self::modifier::*;
pub use self::position::*;
pub use self::attributes::*;

#[cfg(test)]
mod test {
    use super::*;
    use super::super::diff::*;
    use super::super::image::*;
    use super::super::property::*;
    use super::super::resource_manager::*;

    use canvas::*;
    use std::sync::*;

    #[test]
    fn can_create_button() {
        let button = Control::button();

        assert!(button.control_type() == ControlType::Button);
    }

    #[test]
    fn can_create_label_with_text() {
        let label = Control::label().with("Hello");

        assert!(label.control_type() == ControlType::Label);
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::Text("Hello".to_property())));
    }

    #[test]
    fn can_create_label_with_many_attributes() {
        let label = Control::label().with(("Hello", Bounds::fill_all()));

        assert!(label.control_type() == ControlType::Label);
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::Text("Hello".to_property())));
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::BoundingBox(Bounds::fill_all())));
    }

    #[test]
    fn can_create_container_with_components() {
        let container = Control::container()
            .with(vec![Control::label().with("Hello")]);

        assert!(container.control_type() == ControlType::Container);
        assert!(container.attributes().any(|attr| attr == &ControlAttribute::SubComponents(vec![Control::label().with("Hello")])));
    }

    #[test]
    fn can_find_all_subcontrollers() {
        let container = Control::container()
            .with(vec![
                Control::empty().with_controller("Test1"),
                Control::empty().with_controller("Test2"),
                Control::container().with(vec![
                    Control::empty().with_controller("Test3")
                ])
            ]);
        
        let subcontrollers = container.all_controllers();

        assert!(subcontrollers.len() == 3);
        assert!(subcontrollers.iter().any(|c| c == "Test1"));
        assert!(subcontrollers.iter().any(|c| c == "Test2"));
        assert!(subcontrollers.iter().any(|c| c == "Test3"));
    }

    #[test]
    fn will_only_report_subcontrollers_once() {
        let container = Control::container()
            .with(vec![
                Control::empty().with_controller("Test1"),
                Control::empty().with_controller("Test2"),
                Control::container().with(vec![
                    Control::empty().with_controller("Test1")
                ])
            ]);
        
        let subcontrollers = container.all_controllers();

        assert!(subcontrollers.len() == 2);
        assert!(subcontrollers.iter().any(|c| c == "Test1"));
        assert!(subcontrollers.iter().any(|c| c == "Test2"));
    }

    #[test]
    fn image_equals_self() {
        let resources       = ResourceManager::new();
        let image_resource  = resources.register(Image::Png(Arc::new(InMemoryImageData::new(vec![]))));
        let image           = Control::empty()
            .with(image_resource);

        assert!(!image.is_different(&image));
    }

    #[test]
    fn different_images_are_different() {
        let resources       = ResourceManager::new();
        let image_resource1 = resources.register(Image::Png(Arc::new(InMemoryImageData::new(vec![]))));
        let image_resource2 = resources.register(Image::Png(Arc::new(InMemoryImageData::new(vec![]))));
        let image1          = Control::empty()
            .with(image_resource1);
        let image2          = Control::empty()
            .with(image_resource2);

        assert!(image1.is_different(&image2));
    }

    #[test]
    fn text_attributes_are_different() {
        let text1   = ControlAttribute::Text("Text1".to_property());
        let text2   = ControlAttribute::Text("Text2".to_property());

        assert!(text1 != text2);
    }

    #[test]
    fn controls_with_text_attributes_are_not_equal() {
        let text1   = Control::empty()
            .with("Text1");
        let text2   = Control::empty()
            .with("Text2");

        assert!(text1 != text2);
    }

    #[test]
    fn controls_with_text_attributes_and_other_attributes_are_not_equal() {
        let text1   = Control::empty()
            .with(ControlAttribute::FontSize(12.0))
            .with("Text1")
            .with(Bounds::next_horiz(80.0));
        let text2   = Control::empty()
            .with(ControlAttribute::FontSize(12.0))
            .with("Text2")
            .with(Bounds::next_horiz(80.0));

        assert!(text1 != text2);
    }

    #[test]
    fn controls_with_text_attributes_and_other_attributes_are_same() {
        let text1   = Control::empty()
            .with(ControlAttribute::FontSize(12.0))
            .with("Text1")
            .with(Bounds::next_horiz(80.0));
        let text2   = Control::empty()
            .with(ControlAttribute::FontSize(12.0))
            .with("Text1")
            .with(Bounds::next_horiz(80.0));

        assert!(text1 == text2);
    }

    #[test]
    fn different_text_is_different() {
        let text1   = Control::empty()
            .with("Text1");
        let text2   = Control::empty()
            .with("Text2");

        assert!(text1.is_different(&text2));
    }

    #[test]
    fn different_text_in_subtree_are_not_equal() {
        let text1   = Control::empty()
            .with(vec![
                Control::empty()
                    .with("Text1")
            ]);
        let text2   = Control::empty()
            .with(vec![
                Control::empty()
                    .with("Text2")
            ]);

        assert!(text1 != text2);
    }

    #[test]
    fn different_text_in_subtree_are_not_different() {
        let text1   = Control::empty()
            .with(vec![
                Control::empty()
                    .with("Text1")
            ]);
        let text2   = Control::empty()
            .with(vec![
                Control::empty()
                    .with("Text2")
            ]);

        assert!(!text1.is_different(&text2));
    }

    #[test]
    fn same_text_in_subtree_are_equal() {
        let text1   = Control::empty()
            .with(vec![
                Control::empty()
                    .with("Text1")
            ]);
        let text2   = Control::empty()
            .with(vec![
                Control::empty()
                    .with("Text1")
            ]);

        assert!(text1 == text2);
    }

    #[test]
    fn different_text_is_different_with_other_attributes() {
        let text1   = Control::empty()
            .with("Text1");
        let text2   = Control::empty()
            .with("Text2");

        assert!(text1.is_different(&text2));
    }

    #[test]
    fn canvas_equals_self() {
        let resources       = ResourceManager::new();
        let canvas_resource = resources.register(Canvas::new());
        let canvas1         = Control::canvas();
        let canvas2         = Control::canvas()
            .with(canvas_resource);

        assert!(!canvas1.is_different(&canvas1));
        assert!(!canvas2.is_different(&canvas2));
    }

    #[test]
    fn different_canvases_are_different() {
        let resources        = ResourceManager::new();
        let canvas_resource1 = resources.register(Canvas::new());
        let canvas_resource2 = resources.register(Canvas::new());
        let canvas1          = Control::canvas()
            .with(canvas_resource1);
        let canvas2          = Control::canvas()
            .with(canvas_resource2);

        assert!(canvas1.is_different(&canvas2));
    }
}
