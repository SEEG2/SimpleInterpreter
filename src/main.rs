use std::any::Any;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use crate::keyword::*;
use crate::Operator::{Add, Div, Mul, NoOperator, Sub};

mod keyword;

pub const PROGRAM: &str = "flag a35;var int a 45;shout b\n;terminate $a;jump a35";
pub static mut INSTRUCTION_POINTER: isize = 0;
pub static mut INSTRUCTION_COUNTER: isize = 0;

thread_local! {
    pub static FLAG_MAP: RefCell<HashMap<String, usize>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    pub static VAR_MAP: RefCell<HashMap<String, VarValue>> =
        RefCell::new(HashMap::new());
}

#[derive(Clone)]
enum VarValue {
    Bool(bool), Int(i32), Float(f32)
}

impl VarValue {
    fn is_same_type(&self, other: &VarValue) -> bool {
        matches!(
            (self, other),
            (VarValue::Bool(_), VarValue::Bool(_)) |
            (VarValue::Int(_), VarValue::Int(_)) |
            (VarValue::Float(_), VarValue::Float(_))
        )
    }

    fn to_string(&self) -> String {
        match self {
            VarValue::Bool(v) => v.to_string(),
            VarValue::Int(v) => v.to_string(),
            VarValue::Float(v) => v.to_string()
        }
    }

