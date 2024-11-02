use crate::{choice, Choice, Guard};

/// Wraps a fixed number of choices and provides methods that guarantee selection from those choices,
/// where N is the possible number of choices set at compile time.
// probably wouldn't need this type with HKTs :(
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SelectorFixed<const N: usize, T> {
    choices: [T; N],
}

impl<const N: usize, T> SelectorFixed<N, T> {
    pub(crate) fn with_choices(choices: [T; N]) -> SelectorFixed<N, T> {
        SelectorFixed { choices }
    }

    /// The function `chooser` is used to choose from our provided
    /// choices by returning a K-selection of them. The values of these choices are then
    /// returned by the function.
    /// ```
    /// use choose_from::select_from_fixed;
    /// let choices = ["Hi", "how", "are ya?"];
    ///
    /// let chosen = select_from_fixed(choices).with(|[first, second, third]| {
    ///     // the provided choices allow inspection of the values
    ///     assert_eq!(*first, "Hi");
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
        C: FnOnce([Choice<'_, T>; N]) -> [Choice<'_, T>; K],
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

    /// Like [with](SelectorFixed::with), but for returning any number of chosen values. Use this when
    /// you want to ensure some values come from the choices, but the amount of chosen values returned
    /// doesn't matter
    /// ```
    /// use choose_from::select_from_fixed;
    ///
    /// let choices = ["Hi", "how", "are ya?"];
    ///
    /// let chosen = select_from_fixed(choices).any_with(|choices| {
    ///     choices.into_iter().step_by(2).collect()
    /// });
    ///
    /// assert_eq!(chosen, ["Hi", "are ya?"]);
    /// ```
    pub fn any_with<C>(self, chooser: C) -> Vec<T>
    where
        C: FnOnce([Choice<'_, T>; N]) -> Vec<Choice<'_, T>>,
    {
        let _guard = Guard;
        let choices = self.into_choices(&_guard);

        choice::to_values(chooser(choices))
    }

    fn into_choices(self, _guard: &'_ Guard) -> [Choice<'_, T>; N] {
        self.choices.map(|t| Choice::with_guard(t, _guard))
    }
}
