struct Builder {
    queries: Vec<(String, String)>,
}

impl Builder {
    /// Creates a new `Builder`.
    pub fn new() -> Self {
        Builder {
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
    pub fn build() -> String {
        String::new()
    }
}
