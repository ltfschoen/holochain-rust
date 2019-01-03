extern crate futures;
use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{cas::content::Address, error::HcResult};
use std::{
    pin::Pin, sync::Arc,
    thread::sleep,
    time::Duration,
};

/// GetEntry Action Creator
/// This is the network version of get_entry that makes the network module start
/// a look-up process.
///
/// Returns a future that resolves to an ActionResponse.
pub async fn get_links<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
) -> HcResult<Vec<Address>> {
    let action_wrapper = ActionWrapper::new(Action::GetLinks(address.clone()));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let _ = async {
        sleep(Duration::from_secs(60));
        let action_wrapper = ActionWrapper::new(Action::GetLinksTimeout(address.clone()));
        dispatch_action(context.action_channel(), action_wrapper.clone());
    };
    
    await!(GetLinksFuture {
        context: context.clone(),
        address: address.clone(),
    })
}

/// GetEntryFuture resolves to a HcResult<Entry>.
/// Tracks the state of the network module
pub struct GetLinksFuture {
    context: Arc<Context>,
    address: Address,
}

impl Future for GetLinksFuture {
    type Output = HcResult<Vec<Address>>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        let state = self.context.state().unwrap().network();
        if let Err(error) = state.initialized() {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        match state.get_links_results.get(&self.address) {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
