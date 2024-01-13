use hurl_core::ast::{SourceInfo, Pos, LineTerminator, Whitespace};

pub fn empty_source_info() -> SourceInfo {
    SourceInfo {
        start: Pos { column: 0, line: 0 },
        end: Pos { column: 0, line: 0 },
    }
}

pub fn newline() -> LineTerminator {
    LineTerminator {
        space0: Whitespace {
            value: "".to_string(),
            source_info: empty_source_info(),
        },

        comment: None,
        newline: Whitespace {
            value: "\n".to_string(),
            source_info: SourceInfo {
                start: Pos { column: 0, line: 0 },
                end: Pos { column: 0, line: 0 },
            },
        },
    }
}

pub fn empty_space() -> Whitespace {
    Whitespace {
        value: "".to_string(),
        source_info: empty_source_info(),
    }
}