    fn type_to_string(&self) -> String {
        match self {
            VarValue::Bool(v) => "bool".to_string(),
            VarValue::Int(v) => "int".to_string(),
            VarValue::Float(v) => "float".to_string()
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    NoOperator
}

fn main() {
    let operations: Vec<&str> = PROGRAM.split(";").collect();
    unsafe {
        while INSTRUCTION_POINTER >= 0 && INSTRUCTION_POINTER.cast_unsigned() < operations.len() {
            INSTRUCTION_COUNTER += 1;
            let operation = *operations.get(INSTRUCTION_POINTER.cast_unsigned()).unwrap();
            let mut operation_and_context = operation.splitn(2, " ");
            let last_instr_pointer = INSTRUCTION_POINTER;
            INSTRUCTION_POINTER += 1;
            let error_details = interpret_instruction(operation_and_context.next().unwrap(), operation_and_context.next().unwrap_or(""));
            if error_details.is_some() {
                INSTRUCTION_POINTER = -2;
                let local_instruction_counter = INSTRUCTION_COUNTER;
                println!("--- ERROR ---");
                println!("Error Details:");
                println!(" CALL: \"{}\" at position {} with instruction counter at {}", operation, last_instr_pointer+1, local_instruction_counter);
                println!("   {}", error_details.unwrap());
            }


        }

        if INSTRUCTION_POINTER.is_negative() {
            let exit_code = (-INSTRUCTION_POINTER) - 1;
            if exit_code == 0 {
                println!("Program exited successfully (Code: {})", exit_code);
                return;
            }

            println!("Program exited with an error (Code: {})", exit_code);
            return;
        } else if INSTRUCTION_POINTER.cast_unsigned() >= operations.len() {
            println!("Program reached the end (Code: 2)");
            return;
        }
    }
}

fn interpret_instruction(instruction: &str, context: &str) -> Option<String> {
    match instruction {
        INSTR_TERMINATE => instr_terminate(prepare_context(context)),
        INSTR_FLAG => instr_flag(prepare_context(context)),
        INSTR_JUMP => instr_jump(prepare_context(context)),
        INSTR_SHOUT => instr_shout(context),
        INSTR_VAR => instr_var(prepare_context(context)),
        _ => Some(format!("Instruction not found \"{}\"", instruction))
    }
}

fn instr_terminate(context_raw: Vec<&str>) -> Option<String> {
    let mut argument: String = match context_raw.get(0) {
        Some(arg) => (*arg).to_string(),
        None => return Some("Termination code argument is missing".to_string()),
    };


    if context_raw.len() > 1 {
        return Some(create_too_many_args_error(1, context_raw.len()))
    }

    let exit_code;
    let mut chars  = argument.chars();
    if chars.next().unwrap() == IND_RESOLVE_VARIABLE {
        let result  = resolve_variable(&*chars.collect::<String>());

        match result.1 {
            Some(a) => return Some(a),
            None => argument = result.0.to_string()
        }
    }

    exit_code = match argument.parse::<u8>() {
        Ok(code) => code,
        Err(_) => {
            return Some(format!("Invalid termination code {}", argument));
        }
    };

    if exit_code == 2 {
        return Some("Termination code 2 is reserved and cannot be used".to_string())
    }

    unsafe {
        INSTRUCTION_POINTER = -(exit_code as isize) - 1;
    }

    None
}

fn instr_flag(context: Vec<&str>) -> Option<String> {
    let argument = match context.get(0) {
        Some(arg) => *arg,
        None => return Some("Flag label required".to_string()),
    };

    if context.len() > 1 {
        return Some(create_too_many_args_error(1, context.len()))
    }

    unsafe {
        FLAG_MAP.with(|map| {
            if map.borrow_mut().insert(argument.to_owned(), INSTRUCTION_POINTER.cast_unsigned()).is_some() {
                return  Some(format!("A flag with label \"{}\" already exists", argument))
            }
            None
        })
    }
}

fn instr_jump(context: Vec<&str>) -> Option<String> {
    let argument = match context.get(0) {
        Some(arg) => *arg,
        None => return Some("Jump label required".to_string()),
    };

    if context.len() > 1 {
        return Some(create_too_many_args_error(1, context.len()))
    }

    unsafe {
        FLAG_MAP.with(|map| {
            return match map.borrow_mut().get(&argument.to_owned()) {
                Some(i) => {INSTRUCTION_POINTER = *i as isize; return None}
                None => Some(format!("No flag with label \"{}\" exists", argument))
            }
        })
    }
}

fn instr_shout(context_raw: &str) -> Option<String> {
    if context_raw.len() == 0 {
        return Some("Shout string or variable required".to_string())
    }

    let mut context_as_chars = context_raw.chars();
    let first_char = context_as_chars.next().unwrap();
    if first_char  == IND_RESOLVE_VARIABLE {
        let rest: String = context_as_chars.collect();
        let resolve_result = resolve_variable(rest.as_str());
        return match resolve_result.1  {
            Some(v) => Some(v),
            None => {
                print!("{}", resolve_result.0.to_string());
                None
            }
        }
    } else if first_char == IND_STRING_IGNORE {
        print!("{}", context_raw.replacen(IND_STRING_IGNORE, "", 1));
    } else {
        print!("{}", context_raw);
    }

    None
}

fn instr_var(context: Vec<&str>) -> Option<String> {
    let name_and_option = instr_var_sub_name(context.clone());

    if context.len() > 3 {
        return Some(create_too_many_args_error(3, context.len()))
    }

    match context.get(0) {
        Some(a) if *a == TYPE_BOOL => {
            match name_and_option.1 {
                Some(msg) => Some(msg),
                None => instr_var_sub_bool(name_and_option.0, context)
            }
        },
        Some(a) if *a == TYPE_INT => {
            match name_and_option.1 {
                Some(msg) => Some(msg),
                None => instr_var_sub_int(name_and_option.0, context)
            }
        },
        Some(a) if *a == TYPE_FLOAT => {
            match name_and_option.1 {
                Some(msg) => Some(msg),
                None => instr_var_sub_float(name_and_option.0, context)
            }
        },
        Some(a) => Some(format!("Variable type not recognize \"{a}\"")),
        _ => Some("Variable type required".to_string())
    }
}

fn instr_var_sub_name(context: Vec<&str>) -> (&str, Option<String>) {
    match context.get(1) {
        Some(v) => {
            for c in v.chars() {
                if !VARIABLE_NAME_CHARS.contains(&c) {
                    return ("", Some(format!("Invalid character in variable name \"{c}\"")));
                }
            }

            (*v, None)
        },
        None => ("", Some("Variable name required".to_string()))
    }
}

fn instr_var_sub_bool(name: &str, context: Vec<&str>) -> Option<String> {
    match context.get(2) {
        Some(b) => {
            let var_value;

            if *b == "true" {
                var_value = true;
            } else if *b == "false" {
                var_value = false;
            } else {
                return Some(format!("Not a valid boolean value \"{b}\""))
            }

            instr_var_sub_insert(name, VarValue::Bool(var_value));
            None
        }
        None => Some("Variable value required".to_string())
    }
}

fn instr_var_sub_int(name: &str, context: Vec<&str>) -> Option<String> {
    match context.get(2) {
        Some(i) => {
            let var_value;

            match i32::from_str(i) {
                Ok(a) => var_value = a,
                Err(_) => {
                    return Some(format!("Not a valid integer value \"{i}\""))
                }
            }
            instr_var_sub_insert(name, VarValue::Int(var_value));
            None
        }
        None => Some("Variable value required".to_string())
    }
}


fn instr_var_sub_float(name: &str, context: Vec<&str>) -> Option<String> {
    match context.get(2) {
        Some(i) => {
            let var_value;

            match f32::from_str(i) {
                Ok(a) => var_value = a,
                Err(_) => {
                    return Some(format!("Not a valid float value \"{i}\""))
                }
            }
            instr_var_sub_insert(name, VarValue::Float(var_value));
            None
        }
        None => Some("Variable value required".to_string())
    }
}

fn instr_var_sub_insert(name: &str, value: VarValue) {
    VAR_MAP.with(|map| {
        map.borrow_mut().insert(name.to_string(), value);
    })
}

fn resolve_variable(key: &str) -> (VarValue, Option<String>) {
    VAR_MAP.with(|map| {
        match map.borrow().get(key) {
            Some(v) => (v.clone(), None),
            None => (VarValue::Bool(false), Some(format!("No variable names \"{key}\" exists")))
        }
    })
}

fn resolve_variable_to_type(key: &str, expected_type: VarValue) -> (VarValue, Option<String>) {
    let result = resolve_variable(key);

    match result.1  {
        Some(a) => return (VarValue::Bool(false), Some(a)),
        None => {},
    }

    if !result.0.clone().is_same_type(&expected_type) {
        return (VarValue::Bool(false), Some(format!("Variable {key} is of type {} is expected to be of type {}", result.0.clone().type_to_string(), expected_type.type_to_string())))
    }

    (result.0, None)
}

fn create_too_many_args_error(expected: usize, given: usize) -> String {
    if expected == 1 {
        format!("This function takes 1 argument, but {given} were given")
    } else {
        format!("This function takes {expected} arguments, but {given} were given")
    }
}

fn extract_formula_to_int(formula: &str) -> (i32, Option<String>) {
    let chars = formula.chars();

    let mut sub_formulas = Vec::new();
    let mut current = String::new();
    let mut open_bracket = false;
    let mut bracket_was_closed_last = false;
    let mut sub_formulas_operators = Vec::new();
    for char in chars {
        if bracket_was_closed_last {
            sub_formulas_operators.push(match char_to_operator(char) {
                Some(o) => o,
                None => return (0, Some(format!("Operator or end expected after closing bracket in formula \"{formula}\"")))
            });

            continue
        }

        if char == '(' {
            if open_bracket {
                return (0, Some(format!("Invalid double opening bracket in formula \"{formula}\"")))
            }
            open_bracket = true;
        } else if char == ')' {
            if !open_bracket {
                return (0, Some(format!("Closing bracket without opening bracket in formula \"{formula}\"")))
            }

            sub_formulas.push(std::mem::take(&mut current));;
            bracket_was_closed_last = true;
            open_bracket = false;
        } else {
            current.push(char)
        }
    }

    let mut results = Vec::new();
    for sub_formula in sub_formulas {
        if sub_formula.len() < 3 {
            return (0, Some(format!("Sub-formula \"{sub_formula}\" does not match pattern [operand][operator][operand]")))
        }

        let mut operand1 = String::new();
        let mut operand2 = String::new();
        let mut operator = NoOperator;
        let mut is_first_part = true;
        for char in sub_formula.chars()  {
            if char.is_ascii_digit() {
                if is_first_part {
                    operand1.push(char)
                } else {
                    operand2.push(char)
                }
                continue
            }

            match char_to_operator(char) {
                Some(o) => {
                    if operator != NoOperator {
                        return (0, Some(format!("Sub-formula \"{sub_formula}\" contain a repetitive operator")))
                    }
                    operator = o;
                },
                None => return (0, Some(format!("Sub-formula \"{sub_formula}\" contains a character that is not allowed: \"{char}\"")))
            }

        }
        results.push(apply_operation_int(i32::from_str(operand1.as_str()).unwrap(), i32::from_str(operand2.as_str()).unwrap(), operator));
    }

    let mut current_number = results.remove(0);
    for (i,result) in results.iter().enumerate()  {
        current_number = apply_operation_int(current_number,*result, *sub_formulas_operators.get(i).unwrap())
    }

    (current_number, None)
}

fn prepare_context(context: &str) -> Vec<&str> {
    context.split(" ").collect()
}

fn char_to_operator(c: char) -> Option<Operator> {
    Some(match c {
        '+' => Add,
        '-' => Sub,
        '*' => Mul,
        '/' => Div,
        _ => return None
    })
}

fn apply_operation_int(operand1: i32, operand2: i32, operator: Operator) -> i32 {
    match operator  {
        Add => operand1+operand2,
        Sub => operand1-operand2,
        Mul => operand1*operand2,
        Div => operand1/operand2,
        _ => 0,
    }
}