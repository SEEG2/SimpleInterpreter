use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::env::var;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::FormulaElement::{ClosingBracket, Number, NumericOperator};
use crate::keyword::*;
use crate::Operator::{Add, Div, Mul, NoOperator, Sub};

mod keyword;
/* TODO:
         - Add conditions
         - Add pre-processor
         - Improve code quality and fix bugs
         - Improve instruction splitter
         - Improve performance
         - Add comments
 */
const PROGRAM: &str = "var int b -1;var float a #-1--1-1.0*-1*(-1.000000000*-1)*-1+((-0-0)+-0)+-(0+0)--$b;shout $a";
static mut INSTRUCTION_POINTER: isize = 0;
static mut INSTRUCTION_COUNTER: isize = 0;

thread_local! {
    static FLAG_MAP: RefCell<HashMap<String, usize>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    static VAR_MAP: RefCell<HashMap<String, VarValue>> =
        RefCell::new(HashMap::new());
}

#[derive(PartialEq, Clone)]
enum VarValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    Str(String)
}

impl VarValue {
    fn is_same_type(&self, other: &VarValue) -> bool {
        matches!(
            (self, other),
            (VarValue::Bool(_), VarValue::Bool(_)) |
            (VarValue::Int(_), VarValue::Int(_)) |
            (VarValue::Float(_), VarValue::Float(_)) |
            (VarValue::Str(_), VarValue::Str(_))
        )
    }

    fn type_to_string(&self) -> String {
        match self {
            VarValue::Bool(_) => TYPE_BOOL.to_string(),
            VarValue::Int(_) => TYPE_INT.to_string(),
            VarValue::Float(_) => TYPE_FLOAT.to_string(),
            VarValue::Str(_) => TYPE_STR.to_string()
        }
    }
}

impl Display for VarValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VarValue::Bool(v) => write!(f, "{}", v),
            VarValue::Int(v) => write!(f, "{}", v),
            VarValue::Float(v) => write!(f, "{}", v),
            VarValue::Str(v) => write!(f, "{}", v)
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

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Add => write!(f, "{OPERATOR_ADD}"),
            Sub => write!(f, "{OPERATOR_SUB}"),
            Mul => write!(f, "{OPERATOR_MUL}"),
            Div => write!(f, "{OPERATOR_DIV}"),
            NoOperator => write!(f, "{NONE_DISPLAY}"),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum BooleanOperator {
    And,
    Or,
    XOr,
    Not,
    NoBooleanOperator
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Comparator {
    Equals,
    LessThan,
    NoComparator
}

#[derive(PartialEq, Copy, Clone)]
enum FormulaElement {
    NumericOperator(Operator),
    Number(f32),
    OpeningBracket,
    ClosingBracket,
}

impl Display for FormulaElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericOperator(o) => o.fmt(f),
            Number(n) => write!(f, "{}", n),
            FormulaElement::OpeningBracket => write!(f, "{FORMULA_OPENING_BRACKET}"),
            FormulaElement::ClosingBracket => write!(f, "{FORMULA_CLOSING_BRACKET}"),
        }
    }
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
            let prepared_context = prepare_context(context_raw).ok()?;

            return match instruction {
                INSTR_TERMINATE => instr_terminate(prepared_context),
                INSTR_FLAG => instr_flag(prepared_context),
                INSTR_JUMP => instr_jump(prepared_context),
                INSTR_VAR => instr_var(prepared_context),
                _ => Some(format!("Instruction not found \"{}\"", instruction))
            }
        }
    }
}

