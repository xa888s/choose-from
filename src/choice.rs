use std::ops::Deref;

#[derive(Debug)]
pub(crate) struct Guard;

/// A specific choice, passed to closure by [`Selector::with`](crate::Selector::with) or [`SelectorFixed::with`](crate::SelectorFixed::with).
#[derive(Debug)]
#[repr(transparent)]
pub struct Choice<'guard, T> {
    value: T,
    _guard: std::marker::PhantomData<&'guard Guard>,
}

// This type is good to implement Deref because Choice is a transparent wrapper around T
impl<'a, T> Deref for Choice<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'guard, T> Choice<'guard, T> {
    pub(crate) fn with_guard(value: T, _guard: &'guard Guard) -> Choice<'guard, T> {
        Choice {
            value,
            _guard: std::marker::PhantomData,
        }
    }

    pub(crate) fn into_inner(self) -> T {
        self.value
    }
}

pub(crate) fn to_values<T>(choices: Vec<Choice<'_, T>>) -> Vec<T> {
    // TODO: check optimization. This is probably optimized well since
    // choices should have the same size and alignment as T so the collection
    // may not need to reallocate
    choices.into_iter().map(Choice::into_inner).collect()
}
