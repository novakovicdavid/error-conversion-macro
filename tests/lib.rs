#[cfg(test)]
mod tests {
    use error_conversion_macro::ErrorEnum;

    mod anyhow {
        #[derive(Debug)]
        pub struct Error;
    }

    #[derive(Debug)]
    enum ApplicationError {
        AnyhowError(anyhow::Error)
    }

    #[derive(Debug)]
    enum ErrorWithoutAnyhow {
        SomeError,
    }

    #[derive(Debug, ErrorEnum)]
    enum Error {
        AnyhowError(anyhow::Error),
        ApplicationError(ApplicationError),

        #[without_anyhow]
        ErrorWithoutAnyhow(ErrorWithoutAnyhow),
    }

    #[test]
    fn conversion() {
        let error = Error::from(ApplicationError::AnyhowError(anyhow::Error {}));

        let result = match error {
            Error::AnyhowError(_) => true,
            Error::ApplicationError(_) => false,
            Error::ErrorWithoutAnyhow(_) => false,
        };

        assert_eq!(true, result);
    }

    #[test]
    fn without_anyhow() {
        let error = Error::from(ErrorWithoutAnyhow::SomeError);
        assert!(matches!(Error::ErrorWithoutAnyhow(ErrorWithoutAnyhow::SomeError), _error));
    }
}