fn instr_terminate(context_raw: Vec<String>) -> Option<String> {
    let argument: String = match context_raw.get(0) {
        Some(arg) => (*arg).to_string(),
        None => return Some("Termination code argument is missing".to_string()),
    };


    if context_raw.len() > 1 {
        return Some(create_wrong_args_count_error(1, context_raw.len()))
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
        return Some(create_wrong_args_count_error(1, context.len()))
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
        return Some(create_wrong_args_count_error(1, context.len()))
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
    let arg_count = context.len();
    if arg_count != 3 {
        return Some(create_wrong_args_count_error(3, arg_count))
    }

    let expected_type;
    let var_type = &context[0];
    if var_type == TYPE_BOOL {
        expected_type = VarValue::Bool(false)
    } else if var_type == TYPE_INT {
        expected_type = VarValue::Int(0)
    } else if var_type == TYPE_FLOAT {
        expected_type = VarValue::Float(0_f32)
    } else if var_type == TYPE_STR {
        expected_type = VarValue::Str(String::new())
    } else {
        return Some(format!("Variable type not recognize \"{var_type}\""))
    }

    let var_name = &context[1];
    for c in var_name.chars() {
        if !VARIABLE_NAME_CHARS.contains(&c) {
            return Some(format!("Invalid character '{c}' in variable name \"{var_name}\""));
        }
    }

    let value = &context[2];
    match expected_type {
        VarValue::Bool(_) => {
            match value.parse::<bool>() {
                Ok(b) => {
                    instr_var_sub_insert(var_name.as_str(), VarValue::Bool(b));
                    None
                },
                Err(_) => Some(format!("Value \"{value}\" is not of type {TYPE_BOOL}")),
            }
        }
        VarValue::Int(_) => {
            match value.parse::<i32>() {
                Ok(i) => {
                    instr_var_sub_insert(var_name.as_str(), VarValue::Int(i));
                    None
                },
                Err(_) => Some(format!("Value \"{value}\" is not of type {TYPE_INT}")),
            }
        }
        VarValue::Float(_) => {
            match value.parse::<f32>() {
                Ok(f) => {
                    instr_var_sub_insert(var_name.as_str(), VarValue::Float(f));
                    None
                },
                Err(_) => Some(format!("Value \"{value}\" is not of type {TYPE_FLOAT}")),
            }
        }
        VarValue::Str(_) => {
            match value.parse::<String>() {
                Ok(s) => {
                    instr_var_sub_insert(var_name.as_str(), VarValue::Str(s));
                    None
                },
                Err(_) => Some(format!("Value \"{value}\" is not of type {TYPE_STR}")),
            }
        }
    }
}

fn instr_var_sub_insert(name: &str, value: VarValue) {
    VAR_MAP.with(|map| {
        map.borrow_mut().insert(name.to_string(), value);
    })
}

fn resolve_variable(key: &str) -> Result<VarValue, String> {
    VAR_MAP.with(|map| {
        match map.borrow().get(key) {
            Some(v) => Ok(v.clone()),
            None => Err(format!("No variable names \"{key}\" exists"))
        }
    })
}

fn resolve_variable_to_f32(key: &str) -> Result<f32, String> {
    let v = resolve_variable(key)?;
    match v.to_string().parse::<f32>() {
        Ok(f) => Ok(f),
        Err(_) => Err(format!("Failed to parse variable \"{key}\" of type {} to a numeric value", v.type_to_string()))
    }
}

fn create_wrong_args_count_error(expected: usize, given: usize) -> String {
    if expected == 1 {
        format!("This call takes 1 argument, but {given} were given")
    } else {
        format!("This call takes {expected} arguments, but {given} were given")
    }
}

fn resolve_formula_to_float(formula: &str) -> Result<f32, String> {
    let mut is_reading_variable = false;
    let mut is_variable_end_reached = false;
    let mut current_variable = String::new();

    let mut is_reading_number = false;
    let mut is_number_end_reached = false;
    let mut current_number = String::new();

    let mut is_unary_possible_next = true;
    let mut is_unary_minus_set = false;

    let mut buffer = None;
    let mut parsed_formula = Vec::new();
    for (i, c) in formula.chars().enumerate() {
        if buffer.is_some() {
            is_unary_minus_set = false;
            is_unary_possible_next = buffer.unwrap() != ClosingBracket;

            parsed_formula.push(buffer.unwrap());
            buffer = None
        }

        if c == FORMULA_OPENING_BRACKET {
            buffer = Some(FormulaElement::OpeningBracket);
            if is_reading_variable {
                is_variable_end_reached = true;
            } else if is_reading_number {
                is_number_end_reached = true;
            } else {
                continue
            }
        } else if c == FORMULA_CLOSING_BRACKET {
            buffer = Some(FormulaElement::ClosingBracket);
            if is_reading_variable {
                is_variable_end_reached = true;
            } else if is_reading_number {
                is_number_end_reached = true;
            } else {
                continue
            }
        } else if let Some(op) = char_to_operator(c) {
            if is_unary_possible_next && op == Sub {
                if is_unary_minus_set {
                    return Err(format!("Double unary at position {i} in formula"))
                }
                is_unary_minus_set = true;
                continue;
            }
            buffer = Some(FormulaElement::NumericOperator(op));

            if is_reading_variable {
                is_variable_end_reached = true;
            } else if is_reading_number {
                is_number_end_reached = true;
            } else {
                continue
            }
        }

        if is_reading_number {
            if is_number_end_reached {
                let num_res = f32::from_str(current_number.as_str());
                match num_res {
                    Ok(r) =>  {
                        if is_unary_minus_set == true {
                            parsed_formula.push(Number(-r))
                        } else {
                            parsed_formula.push(Number(r))
                        }
                    },
                    Err(_) => {
                        return Err(format!("Cannot resolve number \"{current_number}\" in formula"))
                    }
                } ;

                current_number = String::new();

                is_reading_number = false;
                is_number_end_reached = false;
            } else {
                current_number.push(c);
            }

            is_unary_possible_next = false;
        } else if is_reading_variable {
            if is_variable_end_reached {
                let var_value = resolve_variable_to_f32(current_variable.as_str())?;

                if is_unary_minus_set == true {
                    parsed_formula.push(FormulaElement::Number(-var_value))
                } else {
                    parsed_formula.push(FormulaElement::Number(var_value))
                }

                current_variable = String::new();

                is_reading_variable = false;
                is_variable_end_reached = false;
            } else {
                current_variable.push(c);
            }

            is_unary_possible_next = false;
        } else if c == IND_RESOLVE_VARIABLE {
            is_reading_variable = true;

            is_unary_possible_next = false;
        } else if c.is_numeric() || c == DECIMAL_SEPARATOR {
            current_number.push(c);
            is_reading_number = true;

            is_unary_possible_next = false;
        } else {
            return Err(format!("Invalid char '{c}' at position {i} in formula"))
        }
    }

    if buffer.is_some() {
        parsed_formula.push(buffer.unwrap());
        buffer = None
    } else if is_reading_number {
        let num_res = f32::from_str(current_number.as_str());
        match num_res {
            Ok(r) =>  {
                if is_unary_minus_set == true {
                    parsed_formula.push(Number(-r))
                } else {
                    parsed_formula.push(Number(r))
                }
            },
            Err(_) => {
                return Err(format!("Cannot resolve number \"{current_number}\" in formula"))
            }
        } ;
    } else if is_reading_variable {
        let var_value = resolve_variable_to_f32(current_variable.as_str())?;

        if is_unary_minus_set == true {
            parsed_formula.push(FormulaElement::Number(-var_value))
        } else {
            parsed_formula.push(FormulaElement::Number(var_value))
        }
    }

    solve_formula_to_float(&mut parsed_formula, true)
}

fn solve_formula_to_float(formula: &mut Vec<FormulaElement>, scan_for_subformula: bool) -> Result<f32, String> {
    if formula.is_empty() {
        if scan_for_subformula {
            return Err("Formula is empty".to_string())
        }
        return Err("Formula contains empty bracket enclosure".to_string())
    }

    if scan_for_subformula {
        'outer: loop {
            let mut is_bracket_open = false;
            let mut bracket_start_index = 0;
            for (i, e) in formula.iter().enumerate() {
                if *e == FormulaElement::OpeningBracket {
                    bracket_start_index = i;
                    is_bracket_open = true;
                } else if *e == FormulaElement::ClosingBracket {
                    if !is_bracket_open {
                        return Err("Closing bracket without matching opening bracket".to_string())
                    }

                    let mut sub_formula = formula.drain(bracket_start_index+1..i).collect();

                    match solve_formula_to_float(&mut sub_formula, false) {
                        Ok(v) => {
                            formula[bracket_start_index] = FormulaElement::Number(v);
                            formula.remove(bracket_start_index+1);
                            continue 'outer;
                        },
                        Err(e) => return Err(e)
                    }
                }
            }

            if is_bracket_open {
                return Err("Opening bracket without matching closing bracket".to_string())
            }

            break;
        }
    }

    'outer: loop {
        for (i, e) in formula.iter().enumerate() {
            if let NumericOperator(operator @ (Mul | Div)) = *e {
                let Number(lhs) = formula[i-1] else {
                    return Err(format!("Expected a number left of operator '{operator}', but found '{}'", formula[i-1]));
                };

                let Number(rhs) = formula[i+1] else {
                    return Err(format!("Expected a number right of operator '{operator}', but found '{}'", formula[i+1]));
                };

                formula[i] = Number(apply_operation_float(lhs, rhs, operator));
                formula.remove(i+1);
                formula.remove(i-1);
                continue 'outer;
            }
        }

        break;
    }

    'outer: loop {
        for (i, e) in formula.iter().enumerate() {
            if let NumericOperator(operator @ (Add | Sub)) = *e {
                let Number(lhs) = formula[i-1] else {
                    return Err(format!("Expected a number left of operator '{operator}', but found '{}'", formula[i-1]));
                };

                let Number(rhs) = formula[i+1] else {
                    return Err(format!("Expected a number right of operator '{operator}', but found '{}'", formula[i+1]));
                };

                formula[i] = Number(apply_operation_float(lhs, rhs, operator));
                formula.remove(i+1);
                formula.remove(i-1);
                continue 'outer;
            }
        }

        break;
    }

    if formula.len() != 1 {
        if scan_for_subformula {
            return Err("Formula could not be resolved".to_string())
        }
        return Err("Sub-formula could not be resolved".to_string())
    };

    let Number(final_value) = formula[0] else {
        if scan_for_subformula {
            return Err("Formula could not be resolved".to_string())
        }
        return Err("Sub-formula could not be resolved".to_string())
    };

    Ok(final_value)
}

