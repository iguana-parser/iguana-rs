use error_union::ErrorUnionParser;
use iguana_runtime::{
    arena::Arena,
    ids::{GssNodeId, SlotId, TerminalId},
    input::Input,
    parser::{FailureReportKind, GLLFailureKind, Parser},
    scanner::TerminalSet,
};

static FIRST: TerminalSet = TerminalSet {
    id: 0,
    terminals: &[TerminalId(0)],
};
static SECOND: TerminalSet = TerminalSet {
    id: 1,
    terminals: &[TerminalId(1)],
};

#[test]
fn adjacent_duplicates_preserve_the_first_context_and_expected_union() {
    let input = Input::from("abc");
    let arena = Arena::new();
    let mut parser = ErrorUnionParser::new(&input, &arena);
    for slot in 0..299 {
        parser.add_failure(
            1,
            SlotId(slot),
            Some(GssNodeId(slot as u32)),
            GLLFailureKind::UnexpectedToken(&FIRST),
        );
    }
    assert_eq!(parser.failures().count(), 1);
    parser.add_failure(
        1,
        SlotId(300),
        None,
        GLLFailureKind::UnexpectedToken(&SECOND),
    );
    assert_eq!(parser.failures().count(), 2);
    let report = parser.failure().unwrap();
    assert_eq!(report.slot_id, SlotId(0));
    assert_eq!(report.gss_node_id, Some(GssNodeId(0)));
    let FailureReportKind::UnexpectedToken { expected } = report.kind else {
        panic!("expected a terminal union")
    };
    assert_eq!(expected.len(), 2);
    assert!(expected.contains(&TerminalId(0)) && expected.contains(&TerminalId(1)));
}

#[test]
fn farther_failures_replace_duplicates_at_the_previous_position() {
    let input = Input::from("abc");
    let arena = Arena::new();
    let mut parser = ErrorUnionParser::new(&input, &arena);
    let kind = GLLFailureKind::UnexpectedToken(&FIRST);
    parser.add_failure(1, SlotId(0), None, kind);
    parser.add_failure(2, SlotId(1), None, kind);
    parser.add_failure(1, SlotId(2), None, kind);
    assert_eq!(parser.failures().count(), 1);
    let report = parser.failure().unwrap();
    assert_eq!(report.input_index, 2);
    assert_eq!(report.slot_id, SlotId(1));
}

#[test]
fn different_kinds_are_retained_in_recording_order() {
    let input = Input::from("abc");
    for restriction in [
        GLLFailureKind::ExcludedMatch(&FIRST),
        GLLFailureKind::ForbiddenFollow(&FIRST),
    ] {
        for restriction_first in [false, true] {
            let arena = Arena::new();
            let mut parser = ErrorUnionParser::new(&input, &arena);
            let unexpected = GLLFailureKind::UnexpectedToken(&FIRST);
            let kinds = if restriction_first {
                [restriction, unexpected]
            } else {
                [unexpected, restriction]
            };
            for kind in kinds {
                parser.add_failure(1, SlotId(0), None, kind);
                parser.add_failure(1, SlotId(1), None, kind);
            }
            assert_eq!(parser.failures().count(), 2);
            let report = parser.failure().unwrap();
            assert_eq!(
                matches!(report.kind, FailureReportKind::UnexpectedToken { .. }),
                !restriction_first
            );
            if restriction_first {
                assert!(matches!(
                    (restriction, report.kind),
                    (
                        GLLFailureKind::ExcludedMatch(_),
                        FailureReportKind::ExcludedMatch { .. }
                    ) | (
                        GLLFailureKind::ForbiddenFollow(_),
                        FailureReportKind::ForbiddenFollow { .. }
                    )
                ));
            }
        }
    }
}
