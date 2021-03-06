//! Integration tests for GDLK that expect program success. The programs in
//! these tests should compile successfully, and execute with a successful
//! outcome.

use gdlk::{
    ast::LangValue, compile_and_allocate, HardwareSpec, Machine, ProgramSpec,
    Valid, MAX_CYCLE_COUNT,
};

/// Compiles the program for the given hardware, and executes it against the
/// program spec. Panics if the compile fails or the execution isn't
/// successful.
fn execute_expect_success(
    hardware_spec: HardwareSpec,
    program_spec: ProgramSpec,
    src: &str,
) -> Machine {
    let valid_program_spec = Valid::validate(program_spec).unwrap();
    // Compile from hardware+src
    let mut machine = compile_and_allocate(
        &Valid::validate(hardware_spec).unwrap(),
        &valid_program_spec,
        src,
    )
    .unwrap();

    // Execute to completion
    let success = machine.execute_all().unwrap();

    // Make sure program terminated successfully
    // Check each bit of state individually to make debugging easier
    assert_eq!(machine.input, Vec::new() as Vec<LangValue>);
    assert_eq!(machine.output, valid_program_spec.inner().expected_output);
    // Final sanity check, in case we change the criteria for success
    assert!(success);
    machine
}

#[test]
fn test_read_write() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
            input: vec![1, 2],
            expected_output: vec![1, 2],
        },
        "
        READ RX0
        WRITE RX0
        READ RX0
        WRITE RX0
        ",
    );
}

#[test]
fn test_set_push_pop() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 2,
            num_stacks: 1,
            max_stack_length: 5,
        },
        ProgramSpec {
            input: vec![],
            expected_output: vec![10, 5],
        },
        "
        SET RX0 10
        PUSH RX0 S0
        SET RX0 0
        POP S0 RX0
        WRITE RX0
        SET RX1 5
        SET RX0 RX1
        WRITE RX0
        ",
    );
}

#[test]
fn test_arithmetic() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 2,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
            input: vec![],
            expected_output: vec![-3, 140],
        },
        "
        ADD RX0 1
        SUB RX0 2
        MUL RX0 3
        WRITE RX0

        SET RX0 5
        SET RX1 10
        ADD RX0 RX1
        MUL RX0 RX1
        SUB RX0 RX1
        WRITE RX0
        ",
    );
}

#[test]
fn test_cmp() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 2,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
            input: vec![],
            expected_output: vec![-1, 0, 1, 1],
        },
        "
        CMP RX0 1 2
        WRITE RX0
        CMP RX0 2 2
        WRITE RX0
        CMP RX0 10 0
        WRITE RX0

        SET RX0 3
        SET RX1 1
        CMP RX0 RX0 RX1
        WRITE RX0
        ",
    );
}

#[test]
fn test_jumps() {
    let program_spec = ProgramSpec {
        input: vec![],
        expected_output: vec![1],
    };
    execute_expect_success(
        HardwareSpec::default(),
        program_spec.clone(),
        "
        JMP GOOD
        BAD:
        WRITE -1
        GOOD:
        WRITE 1
        ",
    );
    execute_expect_success(
        HardwareSpec::default(),
        program_spec.clone(),
        "
        JEZ 0 GOOD
        BAD:
        WRITE -1
        GOOD:
        WRITE 1
        JEZ 1 BAD
        ",
    );
    execute_expect_success(
        HardwareSpec::default(),
        program_spec.clone(),
        "
        JNZ 1 GOOD
        BAD:
        WRITE -1
        GOOD:
        WRITE 1
        JNZ 0 BAD
        ",
    );
    execute_expect_success(
        HardwareSpec::default(),
        program_spec.clone(),
        "
        JLZ -10 GOOD
        BAD:
        WRITE -1
        GOOD:
        WRITE 1
        JLZ 0 BAD
        ",
    );
    execute_expect_success(
        HardwareSpec::default(),
        program_spec,
        "
        JGZ 3 GOOD
        BAD:
        WRITE -1
        GOOD:
        WRITE 1
        JGZ 0 BAD
        ",
    );
}

