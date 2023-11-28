//! Postgres change filtering.

struct Filter {
    queries: Vec<(String, String)>,
}

impl Filter {
    /// Creates a new `Filter`.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// let filter = Filter::new()
    ///     .eq("column_name", "foo");
    /// ```
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
        }
    }

    /// Filter by rows whose value on the stated `column` exactly matches the specified `filter`.
    pub fn eq<T, U>(mut self, column: T, filter: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        self.queries
            .push((column.as_ref().into(), format!("eq.{}", filter.as_ref())));
        self
    }

    /// Filter by rows whos value on the stated `column` doesn't match the specified `filter`.
    pub fn neq<T, U>(mut self, column: T, filter: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        self.queries
            .push((column.as_ref().into(), format!("neq.{}", filter.as_ref())));
        self
    }

    /// Filter by rows whose value on the stated `column` is greater than the specified `filter`.
    pub fn gt<T, U>(mut self, column: T, filter: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        self.queries
            .push((column.as_ref().into(), format!("gt.{}", filter.as_ref())));
        self
    }

    /// Filter by rows whose value on the stated `column` is greater than or equal to the specified `filter`.
    pub fn gte<T, U>(mut self, column: T, filter: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        self.queries
            .push((column.as_ref().into(), format!("gte.{}", filter.as_ref())));
        self
    }

    /// Filter by rows whose value on the stated `column` is less than the specified `filter`.
    pub fn lt<T, U>(mut self, column: T, filter: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        self.queries
            .push((column.as_ref().into(), format!("lt.{}", filter.as_ref())));
        self
    }

    /// Filter by rows whose value on the stated `column` is less than or equal to the specified `filter`.
    pub fn lte<T, U>(mut self, column: T, filter: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        self.queries
            .push((column.as_ref().into(), format!("lte.{}", filter.as_ref())));
        self
    }

    /// Filter by rows whose value on the stated `column` is found on the specified `values`.
    pub fn in_<T, U, V>(mut self, column: T, values: U) -> Self
    where
        T: AsRef<str>,
        U: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        let mut values: String = values
            .into_iter()
            .fold(String::new(), |a, s| a + s.as_ref() + ",");
        values.pop();
        self.queries
            .push((column.as_ref().into(), format!("in.({})", values)));
        self
    }

    /// Build the filter statement.
    pub fn build(self) -> String {
        let f: String = self
            .queries
            .into_iter()
            .map(|f| format!("{}={}", f.0, f.1))
            .collect::<Vec<String>>()
            .join(",");
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_filters() {
        let f = Filter::new();
        let built = f.build();

        assert_eq!(built, "");
    }

    #[test]
    fn column_equals() {
        let f = Filter::new().eq("foo_column", "bar_value");
        let built = f.build();

        assert_eq!(built, "foo_column=eq.bar_value");
    }

    #[test]
    fn column_chain_equals() {
        let f = Filter::new()
            .eq("foo_column", "bar_value")
            .eq("foo_column", "baz_value");
        let built = f.build();

        assert_eq!(built, "foo_column=eq.bar_value,foo_column=eq.baz_value");
    }
}
