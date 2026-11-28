//! Which delivery of a document a handler is answering, and whether its
//! response may be kept in the SPEC §7.2 item 11 record.
//!
//! See [`Delivery`].

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

thread_local! {
    static CURRENT: RefCell<Option<Delivery>> = const { RefCell::new(None) };
}

/// The delivery a consume pipeline is running its handler for.
///
/// The consume pipelines ([`consume_inbound`](crate::consume_inbound),
/// [`consume_dispatched`](crate::consume_dispatched),
/// [`consume_dispatched_answer`](crate::consume_dispatched_answer), and the
/// binding consumers built on them) answer a duplicate document from the
/// response its first execution produced. That is what SPEC §7.2
/// (*Disposition of a duplicate*) asks for, and the default.
///
/// A specification can rule it out. `hcp/grants/request` 0.1 issues an access
/// token the provider keeps in **no** record of the exchange, and answers a
/// re-delivered copy of an answered request with a *newly issued* token. A
/// handler for such a task:
///
/// 1. calls [`Delivery::withhold_response`] while answering the first
///    delivery. The pipeline records that the `id` was executed, but not the
///    response ([`ReplayGuard::record_withheld`](crate::ReplayGuard::record_withheld));
/// 2. is called again for a duplicate of that document, with
///    [`Delivery::is_redelivery`] `true`. The document has already been
///    executed: the handler answers it (a new token for the same grant) and
///    **must not** execute it a second time. The pipeline does not claim it.
///
/// A handler that never calls `withhold_response` sees no difference:
/// duplicates are still answered from the record and never reach it.
///
/// The delivery is ambient, like a `tokio::task_local!`, so no handler
/// signature changes: [`Delivery::current`] returns it while the pipeline runs
/// the handler — during the call that builds the handler's future and every
/// poll of that future — and `None` anywhere else, including inside a task the
/// handler spawns. Read it in the handler itself.
///
/// ```rust,ignore
/// |request, parties| async move {
///     let delivery = Delivery::current().expect("called by the consume pipeline");
///     let grant = if delivery.is_redelivery() {
///         grants.for_request(&request.id)?          // already issued: look it up
///     } else {
///         grants.issue(&request, &parties).await?    // the first execution
///     };
///     // The token goes into no record, the replay record included.
///     delivery.withhold_response();
///     Ok(request.respond_with(new_id(), response_with(grant, tokens.issue(&grant))))
/// }
/// ```
///
/// The same mechanism exists in every runtime (TypeScript: `parties.delivery`;
/// Go: `trusttasks.DeliveryFrom(ctx)`; Dart: `Delivery.current`);
/// see `docs/runtime-parity.md`.
#[derive(Debug, Clone)]
pub struct Delivery {
    redelivery: bool,
    withhold: Arc<AtomicBool>,
}

impl Delivery {
    /// The first execution of a document.
    pub(crate) fn first() -> Self {
        Self {
            redelivery: false,
            withhold: Arc::new(AtomicBool::new(false)),
        }
    }

    /// A duplicate of a document already executed, whose response the
    /// handler kept out of the record.
    pub(crate) fn redelivery() -> Self {
        Self {
            redelivery: true,
            withhold: Arc::new(AtomicBool::new(false)),
        }
    }

    /// The delivery the calling handler is answering, while a consume
    /// pipeline runs it; `None` outside one.
    pub fn current() -> Option<Delivery> {
        CURRENT.with(|current| current.borrow().clone())
    }

    /// `true` when this document was executed before and its response kept
    /// out of the record: the handler answers it, and **must not** execute it
    /// again (SPEC §7.2 item 11). `false` on the first execution.
    pub fn is_redelivery(&self) -> bool {
        self.redelivery
    }

    /// Keep the response the handler returns out of the duplicate-execution
    /// record. The `id` is still recorded as executed, so a duplicate is
    /// never executed again; it is handed back to the handler as a
    /// re-delivery instead of being answered from the record.
    ///
    /// Applies to whatever the first execution answers: a success response,
    /// a non-retryable refusal, or nothing. A retryable refusal releases the
    /// claim whatever this says. No effect on a re-delivery (nothing is
    /// recorded for one) or without a replay guard.
    pub fn withhold_response(&self) {
        self.withhold.store(true, Ordering::Relaxed);
    }

    /// Whether [`withhold_response`](Self::withhold_response) was called.
    pub fn is_response_withheld(&self) -> bool {
        self.withhold.load(Ordering::Relaxed)
    }

    /// Run `f` with `self` as [`Delivery::current`], restoring the previous
    /// value afterwards, also when `f` panics.
    pub(crate) fn scope<T>(&self, f: impl FnOnce() -> T) -> T {
        struct Restore(Option<Delivery>);
        impl Drop for Restore {
            fn drop(&mut self) {
                let previous = self.0.take();
                // `try_with`: a destructor must not panic while a thread's
                // locals are being torn down.
                let _ = CURRENT.try_with(|current| *current.borrow_mut() = previous);
            }
        }
        let previous = CURRENT.with(|current| current.replace(Some(self.clone())));
        let _restore = Restore(previous);
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_is_set_only_inside_the_scope_and_nests() {
        assert!(Delivery::current().is_none());
        let outer = Delivery::first();
        outer.scope(|| {
            assert!(!Delivery::current().unwrap().is_redelivery());
            Delivery::redelivery().scope(|| {
                assert!(Delivery::current().unwrap().is_redelivery());
            });
            let current = Delivery::current().unwrap();
            assert!(!current.is_redelivery());
            current.withhold_response();
        });
        assert!(outer.is_response_withheld());
        assert!(Delivery::current().is_none());
    }

    #[test]
    fn a_panic_restores_the_previous_delivery() {
        let caught = std::panic::catch_unwind(|| {
            Delivery::first().scope(|| panic!("handler"));
        });
        assert!(caught.is_err());
        assert!(Delivery::current().is_none());
    }
}
