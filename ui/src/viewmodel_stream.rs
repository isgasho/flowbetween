use super::control::*;
use super::viewmodel::*;
use super::controller::*;
use super::viewmodel_update::*;

use binding::*;

use futures::*;
use futures::stream;

use std::sync::*;
use std::collections::{HashMap, VecDeque};

///
/// A stream of updates from the viewmodels for a controller and its subcontrollers
///
pub struct ViewModelUpdateStream {
    /// The root controller whose updates we should return
    root_controller: Weak<dyn Controller>,

    /// Stream of updates from the root controller
    controller_stream: Box<dyn Stream<Item=Control, Error=()>>,

    /// Updates for the controller viewmodel
    controller_viewmodel_updates: Option<Box<dyn Stream<Item=ViewModelChange, Error=()>>>,

    /// The streams for the subcontrollers
    sub_controllers: HashMap<String, ViewModelUpdateStream>,

    /// The list of pending subcontroller updates
    pending: VecDeque<ViewModelUpdate>
}

impl ViewModelUpdateStream {
    ///
    /// Creates a new viewmodel update stream with the specified root controller
    ///
    pub fn new(root_controller: Arc<dyn Controller>) -> ViewModelUpdateStream {
        let ui                              = root_controller.ui();
        let controller_stream               = stream::once(Ok(ui.get())).chain(follow(ui));
        let controller_viewmodel_updates    = root_controller.get_viewmodel().map(|viewmodel| viewmodel.get_updates());
        let root_controller                 = Arc::downgrade(&root_controller);

        ViewModelUpdateStream {
            root_controller:                root_controller,
            controller_stream:              Box::new(controller_stream),
            controller_viewmodel_updates:   controller_viewmodel_updates,
            sub_controllers:                HashMap::new(),
            pending:                        VecDeque::new()
        }
    }

    ///
    /// When the controller's UI changes, updates the subcontroller streams
    ///
    fn update_subcontrollers(&mut self, root_controller: &dyn Controller, control: &Control) {
        // Create a replacement set of subcontrollers
        let mut new_sub_controllers = HashMap::new();
        let all_controllers         = control.all_controllers();

        // For each subcontroller, either keep the existing stream or 
        for subcontroller_name in all_controllers {
            if !new_sub_controllers.contains_key(&subcontroller_name) {
                if let Some(existing_controller) = self.sub_controllers.remove(&subcontroller_name) {
                    // Was already tracking this subcontroller
                    new_sub_controllers.insert(subcontroller_name, existing_controller);

                } else if let Some(subcontroller) = root_controller.get_subcontroller(&subcontroller_name) {
                    // Need to track with a new subcontroller
                    let subcontroller_stream = ViewModelUpdateStream::new(subcontroller);

                    new_sub_controllers.insert(subcontroller_name.clone(), subcontroller_stream);
                }
            }
        }

        // Replace the sub controllers with the new subcontrollers
        self.sub_controllers = new_sub_controllers;
    }
}

