pub const INSTR_TERMINATE: &str = "terminate";
pub const INSTR_JUMP: &str = "jump";
pub const INSTR_FLAG: &str = "flag";
pub const INSTR_SHOUT: &str = "shout";
pub const INSTR_VAR: &str = "var";
pub const INSTR_IFDO: &str = "ifdo";
pub const INSTR_NOP: &str = "nop";

pub const IND_RESOLVE_VARIABLE: char = '$';
pub const IND_STRING_IGNORE: char = '%';
pub const IND_FORMULA_FLOAT: char = '#';
pub const IND_FORMULA_ROUNDED: char = '~';

pub const TYPE_BOOL: &str = "bool";
pub const TYPE_INT: &str = "int";
pub const TYPE_FLOAT: &str = "float";
pub const TYPE_STR: &str = "str";

pub const OPERATOR_ADD: char = '+';
pub const OPERATOR_SUB: char = '-';
pub const OPERATOR_MUL: char = '*';
pub const OPERATOR_DIV: char = '/';

pub const FORMULA_OPENING_BRACKET: char = '(';
pub const FORMULA_CLOSING_BRACKET: char = ')';

pub const DECIMAL_SEPARATOR: char = '.';

pub const BOOL_OPERATOR_AND: char = '&';
pub const BOOL_OPERATOR_OR: char = '|';
pub const BOOL_OPERATOR_XOR: char = '^';
pub const BOOL_OPERATOR_NOT: char = '!';

pub const BOOL_COMPARATOR_EQUAL: char = '=';
pub const BOOL_COMPARATOR_LESS_THAN: char = '<';
pub const BOOL_COMPARATOR_GREATER_THAN: char = '>';

pub const IND_BOOL_COMPARISON: char = '?';

pub const ARGUMENT_SEPARATOR: char = ' ';
pub const STR_LITERAL_INDICATOR: char = '"';
pub const VARIABLE_NAME_CHARS: &[char] = &['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z',
    '0','1','2','3','4','5','6','7','8','9',
    '_'];