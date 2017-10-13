use super::property::*;
use super::binding::*;

///
/// Represents a viewmodel for a control subtree. ViewModels are
/// used for controls which can be edited and need to have values
/// stored by key in the controller
///
pub trait ViewModel {
    /// Retrieves a property
    fn get_property(&self, property_name: &str) -> Box<Bound<Property>>;

    /// Updates a property
    fn set_property(&self, property_name: &str, new_value: Property);

    /// Retrieves the names of all of the properties in this item
    fn get_property_names(&self) -> Vec<String>;
}

pub struct NullViewModel;

impl NullViewModel {
    pub fn new() -> NullViewModel {
        NullViewModel
    }
}

impl ViewModel for NullViewModel {
    fn get_property(&self, _property_name: &str) -> Box<Bound<Property>> {
        Box::new(bind(Property::Nothing))
    }

    fn set_property(&self, _property_name: &str, _new_value: Property) { 
    }

    fn get_property_names(&self) -> Vec<String> {
        vec![]
    }
}
