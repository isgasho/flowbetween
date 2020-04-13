use super::*;

use std::time::{Duration};

#[test]
fn group_middle_elements() {
    // Draw six lines, IDs 0,3,6,7,8,9
    let six_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAff4DAAoRnIIRA+PAAAAAA9+PDBAAAAAAAAAAAAAB8PAAAAAAAAAAAAS6PAAAAAAAAAAAAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAA2tPAAAAAAAAAAAAibPAAAAAAAAAAAArfPAAAAAAAAAeCArfPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAICAqqPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAAAArfPAAAAAAAAAAAAibPAAAAAAAAAAAAljPAAAAAAAAAAAAknPAAAAAAAAAAAAB2PAAAAAAAAAAAA2tPAAAAAAAAAAAAzwPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        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
    ";

    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, six_lines);

    // Check the elements in this animation are the ones we expect in the order we expect
    {
        let layer       = animation.get_layer_with_id(1).unwrap();
        let frame       = layer.get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids         = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![0,3,6,7,8,9]);
    }

    // Group the middle three lines
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(6), ElementId::Assigned(7), ElementId::Assigned(8)],
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal))
    ]);

    // Group should have replaced the original elements
    {
        animation.flush_caches();
        let layer           = animation.get_layer_with_id(1).unwrap();
        let frame           = layer.get_frame_at_time(Duration::from_millis(0));
        let elements        = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids             = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![0,3,42,9]);

        let group_element   = elements.iter().filter(|elem| elem.id() == ElementId::Assigned(42)).nth(0).unwrap();
        let group_ids       = match group_element {
            Vector::Group(group)    => group.elements().map(|elem| elem.id().id().unwrap()).collect::<Vec<_>>(),
            _                       => vec![]
        };
        assert!(group_ids == vec![6, 7, 8]);
    }
}

#[test]
fn group_first_elements() {
    // Draw six lines, IDs 0,3,6,7,8,9
    let six_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAff4DAAoRnIIRA+PAAAAAA9+PDBAAAAAAAAAAAAAB8PAAAAAAAAAAAAS6PAAAAAAAAAAAAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAA2tPAAAAAAAAAAAAibPAAAAAAAAAAAArfPAAAAAAAAAeCArfPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAICAqqPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAAAArfPAAAAAAAAAAAAibPAAAAAAAAAAAAljPAAAAAAAAAAAAknPAAAAAAAAAAAAB2PAAAAAAAAAAAA2tPAAAAAAAAAAAAzwPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        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
    ";

    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, six_lines);

    // Group the first three lines
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(0), ElementId::Assigned(3), ElementId::Assigned(6)],
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal))
    ]);

    // Group should have replaced the original elements
    {
        animation.flush_caches();
        let layer           = animation.get_layer_with_id(1).unwrap();
        let frame           = layer.get_frame_at_time(Duration::from_millis(0));
        let elements        = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids             = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![42,7,8,9]);

        let group_element   = elements.iter().filter(|elem| elem.id() == ElementId::Assigned(42)).nth(0).unwrap();
        let group_ids       = match group_element {
            Vector::Group(group)    => group.elements().map(|elem| elem.id().id().unwrap()).collect::<Vec<_>>(),
            _                       => vec![]
        };
        assert!(group_ids == vec![0, 3, 6]);
    }
}

#[test]
fn group_last_elements() {
    // Draw six lines, IDs 0,3,6,7,8,9
    let six_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAff4DAAoRnIIRA+PAAAAAA9+PDBAAAAAAAAAAAAAB8PAAAAAAAAAAAAS6PAAAAAAAAAAAAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAA2tPAAAAAAAAAAAAibPAAAAAAAAAAAArfPAAAAAAAAAeCArfPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAICAqqPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAAAArfPAAAAAAAAAAAAibPAAAAAAAAAAAAljPAAAAAAAAAAAAknPAAAAAAAAAAAAB2PAAAAAAAAAAAA2tPAAAAAAAAAAAAzwPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        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
    ";

    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, six_lines);

    // Group the first three lines
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(7), ElementId::Assigned(8), ElementId::Assigned(9)],
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal))
    ]);

    // Group should have replaced the original elements
    {
        animation.flush_caches();
        let layer           = animation.get_layer_with_id(1).unwrap();
        let frame           = layer.get_frame_at_time(Duration::from_millis(0));
        let elements        = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids             = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![0, 3, 6, 42]);

        let group_element   = elements.iter().filter(|elem| elem.id() == ElementId::Assigned(42)).nth(0).unwrap();
        let group_ids       = match group_element {
            Vector::Group(group)    => group.elements().map(|elem| elem.id().id().unwrap()).collect::<Vec<_>>(),
            _                       => vec![]
        };
        assert!(group_ids == vec![7, 8, 9]);
    }
}

#[test]
fn group_all_elements() {
    // Draw six lines, IDs 0,3,6,7,8,9
    let six_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAff4DAAoRnIIRA+PAAAAAA9+PDBAAAAAAAAAAAAAB8PAAAAAAAAAAAAS6PAAAAAAAAAAAAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAA2tPAAAAAAAAAAAAibPAAAAAAAAAAAArfPAAAAAAAAAeCArfPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAICAqqPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAAAArfPAAAAAAAAAAAAibPAAAAAAAAAAAAljPAAAAAAAAAAAAknPAAAAAAAAAAAAB2PAAAAAAAAAAAA2tPAAAAAAAAAAAAzwPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        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
    ";

    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, six_lines);

    // Group the first three lines
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(0), ElementId::Assigned(3), ElementId::Assigned(6), ElementId::Assigned(7), ElementId::Assigned(8), ElementId::Assigned(9)],
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal))
    ]);

    // Group should have replaced the original elements
    {
        animation.flush_caches();
        let layer           = animation.get_layer_with_id(1).unwrap();
        let frame           = layer.get_frame_at_time(Duration::from_millis(0));
        let elements        = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids             = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![42]);

        let group_element   = elements.iter().filter(|elem| elem.id() == ElementId::Assigned(42)).nth(0).unwrap();
        let group_ids       = match group_element {
            Vector::Group(group)    => group.elements().map(|elem| elem.id().id().unwrap()).collect::<Vec<_>>(),
            _                       => vec![]
        };
        assert!(group_ids == vec![0, 3, 6, 7, 8, 9]);
    }
}

