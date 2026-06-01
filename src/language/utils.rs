use logos::Span;

pub fn merge_span(a: &Span, b: &Span) -> Span {
    Span {
        start: a.start.min(b.start),
        end: a.end.max(b.end),
    }
}