fn prepare_context(context: &str) -> Result<Vec<String>, String> {
    let mut arguments= Vec::new();
    let mut is_string_literal_open = false;
    let mut current_arg = String::new();
    let mut should_ignore_next_char = false;
    for c in context.chars()  {
        if should_ignore_next_char {
            current_arg.push(c);
            should_ignore_next_char = false;
        } else if is_string_literal_open {
            if c == STR_LITERAL_INDICATOR {
                is_string_literal_open = false;
                continue;
            }
            current_arg.push(c);
        } else if c == STR_LITERAL_INDICATOR {
            is_string_literal_open = true;
        } else if c == IND_STRING_IGNORE {
            if current_arg.is_empty() {
                current_arg.push(c);
            }
            should_ignore_next_char = true;
        } else if c == ARGUMENT_SEPARATOR && !is_string_literal_open {
            if current_arg.is_empty() {
                continue
            }
            arguments.push(current_arg.clone());
            current_arg = String::new();
        } else {
            current_arg.push(c);
        }
    }

    if is_string_literal_open {
        return Err("String literal was opened but never closed".to_string())
    }

    if !is_string_literal_open && !current_arg.is_empty() {
        arguments.push(current_arg)
    }

    let mut i = 0;
    while i < arguments.len()  {
        match prepare_argument(&mut arguments[i]) {
            Some(s) => return Err(s),
            None => {}
        }

        i+=1;
    }

   Ok(arguments)
}

