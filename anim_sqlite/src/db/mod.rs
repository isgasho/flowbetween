use animation::*;

use desync::*;
use futures::*;
use rusqlite::*;

use std::mem;
use std::sync::*;
use std::ops::Range;
use std::collections::HashMap;

#[cfg(test)] mod tests;

mod flo_store;
mod flo_query;
mod flo_sqlite;
mod db_enum;
mod edit_sink;
mod edit_stream;
mod insert_editlog;
mod animation;
mod animation_core;
mod color;
mod brush;
mod vector_layer;
pub mod vector_frame;

pub use self::animation::*;
pub use self::insert_editlog::*;
pub use self::vector_layer::*;
use self::animation_core::*;
use self::flo_sqlite::*;
use self::flo_store::*;
use self::flo_query::*;
use self::edit_stream::*;

///
/// Database used to store an animation
/// 
pub struct AnimationDb {
    /// The core contains details of the database
    core: Arc<Desync<AnimationDbCore<FloSqlite>>>,

    /// The next available element ID
    next_element_id: Arc<Mutex<i64>>,
}

impl AnimationDb {
    ///
    /// Creates a new animation database with an in-memory database
    /// 
    pub fn new() -> AnimationDb {
        Self::new_from_connection(Connection::open_in_memory().unwrap())
    }

    ///
    /// Creates a new animation database using the specified SQLite connection
    /// 
    pub fn new_from_connection(connection: Connection) -> AnimationDb {
        FloSqlite::setup(&connection).unwrap();

        let core    = Arc::new(Desync::new(AnimationDbCore::new(connection)));

        // We begin assigning element IDs at the current length of the edit log
        let initial_element_id = core.sync(|core| core.db.query_edit_log_length()).unwrap() as i64;

        let db      = AnimationDb {
            core:               core,
            next_element_id:    Arc::new(Mutex::new(initial_element_id))
        };

        db
    }

    ///
    /// Creates an animation database that uses an existing database already set up in a SQLite connection
    /// 
    pub fn from_connection(connection: Connection) -> AnimationDb {
        let core    = Arc::new(Desync::new(AnimationDbCore::new(connection)));

        // We begin assigning element IDs at the current length of the edit log
        let initial_element_id = core.sync(|core| core.db.query_edit_log_length()).unwrap() as i64;

        let db = AnimationDb {
            core:               core,
            next_element_id:    Arc::new(Mutex::new(initial_element_id))
        };

        db
    }

    ///
    /// If there has been an error, retrieves what it is and clears the condition
    /// 
    pub fn retrieve_and_clear_error(&self) -> Option<Error> {
        // We have to clear the error as rusqlite::Error does not implement clone or copy
        self.core.sync(|core| {
            core.retrieve_and_clear_error()
        })
    }

    ///
    /// Performs an async operation on the database
    /// 
    fn async<TFn: 'static+Send+Fn(&mut AnimationDbCore<FloSqlite>) -> Result<()>>(&self, action: TFn) {
        self.core.async(move |core| {
            // Only run the function if there has been no failure
            if core.failure.is_none() {
                // Run the function and update the error status
                let result      = action(core);
                core.failure    = result.err();
            }
        })
    }

    ///
    /// Assigns a new, unique, element ID for the database
    /// 
    pub fn assign_element_id(&self) -> i64 {
        let mut next_element_id = self.next_element_id.lock().unwrap();
        let id                  = *next_element_id;

        *next_element_id += 1;
        id
    }

    ///
    /// Retrieves the number of edits in the animation
    ///
    pub fn get_num_edits(&self) -> Result<usize> {
        self.core.sync(|core| core.db.query_edit_log_length()).map(|length| length as usize)
    }

    ///
    /// Creates a stream for reading the specified range of elements from this animation
    ///
    pub fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<'a+Stream<Item=AnimationEdit, Error=()>> {
        let edit_stream = EditStream::new(&self.core, range);

        Box::new(edit_stream)
    }
}

impl AnimationDbCore<FloSqlite> {
    ///
    /// Creates a new database core with a sqlite connection
    /// 
    fn new(connection: Connection) -> AnimationDbCore<FloSqlite> {
        let core = AnimationDbCore {
            db:                     FloSqlite::new(connection),
            failure:                None,
            active_brush_for_layer: HashMap::new()
        };

        core
    }
}

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// If there has been an error, retrieves what it is and clears the condition
    /// 
    fn retrieve_and_clear_error(&mut self) -> Option<Error> {
        // We have to clear the error as rusqlite::Error does not implement clone or copy
        let mut failure = None;
        mem::swap(&mut self.failure, &mut failure);

        failure
    }
}