#[test]
fn group_and_ungroup_middle_elements() {
    // Draw six lines, IDs 0,3,6,7,8,9
    let six_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAff4DAAoRnIIRA+PAAAAAA9+PDBAAAAAAAAAAAAAB8PAAAAAAAAAAAAS6PAAAAAAAAAAAAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAA2tPAAAAAAAAAAAAibPAAAAAAAAAAAArfPAAAAAAAAAeCArfPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAICAqqPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAAAArfPAAAAAAAAAAAAibPAAAAAAAAAAAAljPAAAAAAAAAAAAknPAAAAAAAAAAAAB2PAAAAAAAAAAAA2tPAAAAAAAAAAAAzwPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        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
    ";

    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, six_lines);

    // Group the middle three lines
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(6), ElementId::Assigned(7), ElementId::Assigned(8)],
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal))
    ]);


    // Ungroup them again
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(42)],
        ElementEdit::Ungroup)
    ]);

    // Group should have replaced the original elements
    {
        animation.flush_caches();
        let layer           = animation.get_layer_with_id(1).unwrap();
        let frame           = layer.get_frame_at_time(Duration::from_millis(0));
        let elements        = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids             = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![0,3,6,7,8,9]);
    }
}

#[test]
fn delete_element_from_group() {
    // Draw six lines, IDs 0,3,6,7,8,9
    let six_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAff4DAAoRnIIRA+PAAAAAA9+PDBAAAAAAAAAAAAAB8PAAAAAAAAAAAAS6PAAAAAAAAAAAAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAA2tPAAAAAAAAAAAAibPAAAAAAAAAAAArfPAAAAAAAAAeCArfPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAICAqqPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAAAArfPAAAAAAAAAAAAibPAAAAAAAAAAAAljPAAAAAAAAAAAAknPAAAAAAAAAAAAB2PAAAAAAAAAAAA2tPAAAAAAAAAAAAzwPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        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
    ";

    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, six_lines);

    // Group the middle three lines
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(6), ElementId::Assigned(7), ElementId::Assigned(8)],
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal))
    ]);

    // Delete an element that was in the group
    animation.perform_edits(vec![AnimationEdit::Element(vec![ElementId::Assigned(7)], ElementEdit::Delete)]);

    // Group should have replaced the original elements
    {
        animation.flush_caches();
        let layer           = animation.get_layer_with_id(1).unwrap();
        let frame           = layer.get_frame_at_time(Duration::from_millis(0));
        let elements        = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids             = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![0,3,42,9]);

        let group_element   = elements.iter().filter(|elem| elem.id() == ElementId::Assigned(42)).nth(0).unwrap();
        let group_ids       = match group_element {
            Vector::Group(group)    => group.elements().map(|elem| elem.id().id().unwrap()).collect::<Vec<_>>(),
            _                       => vec![]
        };
        assert!(group_ids == vec![6, 8]);
    }
}

#[test]
fn detach_element_from_group() {
    // Draw six lines, IDs 0,3,6,7,8,9
    let six_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAff4DAAoRnIIRA+PAAAAAA9+PDBAAAAAAAAAAAAAB8PAAAAAAAAAAAAS6PAAAAAAAAAAAAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAA2tPAAAAAAAAAAAAibPAAAAAAAAAAAArfPAAAAAAAAAeCArfPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAICAqqPAAAAAAAAAAAALXPAAAAAAAAAAAALXPAAAAAAAAAAAArfPAAAAAAAAAAAAibPAAAAAAAAAAAAljPAAAAAAAAAAAAknPAAAAAAAAAAAAB2PAAAAAAAAAAAA2tPAAAAAAAAAAAAzwPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        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
    ";

    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, six_lines);

    // Group the middle three lines
    animation.perform_edits(vec![AnimationEdit::Element(
        vec![ElementId::Assigned(6), ElementId::Assigned(7), ElementId::Assigned(8)],
        ElementEdit::Group(ElementId::Assigned(42), GroupType::Normal))
    ]);

    // Delete an element that was in the group
    animation.perform_edits(vec![AnimationEdit::Element(vec![ElementId::Assigned(7)], ElementEdit::DetachFromFrame)]);

    // Group should have replaced the original elements
    {
        animation.flush_caches();
        let layer           = animation.get_layer_with_id(1).unwrap();
        let frame           = layer.get_frame_at_time(Duration::from_millis(0));
        let elements        = frame.vector_elements().unwrap().collect::<Vec<_>>();

        let ids             = elements.iter().map(|element| element.id().id().unwrap()).collect::<Vec<_>>();
        assert!(ids == vec![0,3,42,9]);

        let group_element   = elements.iter().filter(|elem| elem.id() == ElementId::Assigned(42)).nth(0).unwrap();
        let group_ids       = match group_element {
            Vector::Group(group)    => group.elements().map(|elem| elem.id().id().unwrap()).collect::<Vec<_>>(),
            _                       => vec![]
        };
        assert!(group_ids == vec![6, 8]);
    }
}
