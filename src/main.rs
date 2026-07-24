use std::any::Any;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::thread::current;
use crate::keyword::*;
use crate::Operator::{Add, Div, Mul, NoOperator, Sub};

mod keyword;


/* TODO: - Improve formula parsing
         - Add conditions
         - Add pre-processor
         - Improve code quality and fix bugs
         - Improve instruction splitter
         - Add comments
 */
pub const PROGRAM: &str = "var int a 1;var int b 2;var int ~($a+$b) ~($a/$b);shout $3;shout \n;shout ~(2)+(4)*(2);shout \n;terminate ~($3-2)+(2*1)";
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
            let mut operation_and_context = operation.splitn(2, ARGUMENT_SEPARATOR);
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

fn interpret_instruction(instruction: &str, context_raw: &str) -> Option<String> {
    match instruction {
        INSTR_SHOUT => instr_shout(context_raw),
        _ => {
            let result = prepare_context(context_raw);
            if result.1.is_some() {
                return result.1;
            }

            return match instruction {
                INSTR_TERMINATE => instr_terminate(result.0),
                INSTR_FLAG => instr_flag(result.0),
                INSTR_JUMP => instr_jump(result.0),
                INSTR_VAR => instr_var(result.0),
                _ => Some(format!("Instruction not found \"{}\"", instruction))
            }
        }
    }
}

