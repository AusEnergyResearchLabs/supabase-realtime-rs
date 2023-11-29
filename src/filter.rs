//! Postgres change filtering.

/// Associated filter builders.
///
///
/// # Example
///
/// ```rust
/// let filter = Filter::eq("column_name", "value");
/// ```
struct Filter {}

impl Filter {
    /// Filter by rows whose value on the stated `column` exactly matches the specified `filter`.
    pub fn eq<T, U>(column: T, value: U) -> String
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        format!("{}=eq.{}", column.as_ref(), value.as_ref())
    }

    /// Filter by rows whos value on the stated `column` doesn't match the specified `filter`.
    pub fn neq<T, U>(column: T, value: U) -> String
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        format!("{}=neq.{}", column.as_ref(), value.as_ref())
    }

    /// Filter by rows whose value on the stated `column` is greater than the specified `filter`.
    pub fn gt<T, U>(column: T, value: U) -> String
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        format!("{}=gt.{}", column.as_ref(), value.as_ref())
    }

    /// Filter by rows whose value on the stated `column` is greater than or equal to the specified `filter`.
    pub fn gte<T, U>(column: T, value: U) -> String
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        format!("{}=gte.{}", column.as_ref(), value.as_ref())
    }

    /// Filter by rows whose value on the stated `column` is less than the specified `filter`.
    pub fn lt<T, U>(column: T, value: U) -> String
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        format!("{}=lt.{}", column.as_ref(), value.as_ref())
    }

    /// Filter by rows whose value on the stated `column` is less than or equal to the specified `filter`.
    pub fn lte<T, U>(column: T, value: U) -> String
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        format!("{}=lte.{}", column.as_ref(), value.as_ref())
    }

    /// Filter by rows whose value on the stated `column` is found on the specified `values`.
    pub fn in_<T, U, V>(column: T, values: U) -> String
    where
        T: AsRef<str>,
        U: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        let mut values: String = values
            .into_iter()
            .fold(String::new(), |a, s| a + s.as_ref() + ",");
        values.pop(); // remove last oomma

        format!("{}=in.({})", column.as_ref(), values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_filters() {
        let f = Filter::eq("foo_column", "bar_value");
        assert_eq!(f, "foo_column=eq.bar_value");
        let f = Filter::neq("foo_column", "bar_value");
        assert_eq!(f, "foo_column=neq.bar_value");
        let f = Filter::gt("foo_column", "bar_value");
        assert_eq!(f, "foo_column=gt.bar_value");
        let f = Filter::gte("foo_column", "bar_value");
        assert_eq!(f, "foo_column=gte.bar_value");
        let f = Filter::lt("foo_column", "bar_value");
        assert_eq!(f, "foo_column=lt.bar_value");
        let f = Filter::lte("foo_column", "bar_value");
        assert_eq!(f, "foo_column=lte.bar_value");
        let f = Filter::in_("foo_column", &["bar", "baz"]);
        assert_eq!(f, "foo_column=in.(bar,baz)");
    }
}
