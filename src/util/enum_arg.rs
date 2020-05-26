use std::fmt;
use std::fmt::Formatter;

use itertools::Itertools;

pub trait EnumArg where Self: Clone + 'static {
    fn case_sensitive() -> bool {
        false
    }
    
    fn variants() -> &'static [Self];
    
    /// Must return at least 1 value
    fn displays(&self) -> &'static [&'static str];
    
    fn matches(&self, input: &str) -> bool {
        self.displays()
            .iter()
            .any(|s| match Self::case_sensitive() {
                true => input == *s,
                false => input.eq_ignore_ascii_case(s),
            })
    }
    
    fn str_variants() -> Vec<&'static str> {
        Self::variants()
            .iter()
            .flat_map(|it| it.displays().iter())
            .map(|it| *it)
            .collect()
    }
    
    /// Can't provide a blanket impl for [`Display`]
    /// since there's already a [`&T`] -> [`T`] blanket impl.
    /// Implementors of EnumArg should manually impl [`Display`],
    /// calling this method.
    /// In the future, this should be done by a macro.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.displays()[0])?;
        Ok(())
    }
    
    /// Can't provide a blanket impl for [`FromStr`]
    /// since [`FromStr`] is a foreign trait.
    /// Implementors of EnumArg should manually impl [`FromStr`],
    /// calling this method.
    /// In the future, this should be done by a macro.
    fn from_str(input: &str) -> Result<Self, String> {
        for variant in Self::variants() {
            if variant.matches(input) {
                return Ok(variant.clone());
            }
        }
        Err(format!("{} must be one of [{}]", input, Self::variants()
            .iter()
            .flat_map(|it| it.displays().iter())
            .join(", ")
        ))
    }
}
