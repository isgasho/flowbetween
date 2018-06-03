use super::*;
use super::time_path::*;
use super::motion_path_type::*;

impl AnimationDb {
    ///
    /// Retrieves a particular time path for a motion
    /// 
    fn get_motion_path<TFile: FloFile+Send>(core: &mut AnimationDbCore<TFile>, motion_id: i64, path_type: MotionPathType) -> Result<TimeCurve> {
        // Retrieve the entries for this path
        let entries = core.db.query_motion_timepoints(motion_id, path_type)?;

        // Convert to a time curve
        Ok(time_curve_from_time_points(entries))
    }

    ///
    /// Interprets a motion entry as a translate motion
    /// 
    fn get_translate_motion<TFile: FloFile+Send>(core: &mut AnimationDbCore<TFile>, motion_id: i64, entry: MotionEntry) -> Result<Motion> {
        // Translations should always have an origin: we use 0,0 as a default if none is supplied
        let origin      = entry.origin.unwrap_or((0.0, 0.0));

        // They also have a time curve representing where the translation moves the element
        let motion_path = Self::get_motion_path(core, motion_id, MotionPathType::Position)?;

        // Create the motion
        Ok(Motion::Translate(TranslateMotion {
            origin:     origin,
            translate:  motion_path
        }))
    }
    
    ///
    /// Retrieves the motion with the specified ID
    /// 
    pub fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        if let ElementId::Assigned(motion_id) = motion_id {
            // Query and generate the motion
            self.core.sync(move |core| -> Result<Option<Motion>> {
                // Query the entry for this ID
                let motion_entry = core.db.query_motion(motion_id)?;

                if let Some(motion_entry) = motion_entry {
                    // Generate a motion from this entry
                    match motion_entry.motion_type {
                        MotionType::None        => Ok(Some(Motion::None)),
                        MotionType::Translate   => Ok(Some(Self::get_translate_motion(core, motion_id, motion_entry)?))
                    }
                } else {
                    // No entry with this ID
                    Ok(None)
                }
            }).unwrap()
        } else {
            // The unassigned ID never has any motions attached to it
            None
        }
    }

    ///
    /// Retrieves all of the motion IDs attached to the specified element
    /// 
    pub fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId> {
        if let ElementId::Assigned(element_id) = element_id {
            // Assigned element IDs have attached motions
            let motion_ids = self.core.sync(move |core| core.db.query_motion_ids_for_element(element_id)).unwrap();

            motion_ids.into_iter()
                .map(|raw_id| ElementId::Assigned(raw_id))
                .collect()
        } else {
            // Unassigned element IDs have no attached motions
            vec![]
        }
    }
}