impl Stream for ViewModelUpdateStream {
    type Item = ViewModelUpdate;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<ViewModelUpdate>, ()> {
        if let Some(update) = self.pending.pop_front() {
            // Return pending items before anything else
            Ok(Async::Ready(Some(update)))
        } else if let Some(root_controller) = self.root_controller.upgrade() {
            // Try the updates from the main controller first
            if let Some(controller_viewmodel_updates) = self.controller_viewmodel_updates.as_mut() {
                let mut all_updates = vec![];

                // Drain the controller updates
                let mut full_update = controller_viewmodel_updates.poll();

                while let Ok(Async::Ready(Some(update))) = full_update {
                    all_updates.push(update);

                    full_update = controller_viewmodel_updates.poll();
                }

                // Unset the controller updates if we reach the end of the stream (the controller and its subcontrollers presumably still exist, so the stream does not end)
                if full_update == Ok(Async::Ready(None)) {
                    // self.controller_viewmodel_updates = None;
                }

                // Return the updates if there were any
                if all_updates.len() > 0 {
                    self.pending.push_back(ViewModelUpdate::new(vec![], all_updates));
                }
            }

            // Check for updates to the controller UI
            let next_ui = self.controller_stream.poll();
            if let Ok(Async::Ready(Some(ref next_ui))) = next_ui {
                // Refresh the subcontrollers from the UI
                self.update_subcontrollers(&*root_controller, next_ui);
            }

            if let Ok(Async::Ready(None)) = next_ui {
                // If the controller's UI stream ends, then the viewmodel updates also end (presumably the controller has been disposed of)
                return Ok(Async::Ready(None));
            }

            // Poll the subcontrollers
            for (name, stream) in self.sub_controllers.iter_mut() {
                let mut subcontroller_update = stream.poll();

                while let Ok(Async::Ready(Some(mut update))) = subcontroller_update {
                    // Add the name of this subcontroller
                    update.add_to_start_of_path(name.clone());

                    // Add the update to the pending list
                    self.pending.push_back(update);

                    // Fetch as many updates as we can from the subcontroller
                    subcontroller_update = stream.poll();
                }
            }

            // If any updates were found, return the first from the pending list
            if let Some(update) = self.pending.pop_front() {
                Ok(Async::Ready(Some(update)))
            } else {
                Ok(Async::NotReady)
            }
        } else {
            // Stream has ended when the root controller no longer exists
            Ok(Async::Ready(None))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::property::*;
    use super::super::dynamic_viewmodel::*;

    use futures::executor;

    ///
    /// A controller that does nothing
    ///
    pub struct DynamicController {
        controls: Arc<Binding<Control>>,
        view_model: Arc<DynamicViewModel>,
        subcontrollers: Mutex<HashMap<String, Arc<DynamicController>>>
    }

    impl DynamicController {
        pub fn new() -> DynamicController {
            DynamicController { 
                controls:       Arc::new(bind(Control::empty())),
                view_model:     Arc::new(DynamicViewModel::new()),
                subcontrollers: Mutex::new(HashMap::new())
            }
        }

        pub fn set_controls(&self, new_control: Control) {
            (*self.controls).set(new_control);
        }

        pub fn add_subcontroller(&self, name: String) {
            self.subcontrollers.lock().unwrap().insert(name, Arc::new(DynamicController::new()));
        }
    }

    impl Controller for DynamicController {
        fn ui(&self) -> BindRef<Control> {
            BindRef::from_arc(Arc::clone(&self.controls))
        }

        fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
            let res = self.subcontrollers.lock().unwrap().get(id).map(|x| x.clone());

            if let Some(res) = res {
                Some(res)
            } else {
                None
            }
        }

        fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
            Some(self.view_model.clone())
        }
    }

    #[derive(Clone)]
    struct NotifyNothing;
    impl executor::Notify for NotifyNothing {
        fn notify(&self, _: usize) { }
    }

    #[test]
    fn initially_no_changes() {
        let controller  = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let mut stream  = executor::spawn(ViewModelUpdateStream::new(controller.clone()));

        assert!(stream.poll_stream_notify(&Arc::new(NotifyNothing), 0) == Ok(Async::NotReady));
    }

    #[test]
    fn changes_are_picked_up() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let mut stream  = executor::spawn(ViewModelUpdateStream::new(controller.clone()));

        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        assert!(stream.wait_stream() == Some(Ok(ViewModelUpdate::new(vec![], vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))]))));
    }

    #[test]
    fn new_values_are_picked_up() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let mut stream  = executor::spawn(ViewModelUpdateStream::new(controller.clone()));

        controller.get_viewmodel().unwrap().set_property("NewValue", PropertyValue::Int(2));

        assert!(stream.wait_stream() == Some(Ok(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("NewValue".to_string(), PropertyValue::Int(2)), ViewModelChange::PropertyChanged("NewValue".to_string(), PropertyValue::Int(2))]))));
    }

    #[test]
    fn new_values_are_picked_up_alongside_changes() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let mut stream  = executor::spawn(ViewModelUpdateStream::new(controller.clone()));

        controller.get_viewmodel().unwrap().set_property("NewValue", PropertyValue::Int(3));
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        // TODO: ordering is unstable here
        // assert!(stream.wait_stream() == Some(Ok(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("NewValue".to_string(), PropertyValue::Int(3)), ViewModelChange::PropertyChanged("NewValue".to_string(), PropertyValue::Int(3)), ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))]))));
    }

    #[test]
    fn subcontroller_changes_are_picked_up() {
        let controller = DynamicController::new();
        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());

        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();
        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let controller = Arc::new(controller);

        let update_stream       = ViewModelUpdateStream::new(controller.clone());
        let mut update_stream   = executor::spawn(update_stream);

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        let update = update_stream.wait_stream().unwrap().unwrap();

        assert!(update.controller_path() == &vec!["Subcontroller".to_string()]);
        assert!(update.updates() == &vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))]);
    }
}
