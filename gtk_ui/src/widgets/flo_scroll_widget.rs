use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::flo_fixed_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// The scroll widget manages a layout widget in order to provide a scrollable region
/// 
pub struct FloScrollWidget {
    /// The ID of this widget
    id:             WidgetId,

    /// The scrolling window
    scroll_window:  gtk::ScrolledWindow,

    /// The same, cast as a widget
    as_widget:      gtk::Widget,

    /// The layout, where the actual child controls go
    layout:         gtk::Layout,

    /// We delegate the actual layout tasks (along with things like setting the image and text) to FloFixedWidget
    fixed_widget:   FloFixedWidget,

    /// The horizontal scrollbar policy
    h_policy:       gtk::PolicyType,

    /// The vertical scrollbar policy
    v_policy:       gtk::PolicyType
}

impl FloScrollWidget {
    ///
    /// Creates a new scroll widget
    ///
    pub fn new(id: WidgetId, scroll_window: gtk::ScrolledWindow, widget_data: Rc<WidgetData>) -> FloScrollWidget {
        // Create the widgets
        let layout          = gtk::Layout::new(None, None);

        // Stick them together
        scroll_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never);
        scroll_window.add(&layout);

        // Generate the widget
        let as_widget       = scroll_window.clone().upcast::<gtk::Widget>();
        let fixed_widget    = FloFixedWidget::new(id, layout.clone(), widget_data);

        FloScrollWidget {
            id:             id,
            scroll_window:  scroll_window,
            layout:         layout,
            as_widget:      as_widget,
            fixed_widget:   fixed_widget,
            h_policy:       gtk::PolicyType::Never,
            v_policy:       gtk::PolicyType::Never
        }
    }

    ///
    /// Generates the scrollbar visibility for a particular policy
    /// 
    fn policy_for_visibility(visibility: ScrollBarVisibility) -> gtk::PolicyType {
        use self::ScrollBarVisibility::*;

        match visibility {
            Never           => gtk::PolicyType::Never,
            Always          => gtk::PolicyType::Always,
            OnlyIfNeeded    => gtk::PolicyType::Automatic
        }
    }

    ///
    /// Updates the policy for this scroll widget (which is what GTK calls the rules for showing the scroll bars)
    /// 
    fn update_policy(&self) {
        self.scroll_window.set_policy(self.h_policy, self.v_policy);
    }
}

impl GtkUiWidget for FloScrollWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;
        use self::Scroll::*;
        use self::WidgetContent::SetText;
        use self::Appearance::Image;

        match action {
            // Scroll actions are handled by this control
            &Scroll(MinimumContentSize(width, height))  => { self.layout.set_size(width as u32, height as u32); },
            &Scroll(HorizontalScrollBar(visibility))    => { self.h_policy = Self::policy_for_visibility(visibility); self.update_policy(); },
            &Scroll(VerticalScrollBar(visibility))      => { self.v_policy = Self::policy_for_visibility(visibility); self.update_policy(); },

            // Content actions are handled by the fixed widget
            &Content(SetText(_))                        => { self.fixed_widget.process(flo_gtk, action); },
            &Appearance(Image(_))                       => { self.fixed_widget.process(flo_gtk, action); },

            // All other actions are basic actions
            other_action                                => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        self.fixed_widget.set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}
