use super::sqlite_storage::*;

use flo_animation::*;
use flo_animation::storage::*;

use futures::prelude::*;
use rusqlite::{Connection, OpenFlags};

///
/// Creates a loader for loading animations stored in SQLite files
///
pub fn sqlite_animation_loader() -> impl FileAnimation {
    AnimationLoader(|path| {
        // Connect to the database
        let connection  = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();
        let storage     = SqliteAnimationStorage::new_from_connection(connection);

        // Create the editor for this animation
        create_animation_editor(move |commands| storage.get_responses(commands).boxed())
    })
}