fn instr_terminate(context_raw: Vec<String>) -> Option<String> {
    let mut argument: String = match context_raw.get(0) {
        Some(arg) => (*arg).to_string(),
        None => return Some("Termination code argument is missing".to_string()),
    };


    if context_raw.len() > 1 {
        return Some(create_too_many_args_error(1, context_raw.len()))
    }

    let exit_code;

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

fn instr_flag(context: Vec<String>) -> Option<String> {
    let argument = match context.get(0) {
        Some(arg) => arg,
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

fn instr_jump(context: Vec<String>) -> Option<String> {
    let argument = match context.get(0) {
        Some(arg) => arg,
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

    let mut formatted = context_raw.to_string();
    match prepare_argument(&mut formatted) {
        Some(s) => return Some(s),
        None => {
            print!("{}", formatted);
            None
        }
    }
}

fn instr_var(context: Vec<String>) -> Option<String> {
    let name_and_option = instr_var_sub_name(context.clone());

    if context.len() > 3 {
        return Some(create_too_many_args_error(3, context.len()))
    }

    match context.get(0) {
        Some(a) if *a == TYPE_BOOL => {
            match name_and_option.1 {
                Some(msg) => Some(msg),
                None => instr_var_sub_bool(&*name_and_option.0, context)
            }
        },
        Some(a) if *a == TYPE_INT => {
            match name_and_option.1 {
                Some(msg) => Some(msg),
                None => instr_var_sub_int(&*name_and_option.0, context)
            }
        },
        Some(a) if *a == TYPE_FLOAT => {
            match name_and_option.1 {
                Some(msg) => Some(msg),
                None => instr_var_sub_float(&*name_and_option.0, context)
            }
        },
        Some(a) => Some(format!("Variable type not recognize \"{a}\"")),
        _ => Some("Variable type required".to_string())
    }
}

fn instr_var_sub_name(context: Vec<String>) -> (String, Option<String>) {
    match context.get(1) {
        Some(v) => {
            for c in v.chars() {
                if !VARIABLE_NAME_CHARS.contains(&c) {
                    return ("".to_string(), Some(format!("Invalid character in variable name \"{c}\"")));
                }
            }

            (v.to_string(), None)
        },
        None => ("".to_string(), Some("Variable name required".to_string()))
    }
}

fn instr_var_sub_bool(name: &str, context: Vec<String>) -> Option<String> {
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

fn instr_var_sub_int(name: &str, context: Vec<String>) -> Option<String> {
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


fn instr_var_sub_float(name: &str, context: Vec<String>) -> Option<String> {
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

fn extract_formula_to_float(formula: &str) -> (f32, Option<String>) {
    let result = extract_sub_formulas(formula);

    if result.2.is_some() {
        return (0_f32, result.2);

    }

    let sub_formulas = result.0;
    let mut sub_formulas_operators = result.1;

    let mut results = Vec::new();
    for sub_formula in sub_formulas {
        if sub_formula.is_empty() {
            return (0_f32, Some(format!("Sub-formula \"{sub_formula}\" is empty")))
        }

        let mut operand1 = String::new();
        let mut operand2 = String::new();
        let mut operator = NoOperator;
        let mut is_reading_variable = false;
        let mut is_first_part = true;
        let mut latest_variable_string = String::new();
        for char in sub_formula.chars()  {
            if char == IND_RESOLVE_VARIABLE {
                is_reading_variable = true;
                continue;
            }

            if is_reading_variable {
                if (!operand1.is_empty() && operator == NoOperator) || !operand2.is_empty()  {
                    return (0_f32, Some(format!("Sub-formula \"{sub_formula}\" contains a variable resolve in an invalid position")))
                }

                let potential_operator = char_to_operator(char);
                if potential_operator != None {
                    operator = potential_operator.unwrap();

                    let resolved_result = resolve_variable_to_type(latest_variable_string.as_str(), VarValue::Int(0));
                    match resolved_result.1 {
                        Some(s) => return (0_f32, Some(s)),
                        None => (),
                    }

                    if is_first_part {
                        operand1 = resolved_result.0.to_string();
                    } else {
                        return (0_f32, Some(format!("Sub-formula \"{sub_formula}\" contains an operator in an invalid position")))
                    }

                    latest_variable_string = String::new();
                    is_reading_variable = false;
                    is_first_part = false;

                    continue
                }
                
                latest_variable_string.push(char);
                continue
            }

            if char.is_ascii_digit() || char == DECIMAL_SEPARATOR {
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
                        return (0_f32, Some(format!("Sub-formula \"{sub_formula}\" contains more than one operator")))
                    }
                    operator = o;
                    is_first_part = false;
                },
                None => return (0_f32, Some(format!("Sub-formula \"{sub_formula}\" contains a character that is not allowed: \"{char}\"")))
            }

        }
        
        if is_reading_variable {
            let resolved_result = resolve_variable_to_type(latest_variable_string.as_str(), VarValue::Int(0));
            match resolved_result.1 {
                Some(s) => return (0_f32, Some(s)),
                None => (),
            }

            if is_first_part {
                operand1 = resolved_result.0.to_string();
            } else {
                operand2 = resolved_result.0.to_string();
            }

        }

        if operand2.is_empty() {
            results.push(f32::from_str(operand1.as_str()).unwrap())
        } else {
            results.push(apply_operation_float(f32::from_str(operand1.as_str()).unwrap(), f32::from_str(operand2.as_str()).unwrap(), operator));
        }
    }

    'outer: while results.len() > 1 {
         for i in 0..sub_formulas_operators.len() {
            let operator = sub_formulas_operators[i];
            if operator == Mul || operator == Div {
                results[i] = apply_operation_float(results[i],results.remove(i+1), operator);
                sub_formulas_operators.remove(i);
                break 'outer;
            }
        }

        break;
    }

    let mut current_number = results.remove(0);
    for (i,result) in results.iter().enumerate()  {
        current_number = apply_operation_float(current_number,*result, *sub_formulas_operators.get(i).unwrap())
    }

    (current_number, None)
}

fn extract_sub_formulas(formula: &str) -> (Vec<String>, Vec<Operator>, Option<String>) {
    let chars = formula.chars();
    let mut current = String::new();
    let mut open_bracket = false;
    let mut bracket_was_closed_last = false;
    let mut sub_formulas = Vec::new();
    let mut sub_formulas_operators = Vec::new();

    for char in chars {
        if bracket_was_closed_last {
            sub_formulas_operators.push(match char_to_operator(char) {
                Some(o) => o,
                None => return (Vec::new(), Vec::new(), Some(format!("Operator or end expected after closing bracket in formula \"{formula}\"")))
            });

            bracket_was_closed_last = false;
            current = String::new();
            continue
        }

        if char == '(' {
            if open_bracket {
                return (Vec::new(), Vec::new(), Some(format!("Invalid double opening bracket in formula \"{formula}\"")))
            }
            open_bracket = true;
        } else if char == ')' {
            if !open_bracket {
                return (Vec::new(), Vec::new(), Some(format!("Missing opening bracket in formula \"{formula}\"")))
            }

            sub_formulas.push(std::mem::take(&mut current));

            bracket_was_closed_last = true;
            open_bracket = false;
        } else {
            current.push(char)
        }
    }

    if !bracket_was_closed_last {
        return (Vec::new(), Vec::new(), Some(format!("Missing closing bracket in formula \"{formula}\"")))
    }

    (sub_formulas, sub_formulas_operators, None)
}

fn prepare_context(context: &str) -> (Vec<String>, Option<String>) {
    let mut split_context: Vec<String> = context.split(ARGUMENT_SEPARATOR).map(String::from).collect();

    let mut i = 0;
    while i < split_context.len()  {
        if split_context[i].is_empty() {
            split_context.remove(i);
            continue;
        }

        match prepare_argument(&mut split_context[i]) {
            Some(s) => return (Vec::new(), Some(s)),
            None => {}
        }

        i+=1;
    }

   (split_context, None)
}

fn prepare_argument(argument: &mut String) -> Option<String> {
    if argument.is_empty() {
        return Some("Empty argument was provided".to_string())
    }
    let mut chars = argument.chars();

    let ch = chars.next().unwrap();

    if ch == IND_STRING_IGNORE {
        *argument = chars.as_str().to_string();
        return None;
    } else if ch == IND_RESOLVE_VARIABLE {
        let r = resolve_variable(chars.as_str());
        return match r.1 {
            Some(s) => Some(s),
            None => {
                *argument = r.0.to_string();
                None
            }
        }
    } else if ch == IND_FORMULA_FLOAT {
        let r = extract_formula_to_float(chars.as_str());
        return match r.1 {
            Some(s) => Some(s),
            None => {
                *argument = r.0.to_string();
                None
            }
        }
    } else if ch == IND_FORMULA_ROUNDED {
        let r = extract_formula_to_float(chars.as_str());
        return  match r.1 {
            Some(s) => return Some(s),
            None => {
                *argument = (r.0.round() as i32).to_string();
                None
            }
        }
    }

    None
}

fn char_to_operator(c: char) -> Option<Operator> {
    Some(match c {
        OPERATOR_ADD => Add,
        OPERATOR_SUB => Sub,
        OPERATOR_MUL => Mul,
        OPERATOR_DIV => Div,
        _ => return None
    })
}

fn apply_operation_float(operand1: f32, operand2: f32, operator: Operator) -> f32 {
    match operator  {
        Add => operand1+operand2,
        Sub => operand1-operand2,
        Mul => operand1*operand2,
        Div => operand1/operand2,
        _ => 0_f32,
    }
}