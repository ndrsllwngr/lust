use crate::lex::*;

#[derive(Debug)]
pub enum Literal {
    Identifier(Token),
    Number(Token),
}

#[derive(Debug)]
pub struct FunctionCall {
    pub name: Token,
    pub arguments: Vec<Expression>,
}

#[derive(Debug)]
pub struct BinaryOperation {
    pub operator: Token,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
    BinaryOperation(BinaryOperation),
    Literal(Literal),
}

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct If {
    pub test: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Local {
    pub name: Token,
    pub expression: Expression,
}

#[derive(Debug)]
pub struct Return {
    pub expression: Expression,
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    If(If),
    FunctionDeclaration(FunctionDeclaration),
    Return(Return),
    Local(Local),
}

pub type AST = Vec<Statement>;

fn expect_keyword(tokens: Vec<Token>, index: usize, value: &str) -> bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Keyword && t.value == value;
}

fn expect_syntax(tokens: Vec<Token>, index: usize, value: &str) -> bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Syntax && t.value == value;
}

fn expect_identifier(tokens: Vec<Token>, index: usize) -> bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Identifier;
}

fn expect_number(tokens: Vec<Token>, index: usize) -> bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Number;
}

fn parse_expression(raw: &Vec<char>, tokens: Vec<Token>, index: usize) -> Option<(Expression, usize)> {
    if !expect_identifier(tokens, index) || expect_number(tokens, index) {
	return None;
    }

    let left = match tokens[index].kind {
	TokenKind::Number => Expression::Literal(Literal::Number(tokens[index])),
	TokenKind::Identifier => Expression::Literal(Literal::Identifier(tokens[index])),
    };
    let mut next_index = index + 1;
    if expect_syntax(tokens, next_index, "(") {
	next_index += 1; // Skip past open paren

	// Function call
	let mut arguments: Vec<Expression> = vec![];
	while !expect_syntax(tokens, next_index, ")") {
	    if arguments.len() > 0 {
		if !expect_syntax(tokens, next_index, ",") {
		    println!("{}", tokens[next_index].loc.debug(*raw, "Expected comma between function call arguments:"));
		    return None;
		}

		next_index += 1; // Skip past comma
	    }

	    let res = parse_expression(raw, tokens, next_index);
	    if res.is_some() {
		let (arg, next_next_index) = res.unwrap();
		next_index = next_next_index;
		arguments.push(arg);
	    } else {
		println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid expression in function call arguments:"));
		return None;
	    }
	}

	next_index += 1; // Skip past closing paren

	return Some((Expression::FunctionCall(FunctionCall{name: tokens[index], arguments: arguments}), next_index))
    }

    // Otherwise is a binary operation
    if next_index >= tokens.len() || tokens[next_index].kind != TokenKind::Syntax {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid binary operation:"));
	return None;
    }

    let op = tokens[next_index];
    next_index += 1; // Skip past op

    if !expect_identifier(tokens, next_index) || !expect_number(tokens, next_index) {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid right hand side binary operand:"));
	return None;
    }

    let right = match tokens[next_index].kind {
	TokenKind::Number => Expression::Literal(Literal::Number(tokens[next_index])),
	TokenKind::Identifier => Expression::Literal(Literal::Identifier(tokens[next_index])),
    };
    next_index += 1; // Skip past right hand operand

    Some((Expression::BinaryOperation(BinaryOperation{left: Box::new(left), right: Box::new(right), operator: op}), next_index))
}

fn parse_function(raw: &Vec<char>, tokens: Vec<Token>, index: usize) -> Option<(Statement, usize)> {
    if !expect_keyword(tokens, index, "function") {
	return None;
    }

    let mut next_index = index + 1;
    if !expect_identifier(tokens, next_index) {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid identifier for function name:"));
	return None;
    }
    let name = tokens[next_index];

    next_index += 1; // Skip past name
    if !expect_syntax(tokens, next_index, "(") {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected open parenthesis in function declaration:"));
	return None;
    }

    next_index += 1; // Skip past open paren
    let parameters: Vec<Token> = vec![];
    while !expect_syntax(tokens, next_index, ")") {
	if parameters.len() > 0 {
	    if !expect_syntax(tokens, next_index, ",") {
		println!("{}", tokens[next_index].loc.debug(*raw, "Expected comma or close parenthesis after parameter in function declaration:"));
		return None;
	    }

	    next_index += 1; // Skip past comma
	}

	parameters.push(tokens[next_index]);
    }

    next_index += 1; // Skip past close paren

    let statements: Vec<Statement> = vec![];
    while !expect_keyword(tokens, next_index, "end") {
	let res = parse_statement(raw, tokens, next_index);
	if res.is_some() {
	    let (stmt, next_next_index) = res.unwrap();
	    next_index = next_next_index;
	    statements.push(stmt);
	} else {
	    println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid statement in function declaration:"));
	    return None;
	}
    }

    next_index += 1; // Skip past end

    Some((Statement::FunctionDeclaration(FunctionDeclaration{
	name: name,
	parameters: parameters,
	body: statements,
    }), next_index))
}