#[test]
fn test_square_all() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
            input: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            expected_output: vec![1, 4, 9, 16, 25, 36, 49, 64, 81, 100],
        },
        "
        LOOP:
            JEZ RLI END
            READ RX0
            MUL RX0 RX0
            WRITE RX0
            JMP LOOP
        END:
        ",
    );
}

#[test]
fn test_fibonacci() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 4,
            num_stacks: 0,
            max_stack_length: 0,
        },
        ProgramSpec {
            input: vec![10],
            expected_output: vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34],
        },
        "
        READ RX0
        SET RX1 0
        SET RX2 1
        LOOP:
            JEZ RX0 END
            WRITE RX1
            SET RX3 RX2
            ADD RX2 RX1
            SET RX1 RX3
            SUB RX0 1
            JMP LOOP
        END:
        ",
    );
}

#[test]
fn test_insertion_sort() {
    execute_expect_success(
        HardwareSpec {
            num_registers: 3,
            num_stacks: 2,
            max_stack_length: 16,
        },
        ProgramSpec {
            input: vec![9, 3, 8, 4, 5, 1, 3, 8, 9, 5, 2, 10, 4, 1, 8],
            expected_output: vec![1, 1, 2, 3, 3, 4, 4, 5, 5, 8, 8, 8, 9, 9, 10],
        },
        "
        ; RX0:  the last element pulled off the input
        ; RX1:  the current element in the sorted list we're comparing to
        ; RX2:  scratch space for comparisons and such
        ; S0:   the sorted list so far, with greatest at the bottom
        ; S1:   scratch space used during insertion, to hold the chunk of the
        ;       list that's less than RX0
        READ_LOOP:
            JEZ RLI END_READ_LOOP
            READ RX0
            SET RX2 RS0

            CMP_LOOP:
                JEZ RX2 END_CMP_LOOP
                POP S0 RX1

                ; compare RX0 and RX1
                SET RX2 RX0
                SUB RX2 RX1
                JGZ RX2 0_GT_1

                0_LTE_1:
                    PUSH RX1 S0 ; RX0 <= RX1, put RX1 back on the stack
                    JMP END_CMP_LOOP ; we're done here
                0_GT_1:
                    ; RX0 > RX1, we must go deeper
                    PUSH RX1 S1
                    JGZ RS0 CMP_LOOP ; iterate if there are more values to check
            END_CMP_LOOP:
            ; we found the right spot for RX0!
            PUSH RX0 S0

            ; stack S1 back onto S0
            RESTACK_LOOP:
                JEZ RS1 END_RESTACK_LOOP
                POP S1 RX1
                PUSH RX1 S0
                JMP RESTACK_LOOP
            END_RESTACK_LOOP:

            JMP READ_LOOP
        END_READ_LOOP:

        ; write the sorted list to output
        WRITE_LOOP:
            JEZ RS0 END_WRITE_LOOP
            POP S0 RX0
            WRITE RX0
            JMP WRITE_LOOP
        END_WRITE_LOOP:
        ",
    );
}

#[test]
fn test_cycle_count_simple() {
    let machine = execute_expect_success(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![1],
            expected_output: vec![2],
        },
        "
        READ RX0
        ADD RX0 1
        WRITE RX0
        ",
    );
    assert_eq!(machine.cycle_count, 3);
}

#[test]
fn test_cycle_count_jump() {
    let m1 = execute_expect_success(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![1, 2, 3],
            expected_output: vec![1, 2, 3],
        },
        "
        START:
        JEZ RLI END
        READ RX0
        WRITE RX0
        JMP START
        END:
        ",
    );
    assert_eq!(m1.cycle_count, 13);
}

#[test]
fn test_equal_max_cycle_count() {
    // We can exit successfully with exactly the maximum number of cycles
    let machine = execute_expect_success(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![(MAX_CYCLE_COUNT as i32 - 1) / 2],
            expected_output: vec![],
        },
        "
        READ RX0
        JEZ RX0 END
        LOOP:
        SUB RX0 1
        JGZ RX0 LOOP
        END:
        ",
    );
    assert_eq!(machine.cycle_count, MAX_CYCLE_COUNT);
}

#[test]
fn test_parse_empty_file() {
    execute_expect_success(HardwareSpec::default(), ProgramSpec::default(), "");
}
