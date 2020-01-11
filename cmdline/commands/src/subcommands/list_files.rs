use super::super::state::*;
use super::super::output::*;

use futures::prelude::*;

use flo_stream::*;

///
/// Implementations of the list_files command
///
pub fn list_files<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl 'a+Future<Output=()>+Send {
    async move {
        use self::FloCommandOutput::*;

        let file_manager = state.file_manager();

        // Get all the files in the current folder
        let all_files   = file_manager.get_all_files();
        let count       = format!("{} files", all_files.len());
        output.publish(Message(count)).await;
        output.publish(Message("".to_string())).await;

        let mut index = 0;
        for file in all_files {
            let full_name = file_manager.display_name_for_path(file.as_path()).unwrap_or("<untitled>".to_string());
            let file_name = format!("#{}#: {}", index, full_name);
            output.publish(Message(file_name)).await;

            index += 1;
        }
    }
}