fn parse_return(raw: &Vec<char>, tokens: Vec<Token>, index: usize) -> Option<(Statement, usize)> {
    if !expect_keyword(tokens, index, "return") {
	return None;
    }

    let mut next_index = index + 1; // Skip past return
    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid expression in return statement:"));
	return None;
    }

    let (expr, next_next_index) = res.unwrap();
    next_index = next_next_index;
    if !expect_syntax(tokens, next_index, ";") {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected semicolon in return statement:"));
	return None;
    }

    next_index += 1; // Skip past semicolon

    Some((Statement::Return(Return{expression: expr}), next_index))
}

fn parse_local(raw: &Vec<char>, tokens: Vec<Token>, index: usize) -> Option<(Statement, usize)> {
    if !expect_keyword(tokens, index, "local") {
	return None;
    }

    let mut next_index = index + 1; // Skip past local

    if !expect_identifier(tokens, next_index) {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid identifier for function name:"));
	return None;
    }

    let name = tokens[next_index];
    next_index += 1; // Skip past name

    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid expression in local declaration:"));
	return None;
    }

    let (expr, next_next_index) = res.unwrap();
    next_index = next_next_index;

    if !expect_syntax(tokens, next_index, ";") {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected semicolon in return statement:"));
	return None;
    }

    next_index += 1; // Skip past semicolon

    Some((Statement::Local(Local{name: name, expression: expr}), next_index))
}

fn parse_if(raw: &Vec<char>, tokens: Vec<Token>, index: usize) -> Option<(Statement, usize)> {
    if !expect_keyword(tokens, index, "if") {
	return None;
    }

    let mut next_index = index + 1; // Skip past if
    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid expression for if test:"));
	return None;
    }

    let (test, next_next_index) = res.unwrap();
    next_index = next_next_index;

    if !expect_keyword(tokens, next_index, "then") {
	return None;
    }

    next_index += 1; // Skip past then

    let statements: Vec<Statement> = vec![];
    while !expect_keyword(tokens, next_index, "end") {
	let res = parse_statement(raw, tokens, next_index);
	if res.is_some() {
	    let (stmt, next_next_index) = res.unwrap();
	    next_index = next_next_index;
	    statements.push(stmt);
	} else {
	    println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid statement in if body:"));
	    return None;
	}
    }

    next_index += 1; // Skip past end

    Some((Statement::If(If{test: test, body: statements}), next_index))
}

fn parse_expression_statement(raw: &Vec<char>, tokens: Vec<Token>, index: usize) -> Option<(Statement, usize)> {
    let mut next_index = index;
    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected valid expression in statement:"));
	return None;
    }

    let (expr, next_next_index) = res.unwrap();
    next_index = next_next_index;
    if !expect_syntax(tokens, next_index, ";") {
	println!("{}", tokens[next_index].loc.debug(*raw, "Expected semicolon after expression:"));
	return None;
    }

    next_index += 1; // Skip past semicolon

    Some((Statement::Expression(expr), next_index))
}

fn parse_statement(raw: &Vec<char>, tokens: Vec<Token>, index: usize) -> Option<(Statement, usize)> {
    let parsers = [parse_if, parse_expression_statement, parse_return, parse_function, parse_local];
    for parser in parsers {
	let res = parser(raw, tokens, index);
	if res.is_some() {
	    return res;
	}
    }

    None
}

pub fn parse(raw: &Vec<char>, tokens: Vec<Token>) -> Result<AST, String> {
    let ast = vec![];
    let mut index = 0;
    while index < tokens.len() {
	let res = parse_statement(raw, tokens, index);
	if res.is_some() {
	    let (stmt, next_index) = res.unwrap();
	    index = next_index;
	    ast.push(stmt);
	    continue;
	}

	return Err(tokens[index].loc.debug(*raw, "Invalid token while parsing:"));
    }

    Ok(ast)
}
