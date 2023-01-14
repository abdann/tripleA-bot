use sqlx;

#[derive(Debug, PartialEq, Eq)]
pub enum InsertResult {
    Added,
    AlreadyPresent,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveResult {
    Removed,
    NotPresent,
}

#[derive(Debug)]
pub enum MarkovFetchResultError {
    NotEnoughWords(u8, u32),
    SqlxError(sqlx::Error),
}

impl std::error::Error for MarkovFetchResultError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            MarkovFetchResultError::NotEnoughWords(
                ref desired_sentence_length,
                ref words_collected,
            ) => None,
            MarkovFetchResultError::SqlxError(ref sqlxerror) => Some(sqlxerror),
        }
    }
}

impl std::fmt::Display for MarkovFetchResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MarkovFetchResultError::NotEnoughWords(
                ref desired_sentence_length,
                ref words_collected,
            ) => {
                write!(f, "The provided member does not have enough data stored for a sentence of {} words (Only know {} words)", desired_sentence_length, words_collected)
            }
            MarkovFetchResultError::SqlxError(ref sqlxerror) => {
                write!(f, "SQLx Error: {}", sqlxerror)
            }
        }
    }
}

impl From<sqlx::Error> for MarkovFetchResultError {
    fn from(value: sqlx::Error) -> Self {
        MarkovFetchResultError::SqlxError(value)
    }
}

#[derive(Debug)]
pub enum AddMemberWordsError {
    ImpossibleVectorSizes(usize, usize),
    SqlxError(sqlx::Error),
}

impl std::error::Error for AddMemberWordsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            AddMemberWordsError::ImpossibleVectorSizes(
                ref words_vec_size,
                ref next_words_vec_size,
            ) => None,
            AddMemberWordsError::SqlxError(ref sqlxerror) => Some(sqlxerror),
        }
    }
}

impl std::fmt::Display for AddMemberWordsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AddMemberWordsError::ImpossibleVectorSizes(
                ref words_vec_size,
                ref next_words_vec_size,
            ) => {
                write!(f, "Impossible to chain: words vector is of length {} and next_words vector is of length {} (next_words vector must be at least 1 dimension larger than words vector)", words_vec_size, next_words_vec_size)
            }
            AddMemberWordsError::SqlxError(ref sqlxerror) => write!(f, "SQLx Error: {}", sqlxerror),
        }
    }
}

impl From<sqlx::Error> for AddMemberWordsError {
    fn from(value: sqlx::Error) -> Self {
        AddMemberWordsError::SqlxError(value)
    }
}