fn prepare_argument(argument: &mut String) -> Option<String> {
    assert!(!argument.is_empty());

    let mut chars = argument.chars();

    let ch = chars.next().unwrap();

    if ch == IND_STRING_IGNORE {
        *argument = chars.as_str().to_string();
        return None;
    } else if ch == IND_RESOLVE_VARIABLE {
        let var_value = resolve_variable(chars.as_str()).ok()?;
        *argument = var_value.to_string();
    } else if ch == IND_FORMULA_FLOAT {
        let number_value = resolve_formula_to_float(chars.as_str()).ok()?;
        *argument = number_value.to_string();
    } else if ch == IND_FORMULA_ROUNDED {
        let number_value = resolve_formula_to_float(chars.as_str()).ok()?;
        *argument = (number_value.round() as i32).to_string();
    }

    None
}

fn char_to_comparator(c: char) -> Option<Comparator> {
    Some(match c {
        BOOL_COMPARATOR_EQUAL => Comparator::Equals,
        BOOL_COMPARATOR_LESS_THAN => Comparator::LessThan,
        _ => return None
    })
}

fn char_to_boolean_operator(c: char) -> Option<BooleanOperator> {
    Some(match c {
        BOOL_OPERATOR_AND => BooleanOperator::And,
        BOOL_OPERATOR_OR => BooleanOperator::Or,
        BOOL_OPERATOR_XOR => BooleanOperator::XOr,
        BOOL_OPERATOR_NOT => BooleanOperator::Not,
        _ => return None
    })
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