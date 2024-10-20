use crate::{choice, Choice, Guard};

/// Wraps a variable amount of choices and provides methods that guarantee selection from those choices.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Selector<I, T>
where
    I: IntoIterator<Item = T>,
{
    choices: I,
}

impl<I, T> Selector<I, T>
where
    I: IntoIterator<Item = T>,
{
    pub(crate) fn with_choices(choices: I) -> Selector<I, T> {
        Selector { choices }
    }

    /// The function `chooser` is used to choose from our provided
    /// choices by returning a K-selection of it. The values of these choices are then
    /// returned by the function.
    /// ```
    /// use choose_from::choose_from;
    /// let choices = vec!["Hi", "how", "are ya?"];
    ///
    /// let chosen = choose_from(choices).with(|mut choices| {
    ///     // the provided choices allow inspection of the values
    ///     let third = choices.pop().unwrap();
    ///     assert_eq!(*third.value(), "are ya?");
    ///     
    ///     // ignore 2nd
    ///     choices.pop();
    ///
    ///     let first = choices.pop().unwrap();
    ///     
    ///     // this is our selection
    ///     [first, third]
    /// });
    ///
    /// assert_eq!(chosen, ["Hi", "are ya?"]);
    /// ```
    // we pass our possible choices to the function wrapped in Choice, which only allows
    // inspection of the value, and it must return an array of size K back full
    // of our choices. The values returned are GUARANTEED to only come from our original
    // choices thanks to the Choice struct and guard
    pub fn with<const K: usize, C>(self, chooser: C) -> [T; K]
    where
        C: FnOnce(Vec<Choice<'_, T>>) -> [Choice<'_, T>; K],
    {
        // here we use a guard to prevent the caller from "smuggling" a value out of the closure.
        // This ensures that Choice values built from our given choices are only
        // available within the closure (they can't escape), since Choice has no
        // publicly accessible constructor.
        let _guard = Guard;

        let choices = self.into_choices(&_guard);

        chooser(choices).map(Choice::into_inner)
        // _guard is dropped when function returns, which means that no one
        // has any Choice values anymore
    }

    /// Like [with](Selector::with), but for returning any number of chosen values. Use this when
    /// you want to ensure some values come from the choices, but the amount of chosen values returned
    /// doesn't matter.
    /// ```
    /// use choose_from::choose_from;
    ///
    /// let choices = vec!["Hi", "how", "are ya?"];
    ///
    /// let chosen = choose_from(choices).any_with(|choices| {
    ///     choices.into_iter().step_by(2).collect()
    /// });
    ///
    /// assert_eq!(chosen, ["Hi", "are ya?"]);
    /// ```
    pub fn any_with<'guard, C>(self, chooser: C) -> Vec<T>
    where
        C: FnOnce(Vec<Choice<'_, T>>) -> Vec<Choice<'guard, T>>,
    {
        let _guard = Guard;
        let choices = self.into_choices(&_guard);

        choice::to_values(chooser(choices))
    }

    fn into_choices(self, _guard: &'_ Guard) -> Vec<Choice<'_, T>> {
        // TODO: check optimization. This is probably optimized well since
        // choices should have the same size and alignment as T so the collection
        // may not need to reallocate
        self.choices
            .into_iter()
            .map(|t| Choice::with_guard(t, _guard))
            .collect()
    }
}
