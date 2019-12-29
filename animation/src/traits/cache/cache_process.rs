use futures::*;
use futures::task::{Poll, Context};

///
/// Represents an item that is in the cache, or is in the process of being generated
///
pub enum CacheProcess<Result, Process: Future<Output=Result>+Send> {
    /// The item was already cached and is retrieved
    Cached(Result),

    /// The item has not been cached and is being generated
    Process(Process),
}

impl<Result: Clone, Process: Future<Output=Result>+Send> Future for CacheProcess<Result, Process> {
    type Output = Result;

    fn poll(&mut self, context: &mut Context) -> Poll<Result> {
        match self {
            CacheProcess::Cached(result)    => Poll::Ready(result.clone()),
            CacheProcess::Process(process)  => {
                // Cache value will become available in the future: poll for it
                let poll_result = process.poll(context);

                if let Poll::Ready(poll_result) = poll_result {
                    // Cache value is now available. Update the state to be just 'Cached' so we don't need to poll again
                    *self = CacheProcess::Cached(poll_result.clone());
                    Poll::Ready(poll_result)
                } else {
                    // Pass on the poll result in all other circumstances
                    poll_result
                }
            },
        }
    }
}
